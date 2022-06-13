//! [RealEstate] is the blueprint for autorities to manage real estate rights by NFT, also a real estate market place for citizen.
//! In other words, people can buy, sell real estate rights through this blueprint.
//! This blueprint also contain a taxing mechanism for any traded real estate.

use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstate {
    Land(Bucket),
    LandandHouse(Bucket, Bucket),
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstateData {
    Land(Decimal, String),
    LandandHouse(Decimal, String, Decimal, u32),
}

#[derive(NonFungibleData, TypeId, Encode, Decode, Describe, Copy, Clone)]
pub struct House {
    #[scrypto(mutable)]
    size: Decimal,
    #[scrypto(mutable)]
    floor: u32,
}

#[derive(NonFungibleData)]
pub struct Land {
    size: Decimal,
    location: String,
    #[scrypto(mutable)]
    contain: Option<(NonFungibleId, House)>
}

#[derive(NonFungibleData)]
pub struct Order {
    order_id: NonFungibleId
}

blueprint! {
    struct RealEstateManager {
        /// Component controller badge
        controller_badge: Vault,
        /// House address
        house: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// Tax (%)
        tax: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Badge to track orders on the real estate market
        order_badge: ResourceAddress,
        /// The order book of real estate market
        book: HashMap<NonFungibleId, (Decimal, Option<NonFungibleId>, bool)>,
        order_vault: Vault,
        order_contain_house: Vault,
        payment_vault: Vault,
        tax_vault: Vault
    }

    impl RealEstateManager {
        
        /// Input: tax percent and the medium token used for payment on market
        /// Output: Component address and the authority badge
        pub fn new(tax: Decimal, medium_token: ResourceAddress) -> (ComponentAddress, Bucket) {
           
            let controller_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "Mint Badge")
            .initial_supply(dec!(1));

            let land = ResourceBuilder::new_non_fungible()
                .metadata("name", "Land")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let house = ResourceBuilder::new_non_fungible()
                .metadata("name", "House")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let real_estate_authority = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Authority Badge")
                .initial_supply(dec!(1));

            let order_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Position Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("new_land", rule!(require(real_estate_authority.resource_address())))
                .method("add_house_to_land", rule!(require(real_estate_authority.resource_address())))
                .method("modify_house_info", rule!(require(real_estate_authority.resource_address())))
                .method("take_tax", rule!(require(real_estate_authority.resource_address())))
                .default(rule!(allow_all));

            let comp = Self {
                controller_badge: Vault::with_bucket(controller_badge),
                house: house,
                land: land,
                tax: tax/dec!(100),
                token: medium_token,
                order_badge: order_badge,
                book: HashMap::new(),
                order_vault: Vault::new(land),
                order_contain_house: Vault::new(house),
                payment_vault: Vault::new(medium_token),
                tax_vault: Vault::new(medium_token)
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, real_estate_authority)
        }

        pub fn new_land(&self, data: RealEstateData) -> RealEstate {

            match data {

                RealEstateData::Land(land_size, location) => {

                    assert!(land_size > dec!(0),
                        "Must provide the right land size"
                    );

                    let new_land = Land {
                        contain: None,
                        size: land_size,
                        location: location
                    };

                    let land_id: NonFungibleId = NonFungibleId::random();

                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });

                    RealEstate::Land(land_right)

                }

                RealEstateData::LandandHouse(land_size, location, house_size, house_floor) => {

                    assert!(land_size > dec!(0),
                        "Must provide the right land size."
                    );
        
                    assert!(
                        (house_size > dec!(0)) & (house_floor > 0),
                        "Must provide the right house info."
                    );
        
                    let new_house = House {
                        size: house_size,
                        floor: house_floor
                    };
        
                    let house_id: NonFungibleId = NonFungibleId::random();
        
                    let house_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.house)
                            .mint_non_fungible(&house_id, new_house.clone())
                    });
        
                    let new_land = Land {
                        contain: Some((house_id, new_house)),
                        size: land_size,
                        location: location
                    };
        
                    let land_id: NonFungibleId = NonFungibleId::random();
        
                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });
        
                    RealEstate::LandandHouse(land_right, house_right)
                }
            }
        }

        pub fn add_house_to_land(&self, land_right: Bucket, house_size: Decimal, house_floor: u32) -> (Bucket, Bucket) {

            assert!(land_right.resource_address()==self.land,
                "Wrong resource."
            );

            let new_house = House {
                size: house_size,
                floor: house_floor
            };

            let house_id: NonFungibleId = NonFungibleId::random();

            let house_right = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.house)
                    .mint_non_fungible(&house_id, new_house.clone())
            });

            let mut data: Land = land_right.non_fungible().data();

            assert!(data.contain.is_none(),
                "This land already has a house."
            );

            data.contain = Some((house_id, new_house));

            self.controller_badge
                .authorize(|| land_right.non_fungible().update_data(data));

            return (land_right, house_right)

        }

        pub fn modify_house_info(&self, land_right: Bucket, house_right: Bucket, house_size: Decimal, house_floor: u32) -> (Bucket, Bucket) {

            assert!((land_right.resource_address()==self.land) & (house_right.resource_address() == self.house),
                "Wrong resource."
            );

            let mut house_data: House = house_right.non_fungible().data();

            let house_id = house_right.non_fungible::<House>().id();

            let mut land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.unwrap().0 == house_id,
                "This land doesn't contain the house from provided house right."
            );

            house_data.size = house_size;
            house_data.floor = house_floor;
            land_data.contain = Some((house_id, house_data));

            self.controller_badge
                .authorize(|| {
                    land_right.non_fungible().update_data(land_data);
                    house_right.non_fungible().update_data(house_data)
                });
            
            return (land_right, house_right)

        }

        /// This is method to sell a real estate without house.
        pub fn new_sell_order(&mut self, real_estate: RealEstate, price: Decimal) -> Bucket {

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_id = land_right.non_fungible::<Land>().id();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a house."
                    );
        
                    let new_position = Order {
                        order_id: land_id.clone()
                    };
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&NonFungibleId::random(), new_position)
                    });
        
                    self.book.insert(land_id, (price, None, false));
        
                    self.order_vault.put(land_right);
        
                    return order_badge
                }

                RealEstate::LandandHouse(land_right, house_right) => {

                    assert!((land_right.resource_address()==self.land) & (house_right.resource_address() == self.house),
                        "Wrong resource."
                    );
        
                    let house_id = house_right.non_fungible::<House>().id();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == house_id,
                        "This land doesn't contain the house from provided house right."
                    );
        
                    let land_id = land_right.non_fungible::<Land>().id();
        
                    let new_position = Order {
                        order_id: land_id.clone()
                    };
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&NonFungibleId::random(), new_position)
                    });
        
                    self.book.insert(land_id, (price, Some(house_id), false));
        
                    self.order_vault.put(land_right);
        
                    self.order_contain_house.put(house_right);
        
                    return order_badge

                }
            }    
        }


        /// This is method to buy a real estate without house.
        pub fn buy(&mut self, order_id: u64, mut payment: Bucket) -> (RealEstate, Bucket) {

            let order_id = NonFungibleId::from_u64(order_id);

            assert!(payment.resource_address()==self.token,
                "Wrong resource."
            );

            let result = self.book.get(&order_id);

            assert!(result.is_some(),
                "The order book doesn't contain this order id"
            );

            let (price, house, status) = result.unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );
        
            let tax = price*self.tax;
        
            assert!(
                payment.amount()>=(price + tax),
                    "Not enough payment"
                );
        
            match house.clone() {
        
                None => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(price*self.tax));
                    self.book.insert(order_id.clone(), (price, house, true));
                    return (RealEstate::Land(self.order_vault.take_non_fungible(&order_id)), payment)
        
                }
        
                Some(house_id) => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(price*self.tax));
                    self.book.insert(order_id.clone(), (price, house, true));
                    return (RealEstate::LandandHouse(self.order_vault.take_non_fungible(&order_id), self.order_contain_house.take_non_fungible(&house_id)), payment)
        
                }
              
            }
        }

        pub fn cancel_sell_order(&mut self, order_badge: Bucket) -> RealEstate {

            assert!(order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (_price, house, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );

            self.book.remove(&order_id);

            match house.clone() {

                None => {
                    return RealEstate::Land(self.order_vault.take_non_fungible(&order_id))
                }

                Some(house_id) => {
                    return RealEstate::LandandHouse(self.order_vault.take_non_fungible(&order_id), self.order_contain_house.take_non_fungible(&house_id))
                }

            }
        }

        pub fn take_payment(&mut self, order_badge: Bucket) -> Bucket {

            assert!(
                order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (price, _house, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==true,
                "This real estate haven't bought."
            );

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            self.payment_vault.take(price)

        }

        pub fn take_tax(&mut self) -> Bucket {
            self.tax_vault.take_all()
        }
    }
}