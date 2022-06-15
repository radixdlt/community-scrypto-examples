//! [RealEstateConstructionInstitute] is the blueprint for construction institutes to manage real estate construction rights by NFT.
//! Construction institutes can use this blueprint to manage building NFTs, construction right's NFT through this blueprint.
//! Citizens can request construction through this blueprint.
//! This blueprint also contain a taxing mechanism for any construction authorization service.

use scrypto::prelude::*;
use crate::real_estate_service::*;

/// The building construction structure with needed input
#[derive(TypeId, Encode, Decode, Describe)]
pub struct ConstructBuilding {
    building_size: Decimal, 
    building_floor: u32
}

/// The type of construction
#[derive(TypeId, Encode, Decode, Describe)]
pub enum ConstructionType {
    ConstructBuilding(ConstructBuilding),
    DemolishBuilding
}

/// The construction right's NFT. This NFT is an authorized badge to divide a land, merge lands, constructe a building, re-constructe a building,...
/// Making change to any real estate NFTs need a construction NFT badge, show that the change can happend or has happened.
/// Authorities, organizations in charge of providing construction badge also need to make sure the change is feasible and won't cause bad effect to society.
#[derive(NonFungibleData)]
pub struct Construction {
    construction: ConstructionType
}

/// The NFT keep track of construction request
#[derive(NonFungibleData)]
pub struct ConstructionRequest {
    construction: ConstructionType
}


blueprint! {

    struct RealEstateConstructionInstitute {

        /// Component controller badge
        controller_badge: Vault,
        /// Building address
        building: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// Tax rate for govt authority (tokens)
        rate: Decimal,
        /// Fee rate paid on real estate service for construction institute (tokens)
        fee: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Authority's tax vault
        tax_vault: Vault,
        /// Construction Institute's fee vault
        fee_vault: Vault,
        /// Construction badge NFT resource address.
        construction_badge: ResourceAddress,
        /// Construction request badge NFT resource address.
        request_badge: ResourceAddress,
        /// Request book, contain in-queue construction request of citizens. Struct: Request Id, (the requested real estate data, status)
        request_book: HashMap<NonFungibleId, (RealEstateData, bool)>,
        /// Construction badge vault, contain solved construction request of citizens.
        construction_badge_vault: Vault,
        /// Request id counter
        request_counter: u64

    }

    impl RealEstateConstructionInstitute {
        
        /// This function will create new Real Estate Manager component
        /// Input: tax percent (paid on real estate trade), tax rate (pay on real estate service) and the medium token used for payment on market.
        /// Output: Component address and the authority badge.
        pub fn new(name: String, fee: Decimal, controller_badge: Bucket, rate: Decimal, medium_token: ResourceAddress, land: ResourceAddress, building: ResourceAddress, real_estate_authority: ResourceAddress) -> (ComponentAddress, Bucket) {

            let construction_authority_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + "Construction Institute Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .initial_supply(dec!(1));

            let construction_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone()+"Construction Right's Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let request_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name+"Construction Request Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("edit_rate", rule!(require(real_estate_authority)))
                .method("take_tax", rule!(require(real_estate_authority)))
                .default(rule!(allow_all));

            let comp = Self {

                controller_badge: Vault::with_bucket(controller_badge),
                building: building,
                land: land,
                rate: rate,
                fee: fee,
                token: medium_token,
                tax_vault: Vault::new(medium_token),
                fee_vault: Vault::new(medium_token),
                construction_badge: construction_badge,
                request_badge: request_badge,
                request_book: HashMap::new(),
                construction_badge_vault: Vault::new(construction_badge),
                request_counter: 0
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, construction_authority_badge)

        }

        pub fn new_construction_request(&self, real_estate_proof: RealEstateProof, construction: ConstructionType) -> Bucket {

            let real_estate_data = get_real_estate_data(real_estate_proof, self.land, self.building);
            
            let new_construction_request = ConstructionRequest {
                contain: None,
                size: land_size,
                location: location.clone()
            };

            let land_id: NonFungibleId = NonFungibleId::random();

            let land_right = self.authority_controller_badge.authorize(|| {
                borrow_resource_manager!(self.land)
                    .mint_non_fungible(&land_id, new_land)
            });

            info!("You have created a new land right's NFT of the {}m2 land on {}", land_size, location.clone());

            RealEstate::Land(land_right)

        }

        pub fn check_construction_data(&self, construction_data: ConstructionType) -> ConstructionType {

            return construction_data
            
        }

        pub fn new_construction_badge(&self, construction_data: ConstructionType) -> Bucket {

            let checked_construction_data = self.check_construction_data(construction_data);

            let construction = Construction {
                construction: checked_construction_data
            };

            let construction_id = NonFungibleId::random();

            let construction_right = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.construction_badge)
                    .mint_non_fungible(&construction_id, construction)
            });

            return construction_right

        }

        /// This method is for authority to add new building to an existed land.
        /// Input: The land right's NFT proof and building's data.
        /// Output: The building right's NFT of that land.
        pub fn construct_new_building(&self, land_right: Proof, building_size: Decimal, building_floor: u32) -> Bucket {

            assert!(land_right.resource_address()==self.land,
                "Wrong resource."
            );

            let new_building = Building {
                size: building_size,
                floor: building_floor
            };

            let building_id: NonFungibleId = NonFungibleId::random();

            let building_right = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.building)
                    .mint_non_fungible(&building_id, new_building.clone())
            });

            let mut data: Land = land_right.non_fungible().data();

            assert!(data.contain.is_none(),
                "This land already has a building."
            );

            data.contain = Some((building_id, new_building));

            let location = data.location.clone();

            self.controller_badge
                .authorize(|| land_right.non_fungible().update_data(data));

            info!("You have added a new building right's NFT of the {}m2, {} floor building attached to the land on {} according to construction data", building_size, building_floor, location);

            return building_right

        }

        /// This method is for authority to modify existed building info.
        /// Input: The land right's NFT proof, the building right's NFT proof of that land and the building's new data.
        /// Output: None.
        pub fn reconstruct_a_building(&self, land_right: Proof, building_right: Proof, building_size: Decimal, building_floor: u32) {

            assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                "Wrong resource."
            );

            let mut building_data: Building = building_right.non_fungible().data();

            let building_id = building_right.non_fungible::<Building>().id();

            let mut land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.unwrap().0 == building_id,
                "This land doesn't contain the building from provided building right."
            );

            let location = land_data.location.clone();

            building_data.size = building_size;
            building_data.floor = building_floor;
            land_data.contain = Some((building_id, building_data.clone()));

            info!("You have modified the building right's NFT data of the building attached to the {} land according to construction data. New building is {}m2, {} floor", location, building_size, building_floor);

            self.controller_badge
                .authorize(|| {
                    land_right.non_fungible().update_data(land_data);
                    building_right.non_fungible().update_data(building_data)
                });

        }

        pub fn take_tax(&mut self) -> Bucket {
            self.tax_vault.take_all()
        }

        pub fn edit_rate(&mut self, rate: Decimal) {
            self.rate = rate
        }
    }
}