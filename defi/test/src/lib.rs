use scrypto::prelude::*;


#[derive(NonFungibleData, TypeId, Encode, Decode, Describe, Copy, Clone)]
pub struct Building {
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
    contain: Bucket
}

blueprint! {
    struct Hello {
        land: ResourceAddress,
        building: ResourceAddress
    }

    impl Hello {
        
        pub fn instantiate_hello() -> ComponentAddress {

            let land = ResourceBuilder::new_non_fungible()
                .metadata("name", "Land")
                .mintable(rule!(allow_all), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .updateable_non_fungible_data(rule!(allow_all), LOCKED)
                .no_initial_supply();

            let building = ResourceBuilder::new_non_fungible()
                .metadata("name", "Building")
                .mintable(rule!(allow_all), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .updateable_non_fungible_data(rule!(allow_all), LOCKED)
                .no_initial_supply();

            
            Self {
                land: land,
                building: building
            }
            .instantiate()
            .globalize()
        }

        
        pub fn new_land_with_house(&mut self, building_size: Decimal, building_floor: u32, land_size: Decimal, location: String) -> Bucket {

            let new_building = Building {
                size: building_size,
                floor: building_floor
            };

            let building_id: NonFungibleId = NonFungibleId::random();

            let building_right = borrow_resource_manager!(self.building)
                    .mint_non_fungible(&building_id, new_building.clone());

            let new_land = Land {
                contain: building_right,
                size: land_size,
                location: location.clone()
            };

            let land_id: NonFungibleId = NonFungibleId::random();

            borrow_resource_manager!(self.land)
                .mint_non_fungible(&land_id, new_land)

        }
    }
}