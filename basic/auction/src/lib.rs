use sbor::*;
use scrypto::prelude::*;

const PAYMENT_DEADLINE: u64 = 100;

#[derive(TypeId, Encode, Decode, Describe)]
struct Bidder {
    bid: Decimal,
    bid_bond_reclaimed: bool,
}

blueprint! {
    struct Auction {
        offering: Vault,
        bid_bonds: Vault,
        payment: Vault,

        start: u64,
        duration: u64,
        payment_resource: ResourceAddress,
        reserve_price: Decimal,
        bid_bond: Decimal,
        
        bidders: LazyMap<ResourceAddress, Bidder>,
        highest_bid: Decimal,
        
        auctioneer_badge: ResourceAddress,
        payment_claimed: bool,
    }

    impl Auction {
        pub fn new(
            offering: Bucket, 
            duration: u64, 
            payment_resource: ResourceAddress, 
            reserve_price: Decimal, 
            bid_bond: Decimal
        ) -> (ComponentAddress, Bucket) {
            assert!(offering.amount() > Decimal::zero(), "Incorrect offering");
            
            assert!(bid_bond <= reserve_price, "Bid bond higher than the reserve price");

            // mint the auctioneer badge
            let auctioneer_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Acutioneer badge")
                .initial_supply(1);

            // Setting the access rules for the component
            let access_rules = AccessRules::new()
                .method("claim_payment", rule!(require(auctioneer_badge.resource_address())))
                .default(rule!(allow_all));

            // instantiate an auction component
            let auction = Self {
                offering: Vault::with_bucket(offering),
                bid_bonds: Vault::new(payment_resource.clone()),
                payment: Vault::new(payment_resource.clone()),

                start: Runtime::current_epoch(),
                duration,
                payment_resource: ResourceAddress::from(payment_resource),
                reserve_price,
                bid_bond,

                bidders: LazyMap::new(),
                highest_bid: Decimal::zero(),

                auctioneer_badge: ResourceAddress([0; 26]),
                payment_claimed: false,
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (auction, auctioneer_badge)
        }

        pub fn register(&mut self, bid_bond: Bucket) -> Bucket {
            // check if the auction is open
            assert!(Runtime::current_epoch() <= self.start + self.duration, "Auction closed");

            // check the bid bond
            assert!(bid_bond.resource_address() == self.payment_resource, "Incorrect payment token");
            assert!(bid_bond.amount() == self.bid_bond, "Incorrect bid bond");

            // take the bid bond
            self.bid_bonds.put(bid_bond);

            // mint a bidder badge
            let bidder_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Bidder badge")
                .initial_supply(1);

            // save the bidder using the bidder badge resource definition
            self.bidders.insert(
                bidder_badge.resource_address(), 
                Bidder { 
                    bid: Decimal::zero(), 
                    bid_bond_reclaimed: false 
                }
            );

            bidder_badge
        }

        pub fn bid(&mut self, bid: Decimal, bidder_badge: Proof) {
            // check if the auction is open
            assert!(Runtime::current_epoch() <= self.start + self.duration, "Auction closed");

            // check the bidder badge and get the bidder
            let bidder_id = bidder_badge.resource_address();
            let mut bidder = self.get_bidder(bidder_badge);

            // check the bid
            assert!(bid >= self.reserve_price, "Bid lower than the reserve price");
            assert!(bid > self.highest_bid, "Bid not higer than the current highest bid");

            // save the bid
            bidder.bid = bid;
            self.bidders.insert(bidder_id, bidder);
            self.highest_bid = bid;
        }

        pub fn claim_offering(&mut self, payment: Bucket, bidder_badge: Proof) -> Bucket {
            // check if the auction is closed
            assert!(Runtime::current_epoch() > self.start + self.duration, "Auction open");

            // check if the payment deadline has not passed yet
            assert!(Runtime::current_epoch() <= self.start + self.duration + PAYMENT_DEADLINE, "Payment deadline passed");

            // check the bidder badge and get the bidder
            let bidder = self.get_bidder(bidder_badge);

            // check if it is the winning bidder
            assert!(bidder.bid > Decimal::zero() && bidder.bid == self.highest_bid, "Not the winning bidder");

            // check if the offering hasn't yet been claimed
            assert!(!self.offering.is_empty(), "Offering already claimed");

            // check the payment
            assert!(payment.resource_address() == self.payment_resource, "Incorrect payment token");
            assert!(payment.amount() == self.highest_bid - self.bid_bond, "Incorrect payment amount");

            // take the payment and return the offering
            self.payment.put(payment);

            self.offering.take_all()
        }

        pub fn reclaim_bid_bond(&mut self, bidder_badge: Proof) -> Bucket {
            // check if auction is closed
            assert!(Runtime::current_epoch() > self.start + self.duration, "Acution open");

            // check bidder badge and get the bidder
            let bidder_id = bidder_badge.resource_address();
            let mut bidder = self.get_bidder(bidder_badge);

            // check if it is not the winning bid
            assert!(bidder.bid == Decimal::zero() || bidder.bid != self.highest_bid, "Winning bidder cannot reclaim the bid bond");

            // check if the bidder has not yet reclaimed the bid bond
            assert!(!bidder.bid_bond_reclaimed, "Bid bond already reclaimed");

            // save that the bidder reclaimed the bid bond
            bidder.bid_bond_reclaimed = true;
            self.bidders.insert(bidder_id, bidder);

            self.bid_bonds.take(self.bid_bond)
        }
        
        pub fn claim_payment(&mut self) -> (Bucket, Bucket) {
            // check if auction is closed
            assert!(Runtime::current_epoch() > self.start + self.duration, "Auction open");

            // check if the payment has not been yet claimed
            assert!(!self.payment_claimed, "Payment already claimed");

            // check if there were no bids or the payment has been received or the payment deadline has passed
            assert!(
                self.highest_bid == Decimal::zero() ||
                !self.payment.is_empty() || 
                Runtime::current_epoch() > self.start + self.duration + PAYMENT_DEADLINE,
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

        fn get_bidder(&self, bidder_badge: Proof) -> Bidder {
            assert!(bidder_badge.amount() > Decimal::zero(), "No bidder badge presented");
            let bidder = self.bidders.get(&bidder_badge.resource_address());
            assert!(bidder.is_some(), "Incorrect bidder badge");
            bidder_badge.drop();

            bidder.unwrap()
        }
    }
}

