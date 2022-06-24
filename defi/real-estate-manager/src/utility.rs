use scrypto::prelude::*;
use crate::real_estate_service::*;

pub fn assert_land_proof(land_right: Proof, land: ResourceAddress) -> (NonFungibleId, Land) {

    assert!(land_right.resource_address()==land,
        "Wrong resource."
    );

    let land_data = land_right.non_fungible::<Land>().data();

    assert!(land_data.contain.is_none(),
        "This land contain a building, you should also input the building right's NFT."
    );

    let land_id = land_right.non_fungible::<Land>().id();

    land_right.drop();

    return (land_id, land_data)

}

pub fn assert_landandbuilding_proof(land_right: Proof, building_right: Proof, land: ResourceAddress, building: ResourceAddress) -> (NonFungibleId, Land, NonFungibleId, Building) {

    assert!((land_right.resource_address()==land) & (building_right.resource_address() == building),
                "Wrong resource."
            );

            let building_id = building_right.non_fungible::<Building>().id();

            let building_data: Building = building_right.non_fungible().data();

            let land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.clone().unwrap() == building_id,
                "This land doesn't contain the building from provided building right."
            );
            
            let land_id = land_right.non_fungible::<Land>().id();

    land_right.drop(); building_right.drop();

    return (land_id, land_data, building_id, building_data)

}

/// A utility function to get real estate data from a real estate proof, also assert if the real estate proof is right resources or not.
pub fn get_real_estate_data(real_estate: RealEstateProof, land: ResourceAddress, building: ResourceAddress) -> (NonFungibleId, Option<NonFungibleId>, RealEstateData) {

    match real_estate {

        RealEstateProof::Land(land_right) => {

            let (land_id, land_data) = assert_land_proof(land_right, land);

            return  (land_id, None, RealEstateData::Land(land_data.size, land_data.location))

        }

        RealEstateProof::LandandBuilding(land_right, building_right) => {

            let (land_id, land_data, building_id, building_data) = assert_landandbuilding_proof(land_right, building_right, land, building);

            return (land_id, Some(building_id), RealEstateData::LandandBuilding(land_data.size, land_data.location, building_data.size, building_data.floor))

        }
    }
}
