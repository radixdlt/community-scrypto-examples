//! [RealEstateService] is the core blueprint for govt autorities to manage real estate rights by NFT.
//! Authorities can use this blueprint to manage land NFTs, land modify right's NFT, authorize new Market Place or Construction Institute.
//! Authorities can also collect, edit tax through this blueprint.
//! Citizens can request divide or merge lands through this blueprint.
//! This blueprint also contain a taxing mechanism for any land modify authorization service.

use scrypto::prelude::*;
use crate::real_estate_market_place::RealEstateMarketPlace;
use crate::real_estate_construction_institute::RealEstateConstructionInstitute;

/// The NFTs of real estates, can contain both land right's NFT and building right's NFT (if that land contain a building)
/// Real estate's NFTs cannot deposit into any other component except into the Market Place component,
/// This is to prevent citizens trade real estate's right on black market.
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstate {
    Land(Bucket),
    LandandBuilding(Bucket, Bucket)
}

/// The data of real estates, can contain both land data and building data (if that land contain a building)
#[derive(TypeId, Encode, Decode, Describe, Clone)]
pub enum RealEstateData {
    Land(Decimal, String),
    LandandBuilding(Decimal, String, Decimal, u32)
}

/// The Proofs of real estates, can contain both land right's NFT and building right's NFT (if that land contain a building)
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstateProof {
    Land(Proof),
    LandandBuilding(Proof, Proof)
}

/// The oracle data show that the new construction, new land, modified land is ok or not.
/// IsNextTo: oracle data show that the two queried real estate entity is next to each other.
/// IsNotOverLap: oracle data show that the queried real estate entity is not overlap with existed real estate or any other construction entity.
/// IsOk: oracle data (or manually feeded data) show that the queried construction will not cause bad effect to society.
#[derive(TypeId, Encode, Decode, Describe)]
pub enum Feasible {
    IsNextTo(bool),
    IsNotOverLap(bool),
    IsOk(bool)
}

/// The oracle data show where the building will belong to after divide a real estate (if that real estate contain a building).
#[derive(TypeId, Encode, Decode, Describe)]
pub enum BuildingOnLand {
    None,
    Land1,
    Land2
}

/// The type of modifying a land
/// Merge: The land data which the original real estate will merge with.
/// DivideRequest: The 2 land data which de original real estate will divide into.
/// Divide: same data as divide request but contain the data where the building on the original real estate (if exist) will be.
#[derive(TypeId, Encode, Decode, Describe)]
pub enum LandModifyType {
    Divide(RealEstateData, RealEstateData, BuildingOnLand),
    DivideRequest(RealEstateData, RealEstateData),
    Merge(NonFungibleId)
}

/// The return to citizens after solved the request, return a bucket if their request passed.
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RequestReturn {
    Rejected,
    Passed(Bucket),
}

/// The land modify right's NFT. This NFT is an authorized badge to divide a land, merge lands, constructe a building, re-constructe a building,...
/// Making change to any real estate NFTs need a land modify NFT badge, show that the change can happend or has happened.
/// Authorities, organizations in charge of providing land modify badge also need to make sure the change is feasible.
#[derive(NonFungibleData)]
pub struct LandModify {
    land_id: NonFungibleId,
    land_modify: LandModifyType
}

/// The NFT keep track of land modify or construction request
#[derive(NonFungibleData)]
pub struct Request {
}

/// The building right's NFT
/// All the building data (size, floor) is mutable through construction.
/// size (m2)
/// Building NFT data can only be updated on the Construction Institute component.
#[derive(NonFungibleData, TypeId, Encode, Decode, Describe, Clone)]
pub struct Building {
    #[scrypto(mutable)]
    pub size: Decimal,
    #[scrypto(mutable)]
    pub floor: u32
}

/// The land right's NFT
/// The land data (size, location) is immutable, but the building that land contain is mutable through construction.
/// One land can only contain one building. If citizens want to contain more, they need to divide the land first. 
/// Also, a the existed real estate with building can only merge with a land with no building.
/// This approach is more asset oriented when managing 2 types of asset that's linked to each other.
/// One individual, organization, company,... can have many real estate and group them with each other however they want.
/// size (m2)
/// Land NFT data can only be updated on the Real Estate Service component.
#[derive(NonFungibleData)]
pub struct Land {
    pub size: Decimal,
    pub location: String,
    #[scrypto(mutable)]
    pub contain: Option<(NonFungibleId, Building)>
}

/// A utility function to get real estate data from a real estate proof, also assert if the real estate proof is right resources or not.
pub fn get_real_estate_data(real_estate: RealEstateProof, land: ResourceAddress, building: ResourceAddress) -> RealEstateData {

    match real_estate {

        RealEstateProof::Land(land_right) => {

            assert!(land_right.resource_address()==land,
                "Wrong resource."
            );

            let land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.is_none(),
                "This land contain a building, you should also input the building right's NFT."
            );

            return  RealEstateData::Land(land_data.size, land_data.location)

        }

        RealEstateProof::LandandBuilding(land_right, building_right) => {

            assert!((land_right.resource_address()==land) & (building_right.resource_address() == building),
                "Wrong resource."
            );

            let building_id = building_right.non_fungible::<Building>().id();

            let building_data: Building = building_right.non_fungible().data();

            let land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.unwrap().0 == building_id,
                "This land doesn't contain the building from provided building right."
            );

            return RealEstateData::LandandBuilding(land_data.size, land_data.location, building_data.size, building_data.floor)

        }
    }
}

blueprint! {
    struct RealEstateService {

        /// Authority component controller badge
        controller_badge: Vault,
        /// Resource move badge
        move_badge: ResourceAddress,
        /// Market place component controller badge address
        market_controller_badge: ResourceAddress,
        /// Construction institute component controller address
        institute_controller_badge: ResourceAddress,
        /// Building address
        building: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// Tax percent paid on real estate trade (%)
        tax: Decimal,
        /// Tax rate paid on real estate construction service (tokens)
        rate: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Real estate authority badge address.
        real_estate_authority: ResourceAddress,
        /// Keep track of current running Real Estate Construction Institute.
        real_estate_construction_institute: HashSet<ComponentAddress>,
        /// Keep track of current running Real Estate Market Place.
        real_estate_market_place: HashSet<ComponentAddress>,
        /// The vault store tax.
        tax_vault: Vault,
        /// Land modify badge NFT resource address.
        land_modify_badge: ResourceAddress,
        /// Land modify request badge NFT resource address.
        request_badge: ResourceAddress,
        /// Request book, contain in-queue land modify request of citizens. Struct: Request Id, (the requested land id, status)
        request_book: HashMap<NonFungibleId, (NonFungibleId, LandModifyType)>,
        /// Land modify badge vault, contain solved land modify request of citizens.
        land_modify_badge_vault: Vault,
        /// Request id counter
        request_counter: u64

    }

    impl RealEstateService {
        
        /// This function will create new govt Real Estate Service component
        /// Input: 
        /// - the govt authority's name
        /// - tax percent (paid on real estate trade), 
        /// - tax rate (pay on real estate service),
        /// - the medium token used for payment on market.
        /// Output: Component address and the authority badge
        pub fn new(name: String, tax: Decimal, rate: Decimal, medium_token: ResourceAddress) -> (ComponentAddress, Bucket) {

            let authority_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + "Authority Controller Badge")
                .initial_supply(dec!(1));
           
            let market_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .metadata("name", name.clone() + "Market Place Controller Badge")
                .no_initial_supply();

            let institute_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .metadata("name", name.clone() + "Institute Controller Badge")
                .no_initial_supply();

            let move_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + "Resource Move Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address()) || require(market_controller_badge) || require(institute_controller_badge)), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address()) || require(market_controller_badge) || require(institute_controller_badge)), LOCKED)
                .no_initial_supply();

            let land = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + "Land")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let building = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + "Building")
                .mintable(rule!(require(institute_controller_badge)), LOCKED)
                .burnable(rule!(require(institute_controller_badge)), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .updateable_non_fungible_data(rule!(require(institute_controller_badge)), LOCKED)
                .no_initial_supply();

            let real_estate_authority = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name + "Authority Badge")
                .initial_supply(dec!(1));

            let land_modify_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Land Modify Right's Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let request_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Land Modify Request Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("new_land", rule!(require(real_estate_authority.resource_address())))
                .method("authorize", rule!(require(real_estate_authority.resource_address())))
                .method("collect_tax", rule!(require(real_estate_authority.resource_address())))
                .method("edit_tax", rule!(require(real_estate_authority.resource_address())))
                .method("edit_rate", rule!(require(real_estate_authority.resource_address())))
                .method("authorize_construction_institute", rule!(require(real_estate_authority.resource_address())))
                .method("authorize_marketplace", rule!(require(real_estate_authority.resource_address())))
                .default(rule!(allow_all));

            let comp = Self {

                controller_badge: Vault::with_bucket(authority_controller_badge),
                move_badge: move_badge,
                market_controller_badge: market_controller_badge,
                institute_controller_badge: institute_controller_badge,
                building: building,
                land: land,
                tax: tax/dec!(100),
                rate: rate,
                token: medium_token,
                real_estate_authority: real_estate_authority.resource_address(),
                real_estate_construction_institute: HashSet::new(),
                real_estate_market_place: HashSet::new(),
                tax_vault: Vault::new(medium_token),
                land_modify_badge: land_modify_badge,
                request_badge: request_badge,
                request_book: HashMap::new(),
                land_modify_badge_vault: Vault::new(land_modify_badge),
                request_counter: 0
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, real_estate_authority)

        }

        /// This method is for govt authority to create and distribute new real estate right's NFTs with the input data
        /// Input: 
        /// - real estate data: Enum("Land", Decimal("${land_size}"), "${location}");
        /// - is_overlap: data from Oracle > see if the land is overlap with an existed real estate or not: Enum("IsNotOverLap", Decimal("${land_size}"), "${location}")
        /// Output: The land right's NFT
        pub fn new_land(&self, data: RealEstateData, is_not_overlap: Feasible) -> Bucket {

            assert!(matches!(is_not_overlap, Feasible::IsNotOverLap(_)),
                "Wrong Oracle data"
            );

            assert!(matches!(is_not_overlap, Feasible::IsNotOverLap(true)),
                "This location is overlapped with existed real estate."
            );

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

                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size, location.clone());

                    land_right

                }

                _ => {panic!("You need to input Land data")}

            }
        }

        /// This is method for citizens to make new land divide request.
        /// Input: 
        /// - the citizen's original real estate proof: 
        /// + If the real estate doesn't contain a building: Enum("Land", Proof("land_proof"));
        /// + If the real estate contain a building: Enum("LandandBuilding", Proof("land_proof"), Proof("building_proof"));
        /// - the divided land 1 and land 2 information: Enum("Land", Decimal("${land1_size}"), "${location1}"), Enum("Land", Decimal("${land2_size}"), "${location2}"))
        /// Output: the request badge
        pub fn new_land_divide_request(&mut self, real_estate_proof: RealEstateProof, divided_land1: RealEstateData, divided_land2: RealEstateData) -> Bucket {

            let land_id = match real_estate_proof {
                RealEstateProof::Land(proof) => {proof.non_fungible::<Land>().id()}
                RealEstateProof::LandandBuilding(proof, _) => {proof.non_fungible::<Land>().id()}
            };

            let real_estate_data = get_real_estate_data(real_estate_proof, self.land, self.building);

            match divided_land1 {
                RealEstateData::LandandBuilding(_,_,_,_) => {panic!("Wrong real estate data provided!")}
                _ => {}
            };

            match divided_land2 {
                RealEstateData::LandandBuilding(_,_,_,_) => {panic!("Wrong real estate data provided!")}
                _ => {}
            };

            let location = match real_estate_data.clone() {
                RealEstateData::Land(_, location) => location,
                RealEstateData::LandandBuilding(_,location,_,_) => location
            };

            info!("You have created a new divide land request no.{} on the {} land.", self.request_counter, location);

            let new_land_modify_request = Request {};

            let request_id: NonFungibleId = NonFungibleId::from_u64(self.request_counter);

            let request_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.land)
                    .mint_non_fungible(&request_id, new_land_modify_request)
            });

            self.request_book.insert(request_id, (land_id, LandModifyType::DivideRequest(divided_land1, divided_land2)));

            self.request_counter += 1;

            request_badge

        }

        /// This is method for citizens to make new land merge request.
        /// Input: 
        /// - the citizen's original real estate proofs: 
        /// + If the real estate doesn't contain a building: Enum("Land", Proof("land_proof"));
        /// + If the real estate contain a building: Enum("LandandBuilding", Proof("land_proof"), Proof("building_proof"))
        /// Output: the request badge
        pub fn new_land_merge_request(&mut self, real_estate_proof1: RealEstateProof, real_estate_proof2: RealEstateProof) -> Bucket {

            let land_id1 = match real_estate_proof1 {
                RealEstateProof::Land(proof) => {proof.non_fungible::<Land>().id()}
                RealEstateProof::LandandBuilding(proof, _) => {proof.non_fungible::<Land>().id()}
            };

            let real_estate_data1 = get_real_estate_data(real_estate_proof1, self.land, self.building);

            let real_estate_data2 = get_real_estate_data(real_estate_proof2, self.land, self.building);

            match real_estate_data1.clone() {

                RealEstateData::Land(_, location1) => {

                    match real_estate_data2.clone() {

                        RealEstateData::Land(_, location2) => {

                            info!("You have created a new merge land request no.{} on the {} land and the {} land", self.request_counter, location1, location2);

                        }

                        RealEstateData::LandandBuilding(_, location2, _, _) => {
                            
                            info!("You have created a new merge land request no.{} on the {} land and the {} land with an attached building", self.request_counter, location1, location2);

                        }
                    }
                }

                RealEstateData::LandandBuilding(_, location1,_,_) =>{ 

                    match real_estate_data2.clone() {

                        RealEstateData::Land(_, location2) => {

                            info!("You have created a new merge land request no.{} on the {} land with an attached building and the {} land", self.request_counter, location1, location2);

                        }

                        RealEstateData::LandandBuilding(_, _, _, _) => {
                            
                            panic!("You shouldn't merge 2 land with 2 buildings!");

                        }
                    }
                }

            };
               
            let new_land_modify_request = Request {};

            let request_id: NonFungibleId = NonFungibleId::from_u64(self.request_counter);

            let request_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.land)
                    .mint_non_fungible(&request_id, new_land_modify_request)
            });

            self.request_book.insert(request_id, (real_estate_data1, LandModifyType::Merge(real_estate_data2)));

            self.request_counter += 1;

            request_badge

        }

        /// This method is for govt authority to authorize a land modify request
        /// Input: 
        /// - the request id: ${request_id}u64
        /// - the oracle data show whether this land modify is ok or not: 
        /// + If the user want divide a land: Enum("IsNotOverLap", ${ok_or_not})
        /// + If the user want merge a land: Enum("IsNextTo", ${ok_or_not})
        /// - building on land: the oracle data show that where the building would belong to after divided a real estate contain a building:
        /// + None if the request is to merge a land or that land before divided contain no building
        /// + Land1 if the building would be on the first land after divided
        /// + Land2 if the building would be on the second land after divided
        /// Output: None
        /// This method put the authorized land modify NFT on the component vault if passed.
        pub fn authorize_land_modify(&mut self, id: u64, is_ok: Feasible, building_on_land: BuildingOnLand) {

            let request_id = NonFungibleId::from_u64(id);

            let result = self.request_book.get(&request_id);

            assert!(result.is_some(),
                "The request book doesn't contain this request id."
            );

            let (_, land_modify) = result.unwrap();

            let (real_estate_data, land_modify) = match land_modify {

                LandModifyType::DivideRequest(_, _) => {

                    assert!(matches!(is_ok, Feasible::IsNotOverLap(_)),
                    "Wrong Oracle data."
                    );

                    assert!(matches!(is_ok, Feasible::IsNotOverLap(true)),
                    "The divide line is overlap with other construction, you cannot authorize this land divide."
                    );

                    self.request_book.remove(&request_id).unwrap()

                }

                LandModifyType::Merge(_) => {

                    assert!(matches!(is_ok, Feasible::IsNextTo(_)),
                    "Wrong Oracle data."
                    );

                    assert!(matches!(is_ok, Feasible::IsNextTo(true)),
                    "The two land provided is not next to each other."
                    );

                    self.request_book.remove(&request_id).unwrap()

                }

                _ => {panic!("Wrong land modify data")}
            };

            let land_modify_data = LandModify {
                real_estate_data: real_estate_data,
                land_modify: land_modify
            };

            self.controller_badge.authorize(|| {
                let land_modify_right = borrow_resource_manager!(self.land_modify_badge)
                    .mint_non_fungible(&request_id, land_modify_data);
                self.land_modify_badge_vault.put(land_modify_right);
            });

            info!("You have authorized the land modify request no.{}", id);

        }

        /// This method is for citizens to get their construct badge after authorized.
        /// Input: the request badge: Bucket("${request_badge}")
        /// Output: the request result returned: if passed > return land modify right badge.
        pub fn get_land_modify_badge(&mut self, request_badge: Bucket) -> RequestReturn {

            assert!(request_badge.resource_address() == self.request_badge,
                "Wrong resource!"
            );

            let request_id = request_badge.non_fungible::<Request>().id();

            assert!(!self.request_book.contains_key(&request_id),
                "The authority haven't reviewed your request yet"
            );

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.request_badge)
                    .burn(request_badge)
            });

            if self.land_modify_badge_vault.non_fungible_ids().contains(&request_id) {
                info!("Your land modify request no.{} has passed.", request_id);
                RequestReturn::Passed(self.land_modify_badge_vault.take_non_fungible(&request_id))
            } else {
                info!("Your land modify request no.{} has been rejected.", request_id);
                RequestReturn::Rejected
            }

        }

        /// This method is for authority to divide an existed real estate to 2 other real estates with attached NFTs.
        /// Input: 
        /// real_estate:
        /// - If the real estate have no building > input the land right's NFT
        /// - If the real estate contain a building > input both the land right's NFT and the building right's NFT
        /// real_estate1_data, real_estate2_data: input Enum("Land", Decimal("${land_size}"), "${location}"), these data can also be acquired through Oracle.
        /// building_on_land: data from an Oracle > see the location of building (if the original real estate contain a building) belong to the real estate 1 or real estate 2.
        /// this data is either Default if the original real estate have no building, "1" if the building is on real estate 1, "2" if the building is on real estate 2.
        /// To follow the asset oriented logic, the divided real estate should not have a building which the original real estate didn't have.
        /// Output: 2 divided real estate with the attached NFTs.
        pub fn divide_land(&self, real_estate: RealEstate, real_estate1_data: RealEstateData, real_estate2_data: RealEstateData, building_on_land: String) -> (RealEstate, RealEstate) {

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let result = match real_estate1_data {

                        RealEstateData::Land(land_size1, location1) => {
                            
                            match real_estate2_data {
                                
                                RealEstateData::Land(land_size2, location2) => {

                                    assert!((land_size1+land_size2==land_data.size) & (land_size1>dec!(0)) & (land_size2>dec!(0)),
                                        "Wrong land size data provided!"
                                    );

                                    let new_land1 = Land {
                                        contain: None,
                                        size: land_size1,
                                        location: location1.clone()
                                    };
                
                                    let land_id1: NonFungibleId = NonFungibleId::random();
                
                                    let land_right1 = self.controller_badge.authorize(|| {
                                        borrow_resource_manager!(self.land)
                                            .mint_non_fungible(&land_id1, new_land1)
                                    });
                
                                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size1, location1.clone());

                                    let new_land2 = Land {
                                        contain: None,
                                        size: land_size2,
                                        location: location2.clone()
                                    };
                
                                    let land_id2: NonFungibleId = NonFungibleId::random();
                
                                    let land_right2 = self.controller_badge.authorize(|| {
                                        borrow_resource_manager!(self.land)
                                            .mint_non_fungible(&land_id2, new_land2)
                                    });
                
                                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size2, location2.clone());

                                    info!("You have divided the {} land into a {}m2 land on {} and a {}m2 land on {}", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                    (RealEstate::Land(land_right1), RealEstate::Land(land_right2))
                                }
                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                            }
                        }
                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                    };

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right)
                    });

                    return result

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();

                    let building_data: Building = building_right.non_fungible().data();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );

                    let result = match real_estate1_data {

                        RealEstateData::Land(land_size1, location1) => {
                            
                            match real_estate2_data {
                                
                                RealEstateData::Land(land_size2, location2) => {

                                    assert!((land_size1+land_size2==land_data.size) & (land_size1>dec!(0)) & (land_size2>dec!(0)),
                                        "Wrong land size data provided!"
                                    );

                                    let result2 = if building_on_land == "1" {

                                        let new_land1 = Land {
                                            contain: Some((building_id, building_data)),
                                            size: land_size1,
                                            location: location1.clone()
                                        };
                    
                                        let land_id1: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right1 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id1, new_land1)
                                        });

                                        info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_size1, location1.clone());

                                        let new_land2 = Land {
                                            contain: None,
                                            size: land_size2,
                                            location: location2.clone()
                                        };
                    
                                        let land_id2: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right2 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id2, new_land2)
                                        });
                    
                                        info!("You have created a new land right's NFT of the {}m2 land on {}", land_size2, location2.clone());

                                        info!("You have divided the {} land into a {}m2 land on {} with an attached building and a {}m2 land on {}", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                        (RealEstate::LandandBuilding(land_right1, building_right), RealEstate::Land(land_right2))

                                    } else if building_on_land == "2" {

                                        let new_land1 = Land {
                                            contain: None,
                                            size: land_size1,
                                            location: location1.clone()
                                        };
                    
                                        let land_id1: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right1 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id1, new_land1)
                                        });

                                        info!("You have created a new land right's NFT of the {}m2 land on {}", land_size1, location1.clone());

                                        let new_land2 = Land {
                                            contain: Some((building_id, building_data)),
                                            size: land_size2,
                                            location: location2.clone()
                                        };
                    
                                        let land_id2: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right2 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id2, new_land2)
                                        });
                    
                                        info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_size2, location2.clone());

                                        info!("You have divided the {} land into a {}m2 land on {} and a {}m2 land on {} with an attached building", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                        (RealEstate::Land(land_right1), RealEstate::LandandBuilding(land_right2, building_right))

                                    } else {
                                        panic!("Wrong Oracle data!")
                                    };

                                    result2

                                }
                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                            }
                        }
                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                    };
        
                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right)
                    });

                    return result

                }
            }    
        }

        /// This method is for authority to merge an existed real estate with an existed (non-building) land.
        /// Input: 
        /// real_estate:
        /// - If the real estate have no building > input the land right's NFT.
        /// - If the real estate contain a building > input both the land right's NFT and the building right's NFT.
        /// land_right2: input the land_right's bucket.
        /// is_next_to: data from an Oracle > see if the 2 land next to each other or not.
        /// To follow the asset oriented logic, the merged land should not have more than 2 buildings.
        /// Output: 1 merged real estate with the attached NFTs. The location data of new real estate is equal to the first original real estate.
        pub fn merge_land(&self, real_estate: RealEstate, land_right2: Bucket, is_next_to: bool) -> RealEstate {

            assert!(land_right2.resource_address() == self.land,
                "Wrong resource"
            );

            assert!(is_next_to,
                "The land should be next to each other for the merge!"
            );

            let land_data2: Land = land_right2.non_fungible().data();

            assert!(land_data2.contain.is_none(),
                        "This land shouldn't contain a building."
            );

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_data: Land = land_right.non_fungible().data();

                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let new_land = Land {
                        contain: None,
                        size: land_data.size+land_data2.size,
                        location: land_data.location.clone()
                    };

                    let land_id_merged: NonFungibleId = NonFungibleId::random();

                    let land_right_merged = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id_merged, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_data.size+land_data2.size, land_data.location.clone());

                    info!("You have merged the {}m2 land on {} and {}m2 land on {} into a {}m2 land on {}", land_data.size, land_data.location.clone(), land_data2.size, land_data2.location.clone(), land_data.size+land_data2.size, land_data.location.clone());

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right);
                        borrow_resource_manager!(self.land)
                            .burn(land_right2);
                    });

                    return RealEstate::Land(land_right_merged)

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();

                    let building_data = building_right.non_fungible::<Building>().data();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.clone().unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );
        
                    let new_land = Land {
                        contain: Some((building_id, building_data)),
                        size: land_data.size+land_data2.size,
                        location: land_data.location.clone()
                    };

                    let land_id_merged: NonFungibleId = NonFungibleId::random();

                    let land_right_merged = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id_merged, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_data.size+land_data2.size, land_data.location.clone());

                    info!("You have merged the {}m2 land on {} with an attached building and {}m2 land on {} into a {}m2 land on {} with an attached building", land_data.size, land_data.location.clone(), land_data2.size, land_data2.location.clone(), land_data.size+land_data2.size, land_data.location.clone());

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right);
                        borrow_resource_manager!(self.land)
                            .burn(land_right2);
                    });

                    return RealEstate::LandandBuilding(land_right_merged, building_right)

                }
            }    
        }

        /// This method is for authority to authorize a construction institute.
        /// Input: the institute's name, institute service's fee (tokens)
        /// Output: the institute's badge
        pub fn authorize_construction_institute(&mut self, name: String, fee: Decimal) -> Bucket {

            let institute_controller_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.institute_controller_badge)
                    .mint(dec!(1))
            });
            let (institute_comp, institute_badge) = RealEstateConstructionInstitute::new(name, fee, institute_controller_badge, self.rate, self.token, self.land, self.building, self.real_estate_authority);
            self.real_estate_construction_institute.insert(institute_comp);
            return institute_badge

        }

        /// This method is for authority to authorize a real estate market place.
        /// Input: the market's name, market's fee (%).
        /// Output: the market's badge.
        pub fn authorize_marketplace(&mut self, name: String, fee: Decimal) -> Bucket {

            let market_controller_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.market_controller_badge)
                    .mint(dec!(1))
            });
            let (market_comp, market_host_badge) = RealEstateMarketPlace::new(name, market_controller_badge, fee, self.tax, self.land, self.building, self.token, self.real_estate_authority);
            self.real_estate_market_place.insert(market_comp);
            return market_host_badge

        }

        pub fn edit_tax(&mut self, tax: Decimal) {

            self.tax = tax;

            self.real_estate_market_place.iter().for_each(|address| {
                    let market: RealEstateMarketPlace = address.clone().into();
                    market.edit_tax(tax)
                })

        }

        pub fn edit_rate(&mut self, rate: Decimal) {

            self.rate = rate;

            self.real_estate_construction_institute.iter().for_each(|address| {
                let institute: RealEstateConstructionInstitute = address.clone().into();
                institute.edit_rate(rate)
            })

        }

        pub fn collect_tax(&mut self) -> Bucket {

            self.real_estate_market_place.iter().for_each(|address| {
                let market: RealEstateMarketPlace = address.clone().into();
                self.tax_vault.put(market.take_tax());
            });

            self.real_estate_construction_institute.iter().for_each(|address| {
                let institute: RealEstateConstructionInstitute = address.clone().into();
                self.tax_vault.put(institute.take_tax());
            });

            self.tax_vault.take_all()
        }
    }
}