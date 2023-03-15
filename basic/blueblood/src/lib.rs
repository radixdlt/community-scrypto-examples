use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct NFData {}

#[blueprint]
mod mod_blood {

    struct BlueBlood {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault,

        // my own component address
        component_address: Option<ComponentAddress>,
    }

    impl BlueBlood {
        /*
            This function sets the royalty for the blueprint, is affects
            all instantiated (existing and future) components.
            The u32 value acts like a boolean on/off switch 
            The royalty seems to be fixed to 10^-7 XRD.

            This function needs to be called with the owner-badge used while publishing
            using the --proofs arguments
         */
        pub fn set_blueprint_royalty(u32bool: u32) {
            borrow_package!(Runtime::package_address()).set_royalty_config(BTreeMap::from([(
                "BlueBlood".to_owned(),
                RoyaltyConfigBuilder::new()
                    .add_rule("free_token", u32bool)
                    .default(0u32),
            )]));
        }

        pub fn instantiate(badge: NonFungibleGlobalId) -> ComponentAddress {

            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "KingToken")
                .metadata("symbol", "KT")
                .mint_initial_supply(1000);
            
            let access_rules: AccessRules = AccessRules::new()
                .default(rule!(allow_all), LOCKED);

            let mut component: BlueBloodComponent = Self {
                sample_vault: Vault::with_bucket(my_bucket),
                component_address: None,
            }
            .instantiate();
    
            component.add_access_check(access_rules);
                  
            // Storing the component address of this instantiation
            let globalized_component: ComponentAddress = component.globalize_with_owner(badge);
            let global_ref: BlueBloodGlobalComponentRef = globalized_component.into();
            global_ref.set_component_address(globalized_component); 

            globalized_component

        }

        /*
            Method to one time set the ComponentAddress of the instantiation.
         */
        pub fn set_component_address(&mut self, address: ComponentAddress) {
            match self.component_address {
                None => self.component_address = Some(address),
                Some(_) => panic!()
            }
        }

        /*
            This function needs to be called with the badge used for globalization
            using the --proofs
            The u32 value acts like a boolean on/off switch 
         */
        pub fn set_component_royalty(address: ComponentAddress, u32bool: u32){
            borrow_component!(address).set_royalty_config(
                        RoyaltyConfigBuilder::new()
                            .add_rule("free_token", u32bool)
                            .default(0u32),
                );
        }

        /*
            This function needs to be called with the badge used for globalization
            using the --proofs
         */
        pub fn claim_component_royalty(address: ComponentAddress) -> Bucket {
            borrow_component!(address).claim_royalty()
        }

        /*
            Method that gives free tokens 
            but calling could cost both Package and Component Royalties.
         */
        pub fn free_token(&mut self) -> Bucket {
            info!("My balance is: {} KingToken. Now giving away a token!", self.sample_vault.amount());
            self.sample_vault.take(1)
        }
    }
}