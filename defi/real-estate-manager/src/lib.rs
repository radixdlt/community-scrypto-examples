//! [RealEstate] is the blueprint for autorities to manage real estate rights by NFT, also a real estate market place for citizen.
//! In other words, people can buy, sell real estate rights through this blueprint.
//! This blueprint also contain a taxing mechanism for any traded real estate.

use scrypto::prelude::*;

/// The NFTs of real estates, can contain both land right's NFT and house right's NFT (if that land contain a house)
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstate {
    Land(Bucket),
    LandandHouse(Bucket, Bucket),
}

/// The data of real estates, can contain both land data and house data (if that land contain a house)
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstateData {
    Land(Decimal, String),
    LandandHouse(Decimal, String, Decimal, u32),
}

/// The house right's NFT
/// All the house data (size, floor) is mutable through construction.
/// size (m2)
#[derive(NonFungibleData, TypeId, Encode, Decode, Describe, Copy, Clone)]
pub struct House {
    #[scrypto(mutable)]
    size: Decimal,
    #[scrypto(mutable)]
    floor: u32,
}

/// The land right's NFT
/// The land data (size, location) is immutable, but the house that land contain is mutable through construction.
/// size (m2)
#[derive(NonFungibleData)]
pub struct Land {
    size: Decimal,
    location: String,
    #[scrypto(mutable)]
    contain: Option<(NonFungibleId, House)>
}

/// The NFT keep track of real estate seller's order
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
        /// The Vault contain real estate on sale
        order_vault: Vault,
        /// The Vault contain house on sale with the attached real estate
        order_contain_house: Vault,
        /// Buyer payment vault
        payment_vault: Vault,
        /// Authority's tax vault
        tax_vault: Vault
    }

    impl RealEstateManager {
        
        /// This function will create new Real Estate Manager component
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

        /// This method is for authority to create and distribute new real estate right's NFTs with the input data
        /// Input: Land data:
        /// - If the land have no housing > input Enum("Land", Decimal("${land_size}"), "${location}");
        /// - If the land contain a house > input Enum("LandandHouse", Decimal("${land_size}"), "${location}", Decimal("${house_size}"), ${house_floor}u32);
        /// Output: 
        /// - If the land have no housing > The land right's NFT;
        /// - If the land contain a house > The land right's NFT and the house right's NFT.
        pub fn new_land(&self, data: RealEstateData) -> RealEstate {

            match data {

                RealEstateData::Land(land_size, location) => {

                    assert!(land_size > dec!(0),
                        "Must provide the right land size"
                    );

                    let new_land = Land {
                        contain: None,
                        size: land_size,
                        location: location.clone()
                    };

                    let land_id: NonFungibleId = NonFungibleId::random();

                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size, location);

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
                        location: location.clone()
                    };
        
                    let land_id: NonFungibleId = NonFungibleId::random();
        
                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {} with a house right's NFT of the {}m2, {} floor house", land_size, location, house_size, house_floor);
        
                    RealEstate::LandandHouse(land_right, house_right)
                }
            }
        }

        /// This method is for authority to add new house to an existed land.
        /// Input: The land right's NFT proof and house's data.
        /// Output: The house right's NFT of that land.
        pub fn add_house_to_land(&self, land_right: Proof, house_size: Decimal, house_floor: u32) -> Bucket {

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

            let location = data.location.clone();

            self.controller_badge
                .authorize(|| land_right.non_fungible().update_data(data));

            info!("You have added a new house right's NFT of the {}m2, {} floor house attached to the land on {} according to construction data", house_size, house_floor, location);

            return house_right

        }

        /// This method is for authority to modify existed house info.
        /// Input: The land right's NFT proof, the house right's NFT proof of that land and the house's new data.
        /// Output: None.
        pub fn modify_house_info(&self, land_right: Proof, house_right: Proof, house_size: Decimal, house_floor: u32) {

            assert!((land_right.resource_address()==self.land) & (house_right.resource_address() == self.house),
                "Wrong resource."
            );

            let mut house_data: House = house_right.non_fungible().data();

            let house_id = house_right.non_fungible::<House>().id();

            let mut land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.unwrap().0 == house_id,
                "This land doesn't contain the house from provided house right."
            );

            let location = land_data.location.clone();

            house_data.size = house_size;
            house_data.floor = house_floor;
            land_data.contain = Some((house_id, house_data));

            info!("You have modified the house right's NFT data of the house attached to the {} land according to construction data. New house is {}m2, {} floor", location, house_size, house_floor);

            self.controller_badge
                .authorize(|| {
                    land_right.non_fungible().update_data(land_data);
                    house_right.non_fungible().update_data(house_data)
                });

        }

        /// This method is for seller to sell a real estate right's NFTs.
        /// Input: Real estate's right NFTs:
        /// - If the land have no housing > input Enum("Land", Bucket("${land_right}"));
        /// - If the land contain a house > input Enum("LandandHouse", Bucket("${land_right}"), Bucket("${house_right}"));
        /// Output: The NFT keep track of real estate seller's order
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
        
                    self.book.insert(land_id.clone(), (price, None, false));
        
                    self.order_vault.put(land_right);

                    info!("You have created a sell order no.{} on the {} real estate", land_id, land_data.location);
        
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
        
                    self.book.insert(land_id.clone(), (price, Some(house_id), false));
        
                    self.order_vault.put(land_right);
        
                    self.order_contain_house.put(house_right);

                    info!("You have created a sell order no.{} on the {} real estate with an attached house", land_id, land_data.location);
        
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
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate", order_id, land_location);
                    return (RealEstate::Land(land_right), payment)
        
                }
        
                Some(house_id) => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(price*self.tax));
                    self.book.insert(order_id.clone(), (price, house, true));
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate with the attached house", order_id, land_location);
                    return (RealEstate::LandandHouse(land_right, self.order_contain_house.take_non_fungible(&house_id)), payment)
        
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

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (_price, house, status) = self.book.get(&order_id).unwrap().clone();

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

            match house.clone() {

                None => {
                    return RealEstate::Land(land_right)
                }

                Some(house_id) => {
                    return RealEstate::LandandHouse(land_right, self.order_contain_house.take_non_fungible(&house_id))
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

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (price, _house, status) = self.book.get(&order_id).unwrap().clone();

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
    }
}