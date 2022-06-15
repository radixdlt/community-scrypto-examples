//! [RealEstateMarketPlace] is the blueprint for market's host to run a decentralized real estate market place.
//! Citizens (only) can buy, sell real estate rights through this blueprint.
//! This blueprint also contain a taxing mechanism for any traded real estate.

use scrypto::prelude::*;
use crate::real_estate_service::*;

/// The NFT keep track of real estate seller's order
#[derive(NonFungibleData)]
pub struct Order {}

blueprint! {
    struct RealEstateMarketPlace {

        /// Component controller badge
        controller_badge: Vault,
        /// Building address
        building: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// Tax percent paid on real estate trade for govt authority (%)
        tax: Decimal,
        /// fee paid on real estate trade for market host (%)
        fee: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Badge to track orders on the real estate market
        order_badge: ResourceAddress,
        /// The order book of real estate market, struct: Order Id, (payment, Option(a Building NFT Id or None), Order status)
        book: HashMap<NonFungibleId, (Decimal, Option<NonFungibleId>, bool)>,
        /// The Vault contain real estate on sale
        order_vault: Vault,
        /// The Vault contain building on sale with the attached real estate
        order_contain_building: Vault,
        /// Buyer payment vault
        payment_vault: Vault,
        /// Authority's tax vault
        tax_vault: Vault,
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
        /// - tax: real estate trading tax.
        /// - land: land resource address.
        /// - building: building resource address.
        /// - medium token: the token used for trade.
        /// - real estate authority: the authority that authorized the market.
        /// Output: Component address and the market host badge
        pub fn new(name: String, controller_badge: Bucket, fee: Decimal, tax: Decimal, land: ResourceAddress, building: ResourceAddress, medium_token: ResourceAddress, real_estate_authority: ResourceAddress) -> (ComponentAddress, Bucket) {
    
            let market_host_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + "Real Estate Market Place Host Badge")
                .initial_supply(dec!(1));

            let order_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name + "Market Order Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("take_fee", rule!(require(market_host_badge.resource_address())))
                .method("take_tax", rule!(require(real_estate_authority)))
                .method("edit_fee", rule!(require(market_host_badge.resource_address())))
                .method("edit_tax", rule!(require(real_estate_authority)))
                .default(rule!(allow_all));

            let comp = Self {

                controller_badge: Vault::with_bucket(controller_badge),
                building: building,
                land: land,
                tax: tax/dec!(100),
                fee: fee/dec!(100),
                token: medium_token,
                order_badge: order_badge,
                book: HashMap::new(),
                order_vault: Vault::new(land),
                order_contain_building: Vault::new(building),
                payment_vault: Vault::new(medium_token),
                tax_vault: Vault::new(medium_token),
                fee_vault: Vault::new(medium_token),
                order_counter: 0
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, market_host_badge)
        }

        /// This method is for seller to sell a real estate right's NFTs.
        /// Input: Real estate's right NFTs:
        /// - If the land have no housing > input Enum("Land", Bucket("${land_right}"));
        /// - If the land contain a building > input Enum("LandandBuilding", Bucket("${land_right}"), Bucket("${building_right}"));
        /// Output: The NFT keep track of real estate seller's order
        pub fn new_sell_order(&mut self, real_estate: RealEstate, price: Decimal) -> Bucket {

            assert!(price>=dec!(0), "Price of the real estate must be >= 0");

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let order_id = NonFungibleId::from_u64(self.order_counter);
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let new_position = Order {};
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&order_id, new_position)
                    });
        
                    self.book.insert(order_id.clone(), (price, None, false));
        
                    self.order_vault.put(land_right);

                    info!("You have created a sell order no.{} on the {} real estate", order_id, land_data.location);

                    self.order_counter += 1;
        
                    return order_badge

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );
        
                    let order_id = NonFungibleId::from_u64(self.order_counter);
        
                    let new_position = Order {};
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&order_id, new_position)
                    });
        
                    self.book.insert(order_id.clone(), (price, Some(building_id), false));
        
                    self.order_vault.put(land_right);
        
                    self.order_contain_building.put(building_right);

                    info!("You have created a sell order no.{} on the {} real estate with an attached building", order_id, land_data.location);

                    self.order_counter += 1;
        
                    return order_badge

                }
            }    
        }


        /// This method is for buyer to buy a real estate right's NFTs.
        /// Input: The order id and payment (by medium token).
        /// Output: The real estate's NFTs and payment changes.
        pub fn buy(&mut self, order_id: u64, mut payment: Bucket) -> (RealEstate, Bucket) {

            let order_id = NonFungibleId::from_u64(order_id);

            assert!(payment.resource_address()==self.token,
                "Wrong resource."
            );

            let result = self.book.get(&order_id);

            assert!(result.is_some(),
                "The order book doesn't contain this order id"
            );

            let (price, building, status) = result.unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );
        
            let tax = price*self.tax;

            let fee = price*self.fee;
        
            assert!(
                payment.amount()>=(price + tax + fee),
                    "Not enough payment"
                );
        
            match building.clone() {
        
                None => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(tax));
                    self.fee_vault.put(payment.take(fee));
                    self.book.insert(order_id.clone(), (price, None, true));
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate", order_id, land_location);
                    return (RealEstate::Land(land_right), payment)
        
                }
        
                Some(building_id) => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(tax));
                    self.fee_vault.put(payment.take(fee));
                    self.book.insert(order_id.clone(), (price, building, true));
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let building_right = self.order_contain_building.take_non_fungible(&building_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate with the attached building", order_id, land_location);
                    return (RealEstate::LandandBuilding(land_right, building_right), payment)
        
                }
            }
        }

        /// This is method for seller to cancel an order that haven't been bought.
        /// Input: The order NFT badge.
        /// Output: The real estate right's NFTs.
        pub fn cancel_sell_order(&mut self, order_badge: Bucket) -> RealEstate {

            assert!(order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().id();

            let (_price, building, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );

            self.book.remove(&order_id);

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            let land_right = self.order_vault.take_non_fungible(&order_id);
            let land_location = land_right.non_fungible::<Land>().data().location;

            info!("You have cancel the sell order no.{} on {} real estate", order_id, land_location);

            match building.clone() {

                None => {
                    return RealEstate::Land(land_right)
                }

                Some(building_id) => {
                    return RealEstate::LandandBuilding(land_right, self.order_contain_building.take_non_fungible(&building_id))
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

            let (price, _building, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==true,
                "This real estate haven't bought."
            );

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            info!("You have taken the payment on no.{} order", order_id);

            self.payment_vault.take(price)

        }

        pub fn take_tax(&mut self) -> Bucket {
            self.tax_vault.take_all()
        }

        pub fn take_fee(&mut self) -> Bucket {
            self.fee_vault.take_all()
        }

        pub fn edit_tax(&mut self, tax: Decimal) {
            self.tax = tax
        }

        pub fn edit_fee(&mut self, fee: Decimal) {
            self.fee = fee
        }
    }
}