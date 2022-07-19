//! [RealEstateService] is the core blueprint for govt autorities to manage real estate rights by NFT.
//! Authorities can use this blueprint to manage land NFTs, land modify right's NFT, authorize new Market Place or Construction Institute.
//! Authorities can also collect, edit tax through this blueprint.
//! Citizens can request divide or merge lands through this blueprint.
//! This blueprint also contain a taxing mechanism for any land modify authorization service.

use scrypto::prelude::*;
use crate::real_estate_market_place::RealEstateMarketPlace;
use crate::real_estate_construction_institute::RealEstateConstructionInstitute;
use crate::utility::*;

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

#[derive(TypeId, Encode, Decode, Describe)]
pub enum OptionProof {
    None,
    Some(Proof)
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
/// To follow the asset oriented logic, the merged land should not have more than 2 buildings.
/// DivideRequest: The 2 land data which de original real estate will divide into.
/// Divide: same data as divide request but contain the data where the building on the original real estate (if exist) will be.
#[derive(TypeId, Encode, Decode, Describe)]
pub enum LandModifyType {
    Divide(RealEstateData, RealEstateData, Option<NonFungibleId>, BuildingOnLand),
    DivideRequest(RealEstateData, RealEstateData, Option<NonFungibleId>),
    Merge(NonFungibleId, Option<NonFungibleId>)
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

/// The NFT keep track of citizen's personal information (might be intergrated with citizen ID).
#[derive(NonFungibleData)]
pub struct ID {
}

/// The NFT keep track of market place information.
#[derive(NonFungibleData)]
pub struct MarketHostBadge {
}

/// The NFT keep track of construction institute information.
#[derive(NonFungibleData)]
pub struct InstituteBadge {
}

/// The building right's NFT
/// All the building data (size, floor) is mutable through construction.
/// size (m2)
/// Building NFT data can only be updated on the Construction Institute component.
#[derive(NonFungibleData)]
pub struct Building {
    pub size: Decimal,
    pub floor: u32
}

/// The land right's NFT
/// The land data (size, location) is immutable, but the building that land contain is mutable through construction.
/// One land can only contain one building. If citizens want to contain more, they need to divide the land first. 
/// Also, a the existed real estate with building can only merge with a land with no building.
/// This approach is more asset oriented when managing 2 types of asset that's linked to each other.
/// One individual, organization, company,... can have many real estate and group them with each other however they want.
/// size: m2
/// location: can be the location address ID, not real address location (for privacy purpose)
/// Land NFT data can only be updated on the Real Estate Service component.
#[derive(NonFungibleData)]
pub struct Land {
    pub size: Decimal,
    pub location: String,
    #[scrypto(mutable)]
    pub contain: Option<NonFungibleId>
}

blueprint! {

    struct RealEstateService {

        /// Citizen ID badge
        id_badge: ResourceAddress,
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
        /// The medium token using for payment, possibly the CBDC token on Radix chain.
        token: ResourceAddress,
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
        /// Request book, contain in-queue land modify request of citizens. Struct: Request Id, (the requested land id, the building on land (if that land has a building) id, status)
        request_book: HashMap<NonFungibleId, (NonFungibleId, LandModifyType)>,
        /// Land modify badge vault, contain solved land modify request of citizens.
        land_modify_badge_vault: Vault,
        /// Request id counter
        request_counter: u64,
        /// Construction institute badge
        institute_badge: ResourceAddress,
        /// Market host badge
        market_host_badge: ResourceAddress

    }

    impl RealEstateService {
        
        /// This function will create new govt Real Estate Service component
        /// Input: 
        /// - the govt authority's name
        /// - tax percent (paid on real estate trade), 
        /// - tax rate (pay on real estate service),
        /// - the medium token used for payment on market (potentially the CBDC token on Radix chain).
        /// Output: Component address and the authority badge
        pub fn new(name: String, tax: Decimal, rate: Decimal, medium_token: ResourceAddress) -> (ComponentAddress, Bucket) {

            let authority_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + " Authority Controller Badge")
                .initial_supply(dec!(1));

            let id_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + " Citizen ID badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .restrict_withdraw(rule!(deny_all), LOCKED)
                .no_initial_supply();

            let market_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .metadata("name", name.clone() + " Market Place Controller Badge")
                .no_initial_supply();

            let institute_controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .metadata("name", name.clone() + " Institute Controller Badge")
                .no_initial_supply();

            let move_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + " Resource Move Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address()) || require(market_controller_badge) || require(institute_controller_badge)), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address()) || require(market_controller_badge) || require(institute_controller_badge)), LOCKED)
                .no_initial_supply();

            let land = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + " Land Right's NFT")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .restrict_withdraw(rule!(require(id_badge)), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address()) || require(institute_controller_badge)), LOCKED)
                .no_initial_supply();

            let building = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + " Building Right's NFT")
                .mintable(rule!(require(institute_controller_badge)), LOCKED)
                .burnable(rule!(require(institute_controller_badge)), LOCKED)
                .restrict_withdraw(rule!(require(id_badge)), LOCKED)
                .restrict_deposit(rule!(require(move_badge)), LOCKED)
                .no_initial_supply();

            let real_estate_authority = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", name.clone() + " Authority Badge")
                .initial_supply(dec!(1));

            let land_modify_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + " Land Modify Right's Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let request_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone() + " Land Modify Request Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let market_host_badge = ResourceBuilder::new_non_fungible()
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .metadata("name", name.clone() + " Real Estate Market Place Host Badge")
                .no_initial_supply();

            let institute_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name + " Construction Institute Badge")
                .mintable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(authority_controller_badge.resource_address())), LOCKED)
                .no_initial_supply();
                
            let rules = AccessRules::new()
                .method("authorize_citizen", rule!(require(real_estate_authority.resource_address())))
                .method("new_land", rule!(require(real_estate_authority.resource_address())))
                .method("authorize_land_modify", rule!(require(real_estate_authority.resource_address())))
                .method("collect_tax", rule!(require(real_estate_authority.resource_address())))
                .method("edit_tax", rule!(require(real_estate_authority.resource_address())))
                .method("edit_rate", rule!(require(real_estate_authority.resource_address())))
                .method("authorize_construction_institute", rule!(require(real_estate_authority.resource_address())))
                .method("authorize_marketplace", rule!(require(real_estate_authority.resource_address())))
                .method("rate", rule!(allow_all))
                .method("tax", rule!(allow_all))
                .method("deposit_tax", rule!(allow_all))
                .default(rule!(require(id_badge)));

            let comp = Self {

                id_badge: id_badge,
                controller_badge: Vault::with_bucket(authority_controller_badge),
                move_badge: move_badge,
                market_controller_badge: market_controller_badge,
                institute_controller_badge: institute_controller_badge,
                building: building,
                land: land,
                tax: tax/dec!(100),
                rate: rate,
                token: medium_token,
                real_estate_construction_institute: HashSet::new(),
                real_estate_market_place: HashSet::new(),
                tax_vault: Vault::new(medium_token),
                land_modify_badge: land_modify_badge,
                request_badge: request_badge,
                request_book: HashMap::new(),
                land_modify_badge_vault: Vault::new(land_modify_badge),
                request_counter: 0,
                market_host_badge: market_host_badge,
                institute_badge: institute_badge
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, real_estate_authority)

        }

        /// This method is for govt authority to authorize a citizen to use real estate services.
        /// Input: the citizen id, Output: the ID badge.
        /// Without ID badge, user cannot use any on-chain real estate services.
        pub fn authorize_citizen(&self, citizen_id: u64) -> Bucket {

            let citizen_id = NonFungibleId::from_u64(citizen_id);

            let citizen = ID {};

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.id_badge)
                    .mint_non_fungible(&citizen_id, citizen)
            })

        }

        /// This method is for govt authority to create and distribute new real estate right's NFTs with the input data
        /// Input: 
        /// - real estate data: Decimal("${land_size}"), "${location}"
        /// - is_overlap: data from Oracle > see if the land is overlap with an existed real estate or not: Enum("IsNotOverLap", ${ok_or_not})
        /// Output: The land right's NFT
        pub fn new_land(&self, land_size: Decimal, location: String, is_not_overlap: Feasible) -> (Bucket, Proof) {

            assert!(matches!(is_not_overlap, Feasible::IsNotOverLap(_)),
                "Wrong Oracle data"
            );

            assert!(matches!(is_not_overlap, Feasible::IsNotOverLap(true)),
                "This location is overlapped with existed real estate."
            );

            assert!(land_size > dec!(0),
                "Must provide the right land size"
            );

            let new_land = Land {
                contain: None,
                size: land_size,
                location: location.clone()
            };

            let land_id: NonFungibleId = NonFungibleId::random();

            let result = self.controller_badge.authorize(|| {

                let move_badge = borrow_resource_manager!(self.move_badge)
                    .mint(dec!(1));

                let move_proof = move_badge.create_proof();

                borrow_resource_manager!(self.move_badge)
                    .burn(move_badge);

                (borrow_resource_manager!(self.land)
                    .mint_non_fungible(&land_id, new_land), move_proof)

                });

            info!("You have created a new land right's NFT of the {}m2 land on {}", land_size, location.clone());

            result

        }

        /// This is method for citizens to make new land divide request.
        /// Input: 
        /// - the citizen's original real estate proof: 
        /// + If the real estate doesn't contain a building: Enum("Land", Proof("land_proof"));
        /// + If the real estate contain a building: Enum("LandandBuilding", Proof("land_proof"), Proof("building_proof"));
        /// - the divided land 1 and land 2 information: 
        /// + Enum("Land", Decimal("${land1_size}"), "${location1}") Enum("Land", Decimal("${land2_size}"), "${location2}"))
        /// + These data can be from Oracle.
        /// Output: the request badge
        pub fn new_land_divide_request(&mut self, real_estate_proof: RealEstateProof, divided_land1: RealEstateData, divided_land2: RealEstateData) -> Bucket {

            let (land_id, building_id, real_estate_data) = get_real_estate_data(real_estate_proof, self.land, self.building);

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

            let new_land_modify_request = Request {};

            let request_id: NonFungibleId = NonFungibleId::from_u64(self.request_counter);

            let request_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.request_badge)
                    .mint_non_fungible(&request_id, new_land_modify_request)
            });

            self.request_book.insert(request_id.clone(), (land_id, LandModifyType::DivideRequest(divided_land1, divided_land2, building_id)));

            self.request_counter += 1;

            info!("You have created a new divide land request no.{} on the {} land.", request_id, location);

            request_badge

        }

        /// This is method for citizens to make new land merge request.
        /// Input: 
        /// - the citizen's original real estate proofs: 
        /// + If the real estate doesn't contain a building: Enum("Land", Proof("land_proof"));
        /// + If the real estate contain a building: Enum("LandandBuilding", Proof("land_proof"), Proof("building_proof"))
        /// Output: the request badge
        pub fn new_land_merge_request(&mut self, real_estate_proof1: RealEstateProof, real_estate_proof2: RealEstateProof) -> Bucket {

            let request_id: NonFungibleId = NonFungibleId::from_u64(self.request_counter);

            let (land_id1, land_id2, building_id): (NonFungibleId, NonFungibleId, Option<NonFungibleId>) = match real_estate_proof1 {

                RealEstateProof::Land(proof) => {

                    let (land_id1, land_data1) = assert_land_proof(proof, self.land);

                    let (land_id2, building_id) = match real_estate_proof2 {

                        RealEstateProof::Land(proof2) => {

                            let (land_id2, land_data2) = assert_land_proof(proof2, self.land);

                            info!("You have created a new merge land request no.{} on the {} land and the {} land", request_id.clone(), land_data1.location, land_data2.location);

                            (land_id2, None)

                        }

                        RealEstateProof::LandandBuilding(proof2, building_proof2) => {

                            let (land_id2, land_data2, building_id, _) = assert_landandbuilding_proof(proof2, building_proof2, self.land, self.building);

                            info!("You have created a new merge land request no.{} on the {} land and the {} land with an attached building", request_id.clone(), land_data1.location, land_data2.location);

                            (land_id2, Some(building_id))

                        }
                    };

                    (land_id1, land_id2, building_id)
                }

                RealEstateProof::LandandBuilding(proof, building_proof) => {

                    let (land_id1, land_data1, building_id, _) = assert_landandbuilding_proof(proof, building_proof, self.land, self.building);

                    let land_id2 = match real_estate_proof2 {
                        RealEstateProof::Land(proof2) => {

                            let (land_id2, land_data2) = assert_land_proof(proof2, self.land);

                            info!("You have created a new merge land request no.{} on the {} land with an attached building and the {} land", request_id.clone(), land_data1.location, land_data2.location);
                            land_id2
                        }
                        RealEstateProof::LandandBuilding(_, _) => {
                            panic!("You shouldn't merge 2 land with 2 buildings!");
                        }
                    };

                    (land_id1, land_id2, Some(building_id))

                }
            };
               
            let new_land_modify_request = Request {};

            let request_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.request_badge)
                    .mint_non_fungible(&request_id, new_land_modify_request)
            });

            self.request_book.insert(request_id, (land_id1, LandModifyType::Merge(land_id2, building_id)));

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
        /// + None if the request is to merge a land or that land before divided contain no building: Enum("None")
        /// + Land1 if the building would be on the first land after divided: Enum("Land1")
        /// + Land2 if the building would be on the second land after divided: Enum("Land2")
        /// Output: None
        /// This method put the authorized land modify NFT on the component vault if passed.
        pub fn authorize_land_modify(&mut self, id: u64, is_ok: Feasible, building_on_land: BuildingOnLand) {

            let request_id = NonFungibleId::from_u64(id);

            let result = self.request_book.get(&request_id);

            assert!(result.is_some(),
                "The request book doesn't contain this request id."
            );

            let (_, land_modify) = result.unwrap();

            let (land_id, land_modify) = match land_modify {

                LandModifyType::DivideRequest(_, _, _) => {

                    assert!(matches!(is_ok, Feasible::IsNotOverLap(_)),
                    "Wrong Oracle data."
                    );

                    let (land_id, land_modify) = self.request_book.remove(&request_id).unwrap();

                    let land_modify_result = match land_modify {
                        LandModifyType::DivideRequest(land_data1, land_data2, building_id) => LandModifyType::Divide(land_data1, land_data2, building_id, building_on_land),
                        _ => {panic!("Wrong land modify data")}
                    };

                    assert!(matches!(is_ok, Feasible::IsNotOverLap(true)),
                    "The divide line is overlap with other construction, you cannot authorize this land divide."
                    );

                    (land_id, land_modify_result)

                }

                LandModifyType::Merge(_, _) => {

                    assert!(matches!(is_ok, Feasible::IsNextTo(_)),
                    "Wrong Oracle data."
                    );

                    let (land_id, land_modify) = self.request_book.remove(&request_id).unwrap();

                    assert!(matches!(is_ok, Feasible::IsNextTo(true)),
                    "The two land provided is not next to each other."
                    );

                    (land_id, land_modify)

                }

                _ => {panic!("Wrong land modify data")}
            };

            let land_modify_data = LandModify {
                land_id: land_id,
                land_modify: land_modify
            };

            self.controller_badge.authorize(|| {
                let land_modify_right = borrow_resource_manager!(self.land_modify_badge)
                    .mint_non_fungible(&request_id, land_modify_data);
                self.land_modify_badge_vault.put(land_modify_right);
            });

            info!("You have authorized the land modify request no.{}", request_id);

        }

        /// This method is for citizens to get their construct badge after authorized.
        /// Input: the request badge: Bucket("request_badge")
        /// Output: the request result returned: if passed > return land modify right badge.
        pub fn get_land_modify_badge(&mut self, request_badge: Bucket) -> Option<Bucket> {

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
                Some(self.land_modify_badge_vault.take_non_fungible(&request_id))
            } else {
                info!("Your land modify request no.{} has been rejected.", request_id);
                None
            }

        }

        /// This method is for authority to divide an existed real estate to 2 other real estates with attached NFTs.
        /// Input: 
        /// - the land right's NFT: Bucket("land_right")
        /// - the building NFT proof: 
        /// + If the land contain a building: Enum("Some", Proof("building_proof"))
        /// + If that's a barren land: Enum("None")
        /// - the land modify badge: Bucket("land_modify_badge")
        /// - the payment bucket: Bucket("payment")
        /// Output: The building right's NFTs of the divided lands and payment changes.
        pub fn divide_land(&mut self, land_right: Bucket, building_proof: OptionProof, land_modify_badge: Bucket, mut payment: Bucket) -> (Bucket, Bucket, Bucket, Proof) {

            assert!((land_modify_badge.resource_address()==self.land_modify_badge) & (payment.resource_address()==self.token),
                "Wrong resource."
            );

            let data = land_modify_badge.non_fungible::<LandModify>().data();

            let land_modify_data = data.land_modify;

            let id = data.land_id;

            let amount = payment.amount();

            self.tax_vault.put(payment.take(self.rate));

            match land_modify_data {

                LandModifyType::Divide(real_estate1_data, real_estate2_data, building_id, building_on_land) => {

                    match building_id {

                        None => {

                            assert!(matches!(building_proof, OptionProof::None),
                                "Wrong building proof provided."
                            );
        
                            let (land_id, land_data) = assert_land_proof(land_right.create_proof(), self.land);
        
                            assert!(land_id == id,
                                "Wrong land right's NFT provided"
                            );
                
                            match real_estate1_data {
        
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
        
                                            info!("You have paid {} tokens to divide the {} land into a {}m2 land on {} and a {}m2 land on {}", amount, land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                            let move_proof = self.controller_badge.authorize(|| {

                                                let move_badge = borrow_resource_manager!(self.move_badge)
                                                .mint(dec!(1));

                                                let move_proof = move_badge.create_proof();

                                                borrow_resource_manager!(self.move_badge)
                                                    .burn(move_badge);

                                                borrow_resource_manager!(self.land)
                                                    .burn(land_right);
                                                borrow_resource_manager!(self.land_modify_badge)
                                                    .burn(land_modify_badge);

                                                    move_proof

                                            });
        
                                            (land_right1, land_right2, payment, move_proof)
                                        }
                                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                                    }
                                }
                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                            }
                        }
        
                        Some(building_id) => {

                            match building_proof {

                                OptionProof::None => {panic!("Wrong building proof provided.")}

                                OptionProof::Some(building_proof) => {

                                    let building_proof_id = building_proof.non_fungible::<Building>().id();

                                    assert!(building_proof_id == building_id,
                                        "Wrong building proof provided."
                                    );

                                    let (land_id, land_data, _, _) = assert_landandbuilding_proof(land_right.create_proof(), building_proof, self.land, self.building);

                                    assert!(land_id == id,
                                        "Wrong land right's NFT provided"
                                    );
                
                                    match real_estate1_data {
                
                                        RealEstateData::Land(land_size1, location1) => {
                                            
                                            match real_estate2_data {
                                                
                                                RealEstateData::Land(land_size2, location2) => {
                
                                                    assert!((land_size1+land_size2==land_data.size) & (land_size1>dec!(0)) & (land_size2>dec!(0)),
                                                        "Wrong land size data provided!"
                                                    );

                                                    match building_on_land {

                                                        BuildingOnLand::Land1 => {

                                                            let new_land1 = Land {
                                                                contain: Some(building_id),
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
                    
                                                            info!("You have paid {} tokens to divide the {} land into a {}m2 land on {} with an attached building and a {}m2 land on {}", amount, land_data.location, land_size1, location1.clone(), land_size2, location2.clone());
            
                                                            let move_proof = self.controller_badge.authorize(|| {
                                                                let move_badge = borrow_resource_manager!(self.move_badge)
                                                                .mint(dec!(1));

                                                                let move_proof = move_badge.create_proof();

                                                                borrow_resource_manager!(self.move_badge)
                                                                    .burn(move_badge);
                                                                borrow_resource_manager!(self.land)
                                                                    .burn(land_right);
                                                                borrow_resource_manager!(self.land_modify_badge)
                                                                    .burn(land_modify_badge);
                                                                    move_proof
                                                            });
                    
                                                            (land_right1, land_right2, payment, move_proof)
                                                        }

                                                        BuildingOnLand::Land2 => {

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
                                                                contain: Some(building_id),
                                                                size: land_size2,
                                                                location: location2.clone()
                                                            };
                                        
                                                            let land_id2: NonFungibleId = NonFungibleId::random();
                                        
                                                            let land_right2 = self.controller_badge.authorize(|| {
                                                                borrow_resource_manager!(self.land)
                                                                    .mint_non_fungible(&land_id2, new_land2)
                                                            });
                                        
                                                            info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_size2, location2.clone());
                    
                                                            info!("You have paid {} tokens to divide the {} land into a {}m2 land on {} and a {}m2 land on {} with an attached building", amount, land_data.location, land_size1, location1.clone(), land_size2, location2.clone());
            
                                                            let move_proof = self.controller_badge.authorize(|| {
                                                                let move_badge = borrow_resource_manager!(self.move_badge)
                                                                .mint(dec!(1));

                                                                let move_proof = move_badge.create_proof();

                                                                borrow_resource_manager!(self.move_badge)
                                                                    .burn(move_badge);
                                                                borrow_resource_manager!(self.land)
                                                                    .burn(land_right);
                                                                borrow_resource_manager!(self.land_modify_badge)
                                                                    .burn(land_modify_badge);
                                                                    move_proof 
                                                            });
                    
                                                            (land_right1, land_right2, payment, move_proof)
                                                        }
                                                        _ => panic!("Wrong Oracle data!")
                                                    }
                                                }
                                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                                            }
                                        }
                                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                                    }
                                }
                            }
                        }
                    }   
                }
                _ => {panic!("Wrong land modify badge!")}
            }
        }

        /// This method is for authority to merge an existed real estate with an existed (non-building) land.
        /// Input: 
        /// - the land right1's NFT: Bucket("land_right")
        /// - land_right2: input the land right 2 NFT bucket.
        /// - the building NFT proof: 
        /// + If the land contain a building: Enum("Some", Proof("building_proof"))
        /// + If that's a barren land: Enum("None")
        /// - payment: the tax rate payment bucket.
        /// Output: 1 merged real estate with the attached NFTs. The location data of new real estate is equal to the first original real estate.
        pub fn merge_land(&mut self, land_right1: Bucket, land_right2: Bucket, building_proof: OptionProof, land_modify_badge: Bucket, mut payment: Bucket) -> (Bucket, Bucket, Proof) {

            assert!((land_modify_badge.resource_address()==self.land_modify_badge) 
            & (payment.resource_address()==self.token) 
            & (land_right1.resource_address() == self.land) 
            & (land_right2.resource_address() == self.land),
                "Wrong resource."
            );

            let data = land_modify_badge.non_fungible::<LandModify>().data();

            let land_modify_data = data.land_modify;

            let id = data.land_id;

            let amount = payment.amount();

            self.tax_vault.put(payment.take(self.rate));

            let land_id1 = land_right1.non_fungible::<Land>().id();

            let land_data1 = land_right1.non_fungible::<Land>().data();

            let land_id2 = land_right2.non_fungible::<Land>().id();

            let land_data2 = land_right2.non_fungible::<Land>().data();

            let (land_data1, land_id2, land_data2) = if land_id1 == id {

                (land_data1, land_id2, land_data2)

            } else if land_id2 == id {

                (land_data2, land_id1, land_data1)

            } else {panic!("Wrong land right's NFT provided")};

            match land_modify_data {

                LandModifyType::Merge(land2_id, building_id) => {

                    match building_id {

                        None => {

                            assert!(matches!(building_proof, OptionProof::None),
                                "Wrong building proof provided."
                            );
        
                            assert!((land_id2 == land2_id),
                                "Wrong land right's NFT provided"
                            );
                
                            let new_land = Land {
                                contain: None,
                                size: land_data1.size+land_data2.size,
                                location: land_data1.location.clone()
                            };
        
                            let land_id_merged: NonFungibleId = NonFungibleId::random();
        
                            let land_right_merged = self.controller_badge.authorize(|| {
                                borrow_resource_manager!(self.land)
                                    .mint_non_fungible(&land_id_merged, new_land)
                            });
        
                            info!("You have created a new land right's NFT of the {}m2 land on {}", land_data1.size+land_data2.size, land_data1.location.clone());
        
                            info!("You have paid {} tokens to merge the {}m2 land on {} and {}m2 land on {} into a {}m2 land on {}", amount, land_data1.size, land_data1.location.clone(), land_data2.size, land_data2.location.clone(), land_data1.size+land_data2.size, land_data1.location.clone());
        
                            let move_proof = self.controller_badge.authorize(|| {
                                let move_badge = borrow_resource_manager!(self.move_badge)
                                    .mint(dec!(1));

                                let move_proof = move_badge.create_proof();

                                borrow_resource_manager!(self.move_badge)
                                    .burn(move_badge);
                                borrow_resource_manager!(self.land)
                                    .burn(land_right1);
                                borrow_resource_manager!(self.land)
                                    .burn(land_right2);
                                borrow_resource_manager!(self.land_modify_badge)
                                    .burn(land_modify_badge);
        
                                    move_proof

                            });
        
                            return (land_right_merged, payment, move_proof)
        
                        }
        
                        Some(building_id) => {

                            match building_proof {

                                OptionProof::None => {panic!("Wrong building proof provided.")}

                                OptionProof::Some(building_proof) => {

                                    let building_proof_id = building_proof.non_fungible::<Building>().id();

                                    assert!(building_proof_id == building_id,
                                        "Wrong building proof provided."
                                    );

                                    assert!((land_id2 == land2_id),
                                        "Wrong land right's NFT provided"
                                    );
                        
                                    let new_land = Land {
                                        contain: Some(building_id),
                                        size: land_data1.size+land_data2.size,
                                        location: land_data1.location.clone()
                                    };
                
                                    let land_id_merged: NonFungibleId = NonFungibleId::random();
                
                                    let land_right_merged = self.controller_badge.authorize(|| {
                                        borrow_resource_manager!(self.land)
                                            .mint_non_fungible(&land_id_merged, new_land)
                                    });
                
                                    info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_data1.size+land_data2.size, land_data1.location.clone());
                
                                    info!("You have paid {} tokens to merge the {}m2 land on {} with an attached building and {}m2 land on {} into a {}m2 land on {} with an attached building", amount, land_data1.size, land_data1.location.clone(), land_data2.size, land_data2.location.clone(), land_data1.size+land_data2.size, land_data1.location.clone());
                
                                    let move_proof = self.controller_badge.authorize(|| {
                                        let move_badge = borrow_resource_manager!(self.move_badge)
                                            .mint(dec!(1));
        
                                        let move_proof = move_badge.create_proof();
        
                                        borrow_resource_manager!(self.move_badge)
                                            .burn(move_badge);
                                        borrow_resource_manager!(self.land)
                                            .burn(land_right1);
                                        borrow_resource_manager!(self.land)
                                            .burn(land_right2);
                                        borrow_resource_manager!(self.land_modify_badge)
                                            .burn(land_modify_badge);
                                            move_proof
                                    });
                
                                    return (land_right_merged, payment, move_proof)

                                }
                            }
                        }
                    }    
                }
                _ => panic!("Wrong land modify badge")
            }
        }

        /// This method is for authority to authorize a construction institute.
        /// Input: the institute's name, institute service's fee (tokens)
        /// Output: the institute's badge
        pub fn authorize_construction_institute(&mut self, name: String, fee: Decimal) -> Bucket {

            let institute_id = NonFungibleId::random();

            let institute = InstituteBadge {};

            let (institute_badge, institute_controller_badge) = self.controller_badge.authorize(|| {
                (borrow_resource_manager!(self.institute_badge)
                .mint_non_fungible(&institute_id, institute)
                ,borrow_resource_manager!(self.institute_controller_badge)
                    .mint(dec!(1)))
            });

            let institute_address = NonFungibleAddress::new(self.institute_badge, institute_id);

            let authority_address: ComponentAddress = Runtime::actor().component_address().unwrap();

            let institute_comp = RealEstateConstructionInstitute::new(institute_address, self.id_badge, authority_address, name.clone(), fee, institute_controller_badge, self.token, self.land, self.building, self.move_badge);
            self.real_estate_construction_institute.insert(institute_comp);

            info!("You have authorized {} construction institute with {} tokens fee per service", name, fee);

            return institute_badge

        }

        /// This method is for authority to authorize a real estate market place.
        /// Input: the market's name, market's fee (%).
        /// Output: the market's badge.
        pub fn authorize_marketplace(&mut self, name: String, fee: Decimal) -> Bucket {

            let host_id = NonFungibleId::random();

            let host = MarketHostBadge {};

            let (market_host_badge, market_controller_badge) = self.controller_badge.authorize(|| {
                (borrow_resource_manager!(self.market_host_badge)
                .mint_non_fungible(&host_id, host),
                borrow_resource_manager!(self.market_controller_badge)
                    .mint(dec!(1)))
            });

            let address = NonFungibleAddress::new(self.market_host_badge, host_id);

            let authority_address: ComponentAddress = Runtime::actor().component_address().unwrap();

            let market_comp = RealEstateMarketPlace::new(address, self.id_badge, authority_address, name.clone(), market_controller_badge, fee,self.land, self.building, self.token, self.move_badge);
            self.real_estate_market_place.insert(market_comp);

            info!("You have authorized {} market place with {} % fee per service", name, fee);

            return market_host_badge

        }

        pub fn edit_tax(&mut self, tax: Decimal) {

            self.tax = tax/dec!(100);

            info!("You have edited the tax of all market places into {} % per trade", tax);

        }

        pub fn edit_rate(&mut self, rate: Decimal) {

            self.rate = rate;

            info!("You have edited the tax rate of all institutes into {} tokens per service", rate);

        }

        pub fn collect_tax(&mut self) -> Bucket {

            info!("You have collected {} tokens tax.", self.tax_vault.amount());

            self.tax_vault.take_all()

        }

        pub fn deposit_tax(&mut self, tax: Bucket) {

            self.tax_vault.put(tax);

        }

        /// Read only method for utility
        pub fn tax(&self) -> Decimal {
            self.tax
        }

        /// Read only method for utility
        pub fn rate(&self) -> Decimal {
            self.rate
        }
    }
}