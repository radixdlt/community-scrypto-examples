use scrypto::prelude::*;

#[blueprint]
mod mod_purpleblood {

    struct PurpleBlood {

        sample_vault: Vault,

        // my own component address
        component_address: Option<ComponentAddress>,
    }

    impl PurpleBlood {
        pub fn instantiate() -> ComponentAddress {

            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "AristocratToken")
                .metadata("symbol", "AT")
                .mint_initial_supply(1000);
            
            let access_rules: AccessRules = AccessRules::new()
                .default(rule!(allow_all), LOCKED);

            let mut component: PurpleBloodComponent = Self {
                sample_vault: Vault::with_bucket(my_bucket),
                component_address: None,
            }
            .instantiate();
    
            component.add_access_check(access_rules);
                  
            /*
                globalize_with_owner badge binds the royalty settings to this badge,
                This component's owner-badge is a black-hole address, activly deactivating
                this royalty functions of this component.
            */

            let zero_address = [0u8; 26];
            let blackholebadge = 
                NonFungibleGlobalId::new(ResourceAddress::Normal(zero_address), 0u64.into());

            let globalized_component: ComponentAddress = component.globalize_with_owner(blackholebadge);
            let global_ref: PurpleBloodGlobalComponentRef = globalized_component.into();
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
            Method that gives free tokens 
         */
        pub fn free_token(&mut self) -> Bucket {
            info!("My balance is: {} AristocratToken. Now giving away a token!", self.sample_vault.amount());
            self.sample_vault.take(1)
        }
    }
}