//! [RealEstateMarketPlace] is the blueprint for market's host to run a decentralized real estate market place.
//! Citizens (only) can buy, sell real estate rights through this blueprint.
//! This blueprint also contain a taxing mechanism for any traded real estate.

use scrypto::prelude::*;
use crate::real_estate_service::*;
use crate::utility::*;

/// The NFT keep track of real estate seller's order
#[derive(NonFungibleData)]
pub struct Order {}

blueprint! {
    struct RealEstateMarketPlace {

        /// Component controller badge
        controller_badge: Vault,
        /// Authority component address
        authority_address: ComponentAddress,
        /// Resource move badge
        move_badge: ResourceAddress,
        /// Building address
        building: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// fee paid on real estate trade for market host (%)
        fee: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Badge to track orders on the real estate market
        order_badge: ResourceAddress,
        /// The order book of real estate market, struct: Order Id, (payment, land id, Option(a Building NFT Id or None), Order status)
        book: HashMap<NonFungibleId, (Decimal, NonFungibleId, Option<NonFungibleId>, bool)>,
        /// The Vault contain real estate on sale
        order_vault: Vault,
        /// The Vault contain building on sale with the attached real estate
        order_contain_building: Vault,
        /// Buyer payment vault
        payment_vault: Vault,
        /// Market host's fee vault
        fee_vault: Vault,
        /// Order counter
        order_counter: u64

    }

    impl RealEstateMarketPlace {
        
        /// This function will create new Real Estate Market Place component
        /// Input: 
        /// - name: market name.
        /// - controller badge: the market component controller badge.
        /// - fee: market fee.
        /// - land: land resource address.
        /// - building: building resource address.
        /// - medium token: the token used for trade.
        /// - real estate authority: the authority that authorized the market.
        /// Output: Component address and the market host badge
        pub fn new(market_host_badge: NonFungibleAddress, id_badge: ResourceAddress, authority_address: ComponentAddress, name: String, controller_badge: Bucket, fee: Decimal, land: ResourceAddress, building: ResourceAddress, medium_token: ResourceAddress, move_badge: ResourceAddress) -> ComponentAddress {

            let order_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name + " Market Order Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .restrict_withdraw(rule!(require(id_badge)), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("take_fee", rule!(require(market_host_badge.clone())))
                .method("edit_fee", rule!(require(market_host_badge)))
                .default(rule!(require(id_badge)));

            let comp = Self {

                controller_badge: Vault::with_bucket(controller_badge),
                authority_address: authority_address,
                move_badge: move_badge,
                building: building,
                land: land,
                fee: fee/dec!(100),
                token: medium_token,
                order_badge: order_badge,
                book: HashMap::new(),
                order_vault: Vault::new(land),
                order_contain_building: Vault::new(building),
                payment_vault: Vault::new(medium_token),
                fee_vault: Vault::new(medium_token),
                order_counter: 0
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return comp
        }

        /// This method is for seller to sell a real estate right's NFTs.
        /// Input: Real estate's right NFTs:
        /// - If the land have no housing > input Enum("Land", Bucket("${land_right}"));
        /// - If the land contain a building > input Enum("LandandBuilding", Bucket("${land_right}"), Bucket("${building_right}"));
        /// Output: The NFT keep track of real estate seller's order
        pub fn new_sell_order(&mut self, real_estate: RealEstate, price: Decimal) -> (Bucket, Proof) {

            assert!(price>=dec!(0), "Price of the real estate must be >= 0");

            match real_estate {

                RealEstate::Land(land_right) => {

                    let (land_id, land_data) = assert_land_proof(land_right.create_proof(), self.land);

                    let order_id = NonFungibleId::from_u64(self.order_counter);
        
                    let new_position = Order {};
        
                    self.book.insert(order_id.clone(), (price, land_id, None, false));
                
                    let (order_badge, move_proof) = self.controller_badge.authorize(|| {

                        let move_badge = borrow_resource_manager!(self.move_badge)
                            .mint(dec!(1));

                        move_badge.authorize(|| {self.order_vault.put(land_right)});

                        let move_proof = move_badge.create_proof();

                        borrow_resource_manager!(self.move_badge)
                            .burn(move_badge);

                        (borrow_resource_manager!(self.order_badge)
                        .mint_non_fungible(&order_id, new_position), move_proof)

                    });

                    info!("You have created a sell order no.{} on the {} real estate", order_id, land_data.location);

                    self.order_counter += 1;
        
                    return (order_badge, move_proof)

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    let (land_id, land_data, building_id, _) = assert_landandbuilding_proof(land_right.create_proof(), building_right.create_proof(), self.land, self.building);

                    let order_id = NonFungibleId::from_u64(self.order_counter);
        
                    let new_position = Order {};
        
                    self.book.insert(order_id.clone(), (price, land_id, Some(building_id), false));

                    let (order_badge, move_proof) = self.controller_badge.authorize(|| {

                        let move_badge = borrow_resource_manager!(self.move_badge)
                            .mint(dec!(1));

                        move_badge.authorize(|| {self.order_vault.put(land_right); self.order_contain_building.put(building_right)});

                        let move_proof = move_badge.create_proof();
                        
                        borrow_resource_manager!(self.move_badge)
                            .burn(move_badge);

                        (borrow_resource_manager!(self.order_badge)
                        .mint_non_fungible(&order_id, new_position), move_proof)

                    });

                    info!("You have created a sell order no.{} on the {} real estate with an attached building", order_id, land_data.location);

                    self.order_counter += 1;
        
                    return (order_badge, move_proof)

                }
            }    
        }

        /// This method is for buyer to buy a real estate right's NFTs.
        /// Input: The order id and payment (by medium token).
        /// Output: The real estate's NFTs and payment changes.
        pub fn buy(&mut self, order_id: u64, mut payment: Bucket) -> (RealEstate, Bucket, Proof) {

            let order_id = NonFungibleId::from_u64(order_id);

            assert!(payment.resource_address()==self.token,
                "Wrong resource."
            );

            let result = self.book.get(&order_id);

            assert!(result.is_some(),
                "The order book doesn't contain this order id"
            );

            let (price, land_id, building_id, status) = result.unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );

            let authority: RealEstateService = self.authority_address.into();
        
            let tax = price*authority.tax();

            let fee = price*self.fee;

            let total = price + tax + fee;
        
            assert!(
                payment.amount()>=total,
                    "Not enough payment"
                );

            let move_proof = self.controller_badge.authorize(|| {
                let move_badge = borrow_resource_manager!(self.move_badge)
                    .mint(dec!(1));
                let move_proof = move_badge.create_proof();
                borrow_resource_manager!(self.move_badge)
                    .burn(move_badge);
                return move_proof
                });
        
            match building_id {
        
                None => {
        
                    self.payment_vault.put(payment.take(price));
                    authority.deposit_tax(payment.take(tax));
                    self.fee_vault.put(payment.take(fee));
                    self.book.insert(order_id.clone(), (price, land_id.clone(), None, true));
                    let land_right = self.order_vault.take_non_fungible(&land_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have paid {} tokens to fill the order no.{} and bought the {} real estate (included all tax, fee)", total, order_id, land_location);
                    return (RealEstate::Land(land_right), payment, move_proof)
        
                }
        
                Some(building_id) => {
        
                    self.payment_vault.put(payment.take(price));
                    authority.deposit_tax(payment.take(tax));
                    self.fee_vault.put(payment.take(fee));
                    self.book.insert(order_id.clone(), (price, land_id.clone(), Some(building_id.clone()), true));
                    let land_right = self.order_vault.take_non_fungible(&land_id);
                    let building_right = self.order_contain_building.take_non_fungible(&building_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have paid {} tokens to fill the order no.{} and bought the {} real estate with the attached building (included all tax, fee)", total, order_id, land_location);
                    return (RealEstate::LandandBuilding(land_right, building_right), payment, move_proof)
        
                }
            }
        }

        /// This is method for seller to cancel an order that haven't been bought.
        /// Input: The order NFT badge.
        /// Output: The real estate right's NFTs.
        pub fn cancel_sell_order(&mut self, order_badge: Bucket) -> (RealEstate, Proof) {

            assert!(order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().id();

            let (_, land_id, building_id, status) = self.book.remove(&order_id).unwrap();

            assert!(status==false,
                "This real estate is already bought."
            );

            let land_right = self.order_vault.take_non_fungible(&land_id);
            let land_location = land_right.non_fungible::<Land>().data().location;

            info!("You have cancel the sell order no.{} on {} real estate", order_id, land_location);

            let move_proof = self.controller_badge.authorize(|| {
                let move_badge = borrow_resource_manager!(self.move_badge)
                    .mint(dec!(1));
                let move_proof = move_badge.create_proof();
                borrow_resource_manager!(self.move_badge)
                    .burn(move_badge);
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge);
                return move_proof
                });

            match building_id {

                None => {
                    return (RealEstate::Land(land_right), move_proof)
                }

                Some(building_id) => {
                    return (RealEstate::LandandBuilding(land_right, self.order_contain_building.take_non_fungible(&building_id)), move_proof)
                }
            }
        }

        /// This is method for seller to take the payment.
        /// Input: The order NFT badge.
        /// Output: The real estate right's NFTs.
        pub fn take_payment(&mut self, order_badge: Bucket) -> Bucket {

            assert!(
                order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().id();

            let (price, _, _, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==true,
                "This real estate haven't bought."
            );

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            info!("You have taken the payment of {} tokens on order no.{}", price, order_id);

            self.payment_vault.take(price)

        }

        pub fn take_fee(&mut self) -> Bucket {

            info!("You have collected {} tokens market place fee.", self.fee_vault.amount());
            self.fee_vault.take_all()

        }

        pub fn edit_fee(&mut self, fee: Decimal) {

            info!("You have edited fee rate of the market place into {} % per trade", fee);
            self.fee = fee/dec!(100);

        }
    }
}