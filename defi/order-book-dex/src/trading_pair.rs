use std::cmp::min;

use scrypto::prelude::*;

use crate::model::*;

blueprint! {

    /// This component represents a trading pair like e.g. XRD/rUSD. It can be used by traders to
    /// create limit and market orders.
    struct TradingPair {
        /// Internal authority for minting order NFRs (non fungible resources)
        minter: Vault,

        /// A NFR that represents limit orders
        order_resource: ResourceAddress,

        /// The order book that holds all limit orders that are created for this trading pair
        order_book: OrderBook,

        /// The resources that market makers have deposited into this component by creating limit ask orders
        base_funds: Vault,

        /// The resources that market makers have deposited into this component by creating limit bid orders
        quote_funds: Vault,
    }

    impl TradingPair {
        /// Instantiates a new TradingPair component for the given base_resource and quote_resource.
        /// To create a trading pair XRD/rUSD, one would issue a call like this: `instantiate(xrd_address, rusd_address)`
        pub fn instantiate(base_resource: ResourceAddress, quote_resource: ResourceAddress) -> ComponentAddress {
            assert_is_fungible(&base_resource);
            assert_is_fungible(&quote_resource);

            let minter =
                ResourceBuilder::new_fungible().divisibility(DIVISIBILITY_NONE).initial_supply(1);

            // This NFR represents the orders managed by this trading pair.
            // Orders an be minted, burned and updated by this component.
            let order_resource = ResourceBuilder::new_non_fungible()
                .metadata(
                    "name",
                    format!(
                        "Dex limit order ({}/{})",
                        base_resource,
                        quote_resource
                    ),
                )
                .mintable(rule!(require(minter.resource_address())), LOCKED)
                .burnable(rule!(require(minter.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(minter.resource_address())), LOCKED)
                .no_initial_supply();

            Self {
                minter: Vault::with_bucket(minter),
                order_resource,
                order_book: OrderBook::new(),
                base_funds: Vault::new(base_resource),
                quote_funds: Vault::new(quote_resource),
            }
            .instantiate().globalize()
        }

        /// Creates a new limit order. The side of the order (Ask/Bid) is derived form the given funds bucket.
        /// If the base resource is supplied in that bucket, an Ask order is inferred, if the quote resource is supplied
        /// a Bid order is inferred. The order is created with the given price. The price must always be (irrespective of the side)
        /// the amount of the quote resource that must be paid in order to obtain one unit of the base resource!
        ///
        /// Returns a bucket with a NFR that represents the order. This NFR can be used to track the order
        /// and to cancel it or redeem the traded resources.
        ///
        /// Panics if the order would be a market order. Never executes directly against existing market orders.
        /// Panics if price is <= 0
        pub fn new_limit_order(&mut self, funds: Bucket, price: Decimal) -> Bucket {
            // Determine the side of the order
            let side = self.get_order_side(&funds);
            // Generate a new random order key
            let order_key = NonFungibleId::random();
            // Create a new limit order object. This will check that the price is not <= 0
            let order = LimitOrder::new(order_key.clone(), side, price, funds.amount());
            // Insert the limit order into the order book. This panics if the order would be a market order.
            // The order book will only store a reference to the order (the order key). The order NFR will be given to the
            // user at the end of this method.
            self.order_book.insert_limit_order(&order);

            // Store the user supplied funds in the correct vault
            match side {
                Side::Ask => self.base_funds.put(funds),
                Side::Bid => self.quote_funds.put(funds),
            }

            // Mint a new NFR representing the order and give it to the user
            self.minter.authorize(|| {
                borrow_resource_manager!(self.order_resource)
                    .mint_non_fungible(&order_key, order)
            })
        }

        /// Closes the given limit order. If the order has not been filled completely, it is canceled, i.e. removed from the order book.
        /// The two buckets contain
        /// 1. the refund of the provided resource, in case the order has not been filled completely
        /// 2. the traded/received resource, in case the order has been filled/filled partially
        pub fn close_limit_order(&mut self, order_bucket: Bucket) -> (Bucket, Bucket) {
            // Make sure the given bucket does indeed contain an order NFR
            assert_eq!(
                order_bucket.resource_address(),
                self.order_resource,
                "Invalid resource supplied: bucket does not contain an order"
            );
            // Load the data belonging to the order NFR
            let order: LimitOrder =
                order_bucket.non_fungible::<LimitOrder>().data();

            // If the order has not been filled completely, it still is referenced in the order book so we have to remove it.
            // If the order has already been filled completely, it will already have been removed from the order book.
            if order.quantity_filled < order.quantity {
                self.order_book.remove_limit_order(&order);
            }
            // Burn the order NFR. It is no longer needed as the order will no longer exist after this method finishes.
            self.minter
                .authorize(|| order_bucket.burn());

            // Calculate 1) the amount the user must be refunded if their order has not been filled completely
            // and 2) the amount that the user has traded successfully if their order has been (at least partially) filled.
            let (refund_amount, traded_amount) = order.calculate_close_amounts();

            // Depending on the order side, take the refund and traded resources out of the correct vaults and give them to the user.
            match order.side {
                Side::Ask => (
                    self.base_funds.take(refund_amount),
                    self.quote_funds.take(traded_amount),
                ),
                Side::Bid => (
                    self.quote_funds.take(refund_amount),
                    self.base_funds.take(traded_amount),
                ),
            }
        }

        /// Creates a new market order that is executed directly against existing limit orders.
        /// The side of the order is derived from the given funds bucket.
        /// If the base resource is supplied in that bucket, an Ask order is inferred, if the quote resource is supplied,
        /// a Bid order is inferred.
        ///
        /// A user calling this method should place an assertion in their transaction manifest, enforcing that at least
        /// some minimum amount of the traded resource is received. This is especially important if the user places a
        /// large order and "rides down" the order book multiple price levels.
        ///
        /// Returns two buckets with
        /// 1. Any dust that might still exist in the supplied funds bucket after executing the market order.
        /// 2. The traded funds that are received in exchange for the supplied funds.
        ///
        /// Panics if the order cannot be filled by existing limit orders.
        pub fn new_market_order(&mut self, mut funds: Bucket) -> (Bucket, Option<Bucket>) {
            // Infer the side of the order
            let market_order_side = self.get_order_side(&funds);
            let limit_order_side = market_order_side.opposite();

            // Create a bucket for accumulating all traded funds
            // (funds the user will receive from limit orders that they have filled)
            let mut funds_to_return: Option<Bucket> = None;

            // Enter into a loop of always loading the best limit order from the order book and
            // filling it. Stop when the funds of the market order are expended.
            // This may fail in a low liquidity situation where there are too few funds on the limit
            // order side to fill the market order.
            let mut last_price = Decimal(1i128);
            while !is_almost_zero(funds.amount(), last_price) {
                // Get the current best limit order from the order book (this only returns the order key).
                // Panic if there are no more limit orders that can be matched to the market order
                let limit_order_key = self.order_book.get_best_order(limit_order_side)
                    .expect("Insufficient liquidity: no limit orders found that can be matched to the market order")
                    .clone();

                // Using the order key, load the data for the limit order
                let mut limit_order: LimitOrder =
                    borrow_resource_manager!(self.order_resource).get_non_fungible_data(&limit_order_key);

                // Save the limit order's price as the last known price
                last_price = limit_order.price;

                // Calculate what quantity of the limit order can be filled using the funds that remain in the market order
                let supplied_quantity = match limit_order_side {
                    Side::Ask => funds.amount() / limit_order.price,
                    Side::Bid => funds.amount() * limit_order.price,
                };

                // Also calculate the quantity of the limit order that has not been filled yet
                let unfilled_quantity = limit_order.quantity - limit_order.quantity_filled;

                // The fill quantity for this loop pass. This is the minimum of the quantity supplied via the market order
                // and the unfilled quantity remaining in the limit order
                let fill_quantity = min(supplied_quantity, unfilled_quantity);

                // Then, fill the limit order wit this quantity
                limit_order.fill(fill_quantity);

                // Depending on the limit order side, take an appropriate amount of the market order funds and store them in this component.
                // The market maker user will be able to claim them later. Also add the traded funds that the taker user will receive to the
                // funds_to_return bucket.
                
                let bucket_to_add = match limit_order_side {
                    Side::Ask => {
                        self.quote_funds
                            .put(funds.take(fill_quantity * limit_order.price));
                        self.base_funds.take(fill_quantity)
                    }
                    Side::Bid => {
                        self.base_funds
                            .put(funds.take(fill_quantity / limit_order.price));
                        self.quote_funds.take(fill_quantity)
                    }
                };

                match funds_to_return.as_mut() {
                    Some(bucket) => bucket.put(bucket_to_add),
                    None => funds_to_return = Some(bucket_to_add)
                }

                // Check if the limit order has been filled completely.
                // If so, remove it from the order book.
                // The limit order NFR representing the limit order remains in the user's possession.
                if limit_order.quantity_filled == limit_order.quantity {
                    self.order_book.remove_limit_order(&limit_order);
                }

                // Update the data of the limit order NFR
                self.minter.authorize(|| {
                    borrow_resource_manager!(self.order_resource).update_non_fungible_data(
                        &limit_order_key,
                        limit_order
                    )
                });
            }

            // Finally return to the user 1) the unspent funds of the market order
            // and 2) the traded funds coming from the limit order(s)
            (funds, funds_to_return)
        }

        /// Infers the side of the order from the resource contained in the given bucket.
        /// If the bucket contains the base resource Ask is inferred.
        /// If the bucket contains the quote resource Bid is inferred.
        fn get_order_side(&self, bucket: &Bucket) -> Side {
            if bucket.resource_address() == self.base_funds.resource_address() {
                Side::Ask
            } else if bucket.resource_address() == self.quote_funds.resource_address() {
                Side::Bid
            } else {
                panic!("The supplied bucket contains an invalid resource")
            }
        }
    }
}

fn assert_is_fungible(resource: &ResourceAddress) {
    match borrow_resource_manager!(*resource).resource_type() {
        ResourceType::Fungible { .. } => (), // OK
        ResourceType::NonFungible => panic!(
            "Invalid resource: resource {} is non fungible and \
        cannot be exchanged via a trading pair",
            resource
        ),
    }
}

/// Determines if the given amount is as good as zero with respect to the given price.
/// Essentially, this is used to check whether enough funds remain in a market order
/// in order to at least partially fill the next limit order.
///
/// Return true if the given amount can be considered almost zero, else false
fn is_almost_zero(amount: Decimal, price: Decimal) -> bool {
    let eps = if price >= Decimal::one() {
        price
    } else {
        Decimal::one() / price
    };
    let eps = Decimal(1i128) * eps;

    amount < eps
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_almost_zero() {
        // price, almost_zero, not_almost_zero
        struct TestCase(Decimal, Decimal, Decimal);
        let test_cases = vec![
            TestCase("0.001".into(), Decimal(999i128), Decimal(1000i128)),
            TestCase("0.1".into(), Decimal(9i128), Decimal(10i128)),
            TestCase("1".into(), Decimal(0i128), Decimal(1i128)),
            TestCase("10".into(), Decimal(9i128), Decimal(10i128)),
            TestCase("1000".into(), Decimal(999i128), Decimal(1000i128)),
        ];

        for TestCase(price, almost_zero, not_almost_zero) in test_cases {
            assert!(
                is_almost_zero(almost_zero, price),
                "Amount {} SHOULD be almost zero with respect to a price of {}",
                almost_zero,
                price
            );
            assert!(
                !is_almost_zero(not_almost_zero, price),
                "Amount {} should NOT be almost zero with respect to a price of {}",
                not_almost_zero,
                price
            );
        }
    }
}
