use sbor::*;
use scrypto::prelude::*;

const PAYMENT_DEADLINE: u64 = 100;

#[derive(TypeId, Encode, Decode)]
struct Bidder {
    bid: Decimal,
    bid_bond_reclaimed: bool,
}

blueprint! {
    struct Auction {
        component_address: Address,

        offering: Vault,
        bid_bonds: Vault,
        payment: Vault,

        start: u64,
        duration: u64,
        payment_resource: ResourceDef,
        reserve_price: Decimal,
        bid_bond: Decimal,
        
        bidders: LazyMap<ResourceDef, Bidder>,
        highest_bid: Decimal,
        
        auctioneer_badge: ResourceDef,
        payment_claimed: bool,
    }

    impl Auction {
        pub fn new(offering: Bucket, duration: u64, payment_resource: Address, reserve_price: Decimal, bid_bond: Decimal) -> (Component, Bucket) {
            scrypto_assert!(offering.amount() > Decimal::zero(), "Incorrect offering");
            
            scrypto_assert!(bid_bond <= reserve_price, "Bid bond higher than the reserve price");

            // instantiate an auction component
            let auction = Self {
                component_address: Address::Component([0; 26]),
                
                offering: Vault::with_bucket(offering),
                bid_bonds: Vault::new(payment_resource.clone()),
                payment: Vault::new(payment_resource.clone()),

                start: Context::current_epoch(),
                duration,
                payment_resource: ResourceDef::from(payment_resource),
                reserve_price,
                bid_bond,

                bidders: LazyMap::new(),
                highest_bid: Decimal::zero(),

                auctioneer_badge: ResourceDef::from(Address::ResourceDef([0; 26])),
                payment_claimed: false,
            }
            .instantiate();

            // save the component address (it doesn't seem like there is a way to get it later from iside a method)
            let mut auction_state = auction.get_state::<Self>();
            auction_state.component_address = auction.address();

            // mint the auctioneer badge
            let auctioneer_badge = ResourceBuilder::new()
                .metadata("name", "Acutioneer badge")
                .metadata("auction", auction_state.component_address.to_string())
                .new_badge_fixed(1);

            // save the auctioner badge resource definition
            auction_state.auctioneer_badge = auctioneer_badge.resource_def();
            auction.put_state(auction_state);

            (auction, auctioneer_badge)
        }

        pub fn register(&mut self, bid_bond: Bucket) -> Bucket {
            // check if the auction is open
            scrypto_assert!(Context::current_epoch() <= self.start + self.duration, "Auction closed");

            // check the bid bond
            scrypto_assert!(bid_bond.resource_def() == self.payment_resource, "Incorrect payment token");
            scrypto_assert!(bid_bond.amount() == self.bid_bond, "Incorrect bid bond");

            // take the bid bond
            self.bid_bonds.put(bid_bond);

            // mint a bidder badge
            let bidder_badge = ResourceBuilder::new()
                .metadata("name", "Bidder badge")
                .metadata("auction", self.component_address.to_string())
                .new_badge_fixed(1);

            // save the bidder using the bidder badge resource definition
            self.bidders.insert(bidder_badge.resource_def(), Bidder { bid: Decimal::zero(), bid_bond_reclaimed: false });

            bidder_badge
        }

        pub fn bid(&mut self, bid: Decimal, bidder_badge: BucketRef) {
            // check if the auction is open
            scrypto_assert!(Context::current_epoch() <= self.start + self.duration, "Auction closed");

            // check the bidder badge and get the bidder
            let bidder_id = bidder_badge.resource_def();
            let mut bidder = self.get_bidder(bidder_badge);

            // check the bid
            scrypto_assert!(bid >= self.reserve_price, "Bid lower than the reserve price");
            scrypto_assert!(bid > self.highest_bid, "Bid not higer than the current highest bid");

            // save the bid
            bidder.bid = bid;
            self.bidders.insert(bidder_id, bidder);
            self.highest_bid = bid;
        }

        pub fn claim_offering(&mut self, payment: Bucket, bidder_badge: BucketRef) -> Bucket {
            // check if the auction is closed
            scrypto_assert!(Context::current_epoch() > self.start + self.duration, "Auction open");

            // check if the payment deadline has not passed yet
            scrypto_assert!(Context::current_epoch() <= self.start + self.duration + PAYMENT_DEADLINE, "Payment deadline passed");

            // check the bidder badge and get the bidder
            let bidder = self.get_bidder(bidder_badge);

            // check if it is the winning bidder
            scrypto_assert!(bidder.bid > Decimal::zero() && bidder.bid == self.highest_bid, "Not the winning bidder");

            // check if the offering hasn't yet been claimed
            scrypto_assert!(!self.offering.is_empty(), "Offering already claimed");

            // check the payment
            scrypto_assert!(payment.resource_def() == self.payment_resource, "Incorrect payment token");
            scrypto_assert!(payment.amount() == self.highest_bid - self.bid_bond, "Incorrect payment amount");

            // take the payment and return the offering
            self.payment.put(payment);

            self.offering.take_all()
        }

        pub fn reclaim_bid_bond(&mut self, bidder_badge: BucketRef) -> Bucket {
            // check if auction is closed
            scrypto_assert!(Context::current_epoch() > self.start + self.duration, "Acution open");

            // check bidder badge and get the bidder
            let bidder_id = bidder_badge.resource_def();
            let mut bidder = self.get_bidder(bidder_badge);

            // check if it is not the winning bid
            scrypto_assert!(bidder.bid == Decimal::zero() || bidder.bid != self.highest_bid, "Winning bidder cannot reclaim the bid bond");

            // check if the bidder has not yet reclaimed the bid bond
            scrypto_assert!(!bidder.bid_bond_reclaimed, "Bid bond already reclaimed");

            // save that the bidder reclaimed the bid bond
            bidder.bid_bond_reclaimed = true;
            self.bidders.insert(bidder_id, bidder);

            self.bid_bonds.take(self.bid_bond)
        }
        
        #[auth(auctioneer_badge)]
        pub fn claim_payment(&mut self) -> (Bucket, Bucket) {
            // check if auction is closed
            scrypto_assert!(Context::current_epoch() > self.start + self.duration, "Auction open");

            // check if the payment has not been yet claimed
            scrypto_assert!(!self.payment_claimed, "Payment already claimed");

            // check if there were no bids or the payment has been received or the payment deadline has passed
            scrypto_assert!(
                self.highest_bid == Decimal::zero() ||
                !self.payment.is_empty() || 
                Context::current_epoch() > self.start + self.duration + PAYMENT_DEADLINE,
                "Payment not received and the payment deadline not passed");

            // if the offering has been sold then add the winner's bid bond to the payment
            if self.highest_bid > Decimal::zero() {
                self.payment.put(self.bid_bonds.take(self.bid_bond));
            }

            // save that the auctionner has claimed the payment
            self.payment_claimed = true;

            // take the payment and the offering (in case there there were no bidders or the payment has not been received)
            (self.payment.take_all(), self.offering.take_all())            
        }

        fn get_bidder(&self, bidder_badge: BucketRef) -> Bidder {
            scrypto_assert!(bidder_badge.amount() > Decimal::zero(), "No bidder badge presented");
            let bidder = self.bidders.get(&bidder_badge.resource_def());
            scrypto_assert!(bidder.is_some(), "Incorrect bidder badge");
            bidder_badge.drop();

            bidder.unwrap()
        }
    }
}

