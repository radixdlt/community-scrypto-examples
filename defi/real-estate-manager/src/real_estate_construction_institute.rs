//! [RealEstateConstructionInstitute] is the blueprint for construction institutes to manage real estate construction rights by NFT.
//! Construction institutes can use this blueprint to manage building NFTs, construction right's NFT through this blueprint.
//! Citizens can request construction through this blueprint.
//! This blueprint also contain a taxing mechanism for any construction authorization service.

use scrypto::prelude::*;
use crate::real_estate_service::*;
use crate::utility::*;

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
/// Authorities, organizations in charge of providing construction badge also need to make sure the change won't cause bad effect to society.
/// Struct: 
/// land_id: the id of the land which will make the construction
/// construction: the construction type
#[derive(NonFungibleData)]
pub struct Construction {
    land_id: NonFungibleId,
    construction: ConstructionType
}

blueprint! {

    struct RealEstateConstructionInstitute {

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
        /// Fee rate paid on real estate service for construction institute (tokens)
        fee: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Construction Institute's fee vault
        fee_vault: Vault,
        /// Construction badge NFT resource address.
        construction_badge: ResourceAddress,
        /// Construction request badge NFT resource address.
        request_badge: ResourceAddress,
        /// Request book, contain in-queue construction request of citizens. Struct: Request Id, (the requested construction land id, the requested construction data)
        request_book: HashMap<NonFungibleId, (NonFungibleId, ConstructionType)>,
        /// Construction badge vault, contain solved construction request of citizens.
        construction_badge_vault: Vault,
        /// Request id counter
        request_counter: u64

    }

    impl RealEstateConstructionInstitute {
        
        /// This function will create new Real Estate Construction Institute component.
        /// Input: 
        /// - name: institute name.
        /// - controller badge: the institute component controller badge.
        /// - fee: institute service fee.
        /// - medium token: the token used for trade.
        /// - land: land resource address.
        /// - building: building resource address.
        /// - real estate authority: the authority that authorized the market.
        /// Output: Component address and the market host badge
        pub fn new(construction_authority_badge: NonFungibleAddress, id_badge: ResourceAddress, authority_address: ComponentAddress, name: String, fee: Decimal, controller_badge: Bucket, medium_token: ResourceAddress, land: ResourceAddress, building: ResourceAddress, move_badge: ResourceAddress) -> ComponentAddress {

            let construction_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone()+" Construction Right's Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let request_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", name+" Construction Request Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("take_fee", rule!(require(construction_authority_badge.clone())))
                .method("edit_fee", rule!(require(construction_authority_badge.clone())))
                .method("authorize_construction", rule!(require(construction_authority_badge)))
                .default(rule!(require(id_badge)));

            let comp = Self {

                controller_badge: Vault::with_bucket(controller_badge),
                authority_address: authority_address,
                move_badge: move_badge,
                building: building,
                land: land,
                fee: fee,
                token: medium_token,
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

            return comp

        }

        /// This is method for citizens to make new construction request.
        /// Input: 
        /// - the citizen's real estate proof: 
        /// + If the real estate doesn't contain a building: Enum("Land", Proof("land_proof"));
        /// + If the real estate contain a building: Enum("LandandBuilding", Proof("land_proof"), Proof("building_proof"));
        /// - the requested construction information: 
        /// + If citizen want to construct a building: Enum("ConstructBuilding", Struct(Decimal("${building_size}"), ${building_floor}u32)
        /// + If citizen want to demolish a building: Enum("DemolishBuilding")
        /// Output: the request badge
        pub fn new_construction_request(&mut self, real_estate_proof: RealEstateProof, construction: ConstructionType) -> Bucket {
            
            let (land_id, _, real_estate_data) = get_real_estate_data(real_estate_proof, self.land, self.building);

            let (construct_type, location) = match construction {

                ConstructionType::ConstructBuilding(_) => {
                    let location = match real_estate_data.clone() {
                        RealEstateData::Land(_, location) => location,
                        RealEstateData::LandandBuilding(_,_,_,_) => panic!("You cannot construct a building on an already existed building.")
                    };

                    ("construct building", location)
                },

                ConstructionType::DemolishBuilding => {
                    let location = match real_estate_data.clone() {
                        RealEstateData::Land(_, _) => panic!("You cannot demolish a barren land."),
                        RealEstateData::LandandBuilding(_, location,_,_) => location
                    };

                    ("demolish building", location)
            }
            };

            let new_construction_request = Request {};

            let request_id: NonFungibleId = NonFungibleId::from_u64(self.request_counter);

            let request_badge = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.request_badge)
                    .mint_non_fungible(&request_id, new_construction_request)
            });

            self.request_book.insert(request_id.clone(), (land_id, construction));

            info!("You have created a new {} request no.{} on the {} land.", construct_type, request_id, location);

            self.request_counter += 1;

            request_badge

        }

        /// This method is for institutes to authorize a construction request
        /// Input: 
        /// - the request id: ${request_id}u64
        /// - the oracle data show whether this construction is harmful or not: Enum("IsOk", ${ok_or_not})
        /// Output: None
        /// This method put the authorized construction NFT on the component vault if passed.
        pub fn authorize_construction(&mut self, id: u64, is_ok: Feasible) {

            assert!(matches!(is_ok, Feasible::IsOk(_)),
                "Wrong Oracle data."
            );

            let request_id = NonFungibleId::from_u64(id);

            let result = self.request_book.remove(&request_id);

            assert!(result.is_some(),
                "The request book doesn't contain this request id."
            );

            if matches!(is_ok, Feasible::IsOk(true)) {

                let (land_id, construction) = result.unwrap();

                let construction_data = Construction {
                    land_id: land_id,
                    construction: construction
                };

                self.controller_badge.authorize(|| {
                    let construction_right = borrow_resource_manager!(self.construction_badge)
                        .mint_non_fungible(&request_id, construction_data);
                    self.construction_badge_vault.put(construction_right);
                });

                info!("You have authorized the construction request no.{}", request_id);

            } else {
                error!("The construction no.{} is harmful, rejected the construction request.", request_id);
            }
                
        }

        /// This method is for citizens to get their construct badge after authorized.
        /// Input: the request badge: Bucket("request_badge")
        /// Output: the request result returned: if passed > return construction right badge.
        pub fn get_construction_badge(&mut self, request_badge: Bucket) -> Option<Bucket> {

            assert!(request_badge.resource_address() == self.request_badge,
                "Wrong resource!"
            );

            let request_id = request_badge.non_fungible::<Request>().id();

            assert!(!self.request_book.contains_key(&request_id),
                "Construction institute haven't reviewed your request yet"
            );

            self.controller_badge
                .authorize(|| { 
                    borrow_resource_manager!(self.request_badge)
                        .burn(request_badge);
                });

            if self.construction_badge_vault.non_fungible_ids().contains(&request_id) {
                info!("Your construction request no.{} has passed.", request_id);
                Some(self.construction_badge_vault.take_non_fungible(&request_id))
            } else {
                info!("Your construction request no.{} has been rejected.", request_id);
                None
            }
        }

        /// This method is for citizens to construct new building to an existed land.
        /// Input: 
        /// - the land right's NFT proof: Proof("land_right")
        /// - the construction badge: Bucket("construction_badge")
        /// - the payment bucket: Bucket("payment")
        /// Output: The building right's NFT of that land and payment changes.
        pub fn construct_new_building(&mut self, land_right: Proof, construction_badge: Bucket, mut payment: Bucket) -> (Bucket, Bucket, Proof) {

            assert!((land_right.resource_address()==self.land) & (construction_badge.resource_address()==self.construction_badge) & (payment.resource_address()==self.token),
                "Wrong resource."
            );

            let authority: RealEstateService = self.authority_address.into();

            let rate = authority.rate();

            assert!(payment.amount() >= rate+self.fee, "Payment is not enough");

            let id = land_right.non_fungible::<Land>().id();

            let mut data: Land = land_right.non_fungible().data();

            let construct_data = construction_badge.non_fungible::<Construction>().data();

            assert!(construct_data.land_id==id, "Wrong land proof provided");

            let (new_building, size, floor) = match construct_data.construction {

                ConstructionType::ConstructBuilding(construct_building) => {
                    (Building {
                        size: construct_building.building_size,
                        floor: construct_building.building_floor
                    }, construct_building.building_size, construct_building.building_floor)
                }

                _ => panic!("Wrong construction badge provided.")

            };

            let building_id: NonFungibleId = NonFungibleId::random();

            data.contain = Some(building_id.clone());

            let location = data.location.clone();

            let (building_right, move_proof) = self.controller_badge
                .authorize(|| {

                    let move_badge = borrow_resource_manager!(self.move_badge)
                        .mint(dec!(1));

                    let move_proof = move_badge.create_proof();

                    borrow_resource_manager!(self.move_badge)
                        .burn(move_badge);

                    borrow_resource_manager!(self.construction_badge)
                    .burn(construction_badge);

                    land_right.non_fungible().update_data(data);

                    (borrow_resource_manager!(self.building)
                    .mint_non_fungible(&building_id, new_building),move_proof)

                });

            info!("You have paid {} tokens to mint a new building right's NFT of the {}m2, {} floor building attached to the land on {} according to construction data",rate+self.fee, size, floor, location);

            self.fee_vault.put(payment.take(self.fee));
            authority.deposit_tax(payment.take(rate));

            return (building_right, payment, move_proof)

        }

        /// This method is for citizens to demolish an existed building.
        /// Input: 
        /// - the land right's NFT proof: Proof("land_right")
        /// - the building right's NFT: Bucket("building_right")
        /// - the construction badge: Bucket("construction_badge")
        /// - the payment bucket: Bucket("payment")
        /// Output: The payment changes.
        pub fn demolish_building(&mut self, land_right: Proof, building: Bucket, construction_badge: Bucket, mut payment: Bucket) -> Bucket {

            assert!((land_right.resource_address()==self.land) & (building.resource_address()==self.building) & (construction_badge.resource_address()==self.construction_badge) & (payment.resource_address()==self.token),
                "Wrong resource."
            );

            let authority: RealEstateService = self.authority_address.into();

            let rate = authority.rate();

            assert!(payment.amount()>=rate+self.fee, "Payment is not enough");

            let id = land_right.non_fungible::<Land>().id();

            let mut land_data: Land = land_right.non_fungible().data();

            let building_id = building.non_fungible::<Building>().id();

            assert!(land_data.contain.unwrap() == building_id,
                    "This land doesn't contain the building from provided building right."
                );

            let construct_data = construction_badge.non_fungible::<Construction>().data();

            assert!(construct_data.land_id==id, "Wrong land proof provided");

            match construct_data.construction {

                ConstructionType::DemolishBuilding => {}

                _ => panic!("Wrong construction badge provided.")

            };

            land_data.contain = None;

            let location = land_data.location.clone();

            self.controller_badge
                .authorize(|| {

                    borrow_resource_manager!(self.construction_badge)
                    .burn(construction_badge);

                    land_right.non_fungible().update_data(land_data);

                    borrow_resource_manager!(self.building)
                    .burn(building)

                });

            info!("You have paid {} tokens to burn the building right's NFT of the building attached to the land on {} according to construction data", rate+self.fee, location);

            self.fee_vault.put(payment.take(self.fee));
            authority.deposit_tax(payment.take(rate));

            return payment

        }

        pub fn take_fee(&mut self) -> Bucket {

            info!("You have collected {} tokens institute fee.", self.fee_vault.amount());
            self.fee_vault.take_all()

        }

        pub fn edit_fee(&mut self, fee: Decimal) {

            info!("You have edited fee of the institute into {} tokens per service", fee);
            self.fee = fee
        }
    }
}