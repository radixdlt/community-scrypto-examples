use scrypto::prelude::*;

// Enum describing the type of real estate
#[derive(ScryptoSbor, PartialEq, Display, Debug)]
pub enum PropertyType {
    SingleFamily,
    MultiFamily,
    Office,
    Industrial,
}

// Struct describing the data of a real estate nft
#[derive(ScryptoSbor, NonFungibleData, Debug)]
pub struct RealEstateData {
    #[mutable]
    owner_name: String,

    #[mutable]
    owner_address: ComponentAddress,

    #[mutable]
    property_type: PropertyType,

    #[mutable]
    area_per_floor: Decimal,

    #[mutable]
    number_of_floors: Decimal,

    #[mutable]
    annual_rent: Decimal,

    #[mutable]
    property_value: Decimal,

    #[mutable]
    cap_rate: Decimal,

    #[mutable]
    monthly_rent: Decimal,

    #[mutable]
    monthly_rent_per_square_foot: Decimal,

    #[mutable]
    annual_rent_per_square_foot: Decimal,

    #[mutable]
    price_per_square_foot: Decimal,
}

#[blueprint]
mod tokenize_real_estate {
    // Struct describing the Tokenize Real Estate component
    struct TokenizeRealEstate {
        admin_badge: ResourceAddress,
        xrd_vault: Vault,
        mint_nft_price: Decimal,
        update_nft_price: Decimal,
        mint_real_estate_nft_badge_vault: Vault,
        real_estate_nft: ResourceAddress,
        nft_id_counter: u64,
    }
    // Implementation of the Tokenize Real Estate component
    impl TokenizeRealEstate {
        pub fn instantiate_tokenize_real_estate(
            mint_nft_price: Decimal,
            update_nft_price: Decimal,
        ) -> (ComponentAddress, Bucket) {
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("symbol", "AB")
                .divisibility(0)
                .mint_initial_supply(1);

            let mint_real_estate_nft_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Mint Real Estate NFT Badge")
                .metadata("symbol", "MRENB")
                .divisibility(0)
                .mint_initial_supply(1);

            let real_estate_nft: ResourceAddress =
                ResourceBuilder::new_integer_non_fungible::<RealEstateData>()
                    .metadata("name", "Real Estate NFT")
                    .mintable(
                        rule!(require(mint_real_estate_nft_badge.resource_address())),
                        LOCKED,
                    )
                    .burnable(rule!(allow_all), LOCKED)
                    .updateable_non_fungible_data(
                        rule!(require(mint_real_estate_nft_badge.resource_address())),
                        LOCKED,
                    )
                    .create_with_no_initial_supply();

            let access_rule: AccessRule = rule!(require(admin_badge.resource_address()));

            let access_rules = AccessRulesConfig::new()
                .method("claim_xrd", access_rule.clone(), AccessRule::DenyAll)
                .default(rule!(allow_all), AccessRule::DenyAll);

            let component = Self {
                admin_badge: admin_badge.resource_address(),
                xrd_vault: Vault::new(RADIX_TOKEN),
                mint_real_estate_nft_badge_vault: Vault::with_bucket(mint_real_estate_nft_badge),
                real_estate_nft,
                mint_nft_price,
                update_nft_price,
                nft_id_counter: 0,
            }
            .instantiate();

            let tokenize_component = component.globalize_with_access_rules(access_rules);

            return (tokenize_component, admin_badge);
        }
        // Method to mint a real estate nft
        pub fn mint_nft(
            &mut self,
            mut payment: Bucket,
            owner_name: String,
            owner_address: ComponentAddress,
            property_type: PropertyType,
            area_per_floor: Decimal,
            number_of_floors: Decimal,
            annual_rent: Decimal,
            property_value: Decimal,
        ) -> (Bucket, Bucket) {
            assert!(payment.resource_address() == RADIX_TOKEN);

            assert!(payment.amount() >= self.mint_nft_price);

            assert!(
                property_type == PropertyType::SingleFamily
                    || property_type == PropertyType::MultiFamily
                    || property_type == PropertyType::Office
                    || property_type == PropertyType::Industrial
            );

            assert!(area_per_floor > Decimal::from(0));

            assert!(number_of_floors > Decimal::from(0));

            assert!(annual_rent > Decimal::from(0));

            assert!(property_value > Decimal::from(0));

            self.xrd_vault.put(payment.take(self.mint_nft_price));

            let total_area: Decimal = area_per_floor * number_of_floors;

            let cap_rate: Decimal = annual_rent / property_value;

            let monthly_rent: Decimal = annual_rent / 12;

            let monthly_rent_per_square_foot: Decimal = monthly_rent / total_area;

            let annual_rent_per_square_foot: Decimal = annual_rent / total_area;

            let price_per_square_foot: Decimal = property_value / total_area;

            let tokenized_real_estate_nft = RealEstateData {
                owner_name,
                owner_address,
                property_type,
                area_per_floor,
                number_of_floors,
                annual_rent,
                property_value,
                cap_rate,
                monthly_rent,
                monthly_rent_per_square_foot,
                annual_rent_per_square_foot,
                price_per_square_foot,
            };

            let nft_bucket = self.mint_real_estate_nft_badge_vault.authorize(|| {
                borrow_resource_manager!(self.real_estate_nft).mint_non_fungible(
                    &NonFungibleLocalId::Integer(self.nft_id_counter.into()),
                    tokenized_real_estate_nft,
                )
            });
            self.nft_id_counter += 1;

            (nft_bucket, payment)
        }
        // Method to update the number data of a real estate nft
        pub fn update_nft_number_data(
            &mut self,
            nft: Bucket,
            mut payment: Bucket,
            update_data_field: String,
            update_data_value: Decimal,
        ) -> (Bucket, Bucket) {
            assert!(
                nft.resource_address() == self.real_estate_nft,
                "Must be a valid Real Estate NFT"
            );

            assert!(
                nft.amount() == Decimal::from(1),
                "We can only update one NFT at a time"
            );

            assert!(payment.resource_address() == RADIX_TOKEN);

            assert!(payment.amount() >= self.update_nft_price);

            assert!(
                update_data_field == "area_per_floor"
                    || update_data_field == "number_of_floors"
                    || update_data_field == "annual_rent"
                    || update_data_field == "property_value"
            );

            assert!(update_data_value > Decimal::from(0));

            self.xrd_vault.put(payment.take(self.update_nft_price));

            let nft_local_id: NonFungibleLocalId = nft.non_fungible_local_id();
            let mut resource_manager: ResourceManager =
                borrow_resource_manager!(self.real_estate_nft);
            let mut fields: Vec<(String, Decimal)> = Vec::new();

            self.mint_real_estate_nft_badge_vault.authorize(|| {
                resource_manager.update_non_fungible_data(
                    &nft_local_id,
                    &update_data_field,
                    update_data_value,
                );
            });

            let non_fungible_data: RealEstateData = nft.non_fungible().data();

            match &update_data_field[..] {
                "area_per_floor" | "number_of_floors" | "annual_rent" | "property_value" => {
                    let total_area =
                        non_fungible_data.area_per_floor * non_fungible_data.number_of_floors;
                    let monthly_rent = non_fungible_data.annual_rent / Decimal::from(12);

                    fields.push((
                        "cap_rate".to_string(),
                        non_fungible_data.annual_rent / non_fungible_data.property_value,
                    ));
                    fields.push(("monthly_rent".to_string(), monthly_rent));
                    fields.push((
                        "monthly_rent_per_square_foot".to_string(),
                        monthly_rent / total_area,
                    ));
                    fields.push((
                        "annual_rent_per_square_foot".to_string(),
                        non_fungible_data.annual_rent / total_area,
                    ));
                    fields.push((
                        "price_per_square_foot".to_string(),
                        non_fungible_data.property_value / total_area,
                    ));
                }

                _ => {}
            }

            for (field_name, field_value) in fields {
                self.mint_real_estate_nft_badge_vault.authorize(|| {
                    resource_manager.update_non_fungible_data(
                        &nft_local_id,
                        &field_name,
                        field_value,
                    );
                });
            }

            (nft, payment)
        }
        // Method to update the owner data of a real estate nft
        pub fn update_nft_owner_data(
            &mut self,
            nft: Bucket,
            mut payment: Bucket,
            update_owner_name: String,
            update_owner_component_address: ComponentAddress,
        ) -> (Bucket, Bucket) {
            assert!(update_owner_name.len() > 0);

            assert!(
                nft.resource_address() == self.real_estate_nft,
                "Must be a valid  Real Estate NFT"
            );

            assert!(
                nft.amount() == Decimal::from(1),
                "We can only update one NFT at a time"
            );

            assert!(payment.resource_address() == RADIX_TOKEN);

            assert!(payment.amount() >= self.update_nft_price);

            self.xrd_vault.put(payment.take(self.update_nft_price));

            let nft_local_id: NonFungibleLocalId = nft.non_fungible_local_id();

            let mut resource_manager: ResourceManager =
                borrow_resource_manager!(self.real_estate_nft);

            self.mint_real_estate_nft_badge_vault.authorize(|| {
                resource_manager.update_non_fungible_data(
                    &nft_local_id,
                    "owner_name",
                    update_owner_name,
                );
            });

            self.mint_real_estate_nft_badge_vault.authorize(|| {
                resource_manager.update_non_fungible_data(
                    &nft_local_id,
                    "owner_address",
                    update_owner_component_address,
                );
            });

            (nft, payment)
        }
        // Method to update the property type data of a real estate nft
        pub fn update_nft_property_type_data(
            &mut self,
            nft: Bucket,
            mut payment: Bucket,
            update_property_type: PropertyType,
        ) -> (Bucket, Bucket) {
            assert!(
                nft.resource_address() == self.real_estate_nft,
                "Must be a valid  Real Estate NFT"
            );

            assert!(
                nft.amount() == Decimal::from(1),
                "We can only update one NFT at a time"
            );

            assert!(payment.resource_address() == RADIX_TOKEN);

            assert!(payment.amount() >= self.update_nft_price);

            self.xrd_vault.put(payment.take(self.update_nft_price));

            let nft_local_id: NonFungibleLocalId = nft.non_fungible_local_id();

            let mut resource_manager: ResourceManager =
                borrow_resource_manager!(self.real_estate_nft);

            self.mint_real_estate_nft_badge_vault.authorize(|| {
                resource_manager.update_non_fungible_data(
                    &nft_local_id,
                    "property_type",
                    update_property_type,
                );
            });

            (nft, payment)
        }
        // Method to permanently delete a real estate nft
        pub fn delete_nft(&mut self, real_estate_nft: Bucket) {
            assert!(real_estate_nft.resource_address() == self.real_estate_nft);

            real_estate_nft.burn()
        }
        // Method to view the data of a real estate nft
        pub fn view_nft_data(&self, nft_proof: Proof) {
            assert!(nft_proof.resource_address() == self.real_estate_nft);

            let auth = nft_proof
                .validate_proof(self.real_estate_nft)
                .expect("Invalid NFT Proof");

            let nft: NonFungible<RealEstateData> = auth.non_fungible();

            let nft_data: RealEstateData = nft.data();

            info!("Current NFT Data: {:?}", nft_data);
        }
        // Method for the owner of the component to claim the xrd
        pub fn claim_xrd(&mut self) -> Bucket {
            info!("{} XRD Claimed", self.xrd_vault.amount());

            self.xrd_vault.take_all()
        }
    }
}
