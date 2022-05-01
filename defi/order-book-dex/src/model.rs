use scrypto::prelude::*;

/// Represents the side of the order book that an order can be on.
/// In an example trading pair XRD/rUSD an Ask order would represent a user asking to receive rUSD as payment for their XRD.
/// Conversely, a Bid order would represent a user bidding their rUSD and expecting to receive XRD in return.
#[derive(sbor::TypeId, sbor::Encode, sbor::Decode, sbor::Describe, Clone, Copy, Debug)]
pub(crate) enum Side {
    Ask,
    Bid,
}

impl Side {
    /// Returns the opposite of this Side
    pub fn opposite(&self) -> Self {
        match self {
            Self::Ask => Self::Bid,
            Self::Bid => Self::Ask,
        }
    }
}

/// Represent a limit order in the order book.
#[derive(sbor::TypeId, sbor::Encode, sbor::Decode, sbor::Describe, NonFungibleData)]
pub(crate) struct LimitOrder {
    /// A key that uniquely identifies the order
    pub order_key: NonFungibleId,

    /// The side of the order
    pub side: Side,

    /// The amount of the quote resource that must be paid to get one unit of the base resource.
    /// For a trading pair XRD/rUSD a price of 10 would mean that 10 rUSD would have to be paid in order to buy 1 XRD.
    /// The price is always defined as stated above, irrespective of the order side!
    pub price: Decimal,

    /// The amount of the resource that a user is providing to the DEX for their order.
    /// For a trading pair XRD/rUSD and an Ask order this would be the amount of XRD, as the user is asking rUSD for the XRD they are selling.
    /// For a Bid order this would be the amount of rUSD, as the user is bidding rUSD for the XRD they are wanting to buy.
    /// Note that this is asymmetric with respect to the price!
    pub quantity: Decimal,

    /// The amount of the order quantity that has already been filled.
    #[scrypto(mutable)]
    pub quantity_filled: Decimal,
}

impl LimitOrder {
    /// Creates a new limit order
    /// Panics if price or quantity are <= 0
    pub fn new(
        order_key: NonFungibleId,
        side: Side,
        price: Decimal,
        quantity: Decimal,
    ) -> LimitOrder {
        assert!(price.is_positive(), "Parameter price must be > zero");
        assert!(quantity.is_positive(), "Parameter quantity must be > zero");
        LimitOrder {
            order_key,
            side,
            price,
            quantity,
            quantity_filled: Decimal::zero(),
        }
    }

    /// Fill the market order with the given quantity
    /// Panics if the given quantity would "overfill" the order
    pub fn fill(&mut self, quantity: Decimal) {
        assert!(
            quantity <= self.quantity - self.quantity_filled,
            "The fill quantity is too high"
        );
        self.quantity_filled += quantity;
    }

    /// Calculate the amounts of resources the user will receive upon closing the order.
    /// The first value returned is the amount the user will be refunded in case the order has not been filled fully.
    /// The second value returned is the amount that the user has successfully traded/received when the order was filled/partially filled.
    pub fn calculate_close_amounts(&self) -> (Decimal, Decimal) {
        let refund_amount = self.quantity - self.quantity_filled;
        let traded_amount = match self.side {
            Side::Ask => self.quantity_filled * self.price,
            Side::Bid => self.quantity_filled / self.price,
        };

        (refund_amount, traded_amount)
    }
}

/// Represents one of the sides of an order book.
#[derive(sbor::TypeId, sbor::Encode, sbor::Decode, sbor::Describe, Debug)]
struct OrderBookSide {
    /// The side of the order book
    side: Side,

    /// The orders on this side of the order book
    /// Keys represent the price level while values are vectors holding the keys of all orders that live on this price level.
    orders: BTreeMap<Decimal, Vec<NonFungibleId>>,
}

impl OrderBookSide {
    fn new(side: Side) -> Self {
        Self {
            side,
            orders: BTreeMap::new(),
        }
    }

    /// Retrieves the best price level that exists on this side of the order book. If no orders exist, None is returned.
    /// The returned tuple contains 1) the price level and 2) all orders at this price level.
    fn get_best_price_level(&self) -> Option<(&Decimal, &Vec<NonFungibleId>)> {
        // Implementation is likely inefficient and should be replaced with a better solution before production.
        // Once https://github.com/rust-lang/rust/issues/62924 is resolved, switch to the appropriate methods.
        match self.side {
            Side::Ask => self.orders.iter().next(),
            Side::Bid => self.orders.iter().last(),
        }
    }

    /// Returns the key of the best order that exists on this side of the order book. Returns None if no order exists.
    fn get_best_order(&self) -> Option<&NonFungibleId> {
        let (_, best_orders) = self.get_best_price_level()?;
        best_orders.first()
    }

    /// Returns the price of the best order that exists on this side of the order book. Returns None if no order exists.
    fn get_best_price(&self) -> Option<Decimal> {
        let (price, _) = self.get_best_price_level()?;
        Some(*price)
    }

    /// Inserts the given limit order into this side of the order book
    fn insert_limit_order(&mut self, order: &LimitOrder) {
        match self.orders.get_mut(&order.price) {
            Some(price_level) => price_level.push(order.order_key.clone()),
            None => {
                self.orders
                    .insert(order.price, vec![order.order_key.clone()]);
            }
        }
    }

    /// Removes the given order form this side of the order book
    fn remove_order(&mut self, to_remove: &LimitOrder) {
        let price_level = self.orders.get_mut(&to_remove.price).unwrap();
        price_level.retain(|order_key| order_key.to_vec() != to_remove.order_key.to_vec());
        if price_level.is_empty() {
            self.orders.remove(&to_remove.price);
        }
    }
}

/// Represents the order book for a trading pair
#[derive(sbor::TypeId, sbor::Encode, sbor::Decode, sbor::Describe, Debug)]
pub(crate) struct OrderBook {
    /// The side of the order book holding all Ask orders
    asks: OrderBookSide,

    /// The side of the order book holding all Bid orders
    bids: OrderBookSide,
}

impl OrderBook {
    /// Creates a new empty order book
    pub fn new() -> Self {
        Self {
            asks: OrderBookSide::new(Side::Ask),
            bids: OrderBookSide::new(Side::Bid),
        }
    }

    /// Returns the key of the best order in the order book for the given side. Returns None if no orders exists
    /// on that side of the order book.
    pub fn get_best_order(&self, side: Side) -> Option<&NonFungibleId> {
        let side = match side {
            Side::Ask => &self.asks,
            Side::Bid => &self.bids,
        };
        side.get_best_order()
    }

    /// Inserts a limit order into the given side of the order book.
    /// Panics if the order is priced such that it would constitute a market order.
    pub fn insert_limit_order(&mut self, order: &LimitOrder) {
        match order.side {
            Side::Ask => {
                let best_bid_price = self.bids.get_best_price();
                assert!(
                    best_bid_price.is_none() || order.price > best_bid_price.unwrap(),
                    "Order would be a market order"
                );
                self.asks.insert_limit_order(order);
            }
            Side::Bid => {
                let best_ask_price = self.asks.get_best_price();
                assert!(
                    best_ask_price.is_none() || order.price < best_ask_price.unwrap(),
                    "Order would be a market order"
                );
                self.bids.insert_limit_order(order)
            }
        };
    }

    /// Removes the given limit order from the order book.
    pub fn remove_limit_order(&mut self, order: &LimitOrder) {
        let side = match order.side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids,
        };
        side.remove_order(order);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "Parameter price must be > zero")]
    fn test_limit_order_constructor_panics_on_zero_price() {
        LimitOrder::new(
            NonFungibleId::from_str("1234").unwrap(),
            Side::Ask,
            0.into(),
            1.into(),
        );
    }

    #[test]
    #[should_panic(expected = "Parameter price must be > zero")]
    fn test_limit_order_constructor_panics_on_negative_price() {
        LimitOrder::new(
            NonFungibleId::from_str("1234").unwrap(),
            Side::Ask,
            (-1).into(),
            1.into(),
        );
    }

    #[test]
    #[should_panic(expected = "Parameter quantity must be > zero")]
    fn test_limit_order_constructor_panics_on_zero_quantity() {
        LimitOrder::new(
            NonFungibleId::from_str("1234").unwrap(),
            Side::Ask,
            1.into(),
            0.into(),
        );
    }

    #[test]
    #[should_panic(expected = "Parameter quantity must be > zero")]
    fn test_limit_order_constructor_panics_on_negative_quantity() {
        LimitOrder::new(
            NonFungibleId::from_str("1234").unwrap(),
            Side::Ask,
            1.into(),
            (-1).into(),
        );
    }
}
