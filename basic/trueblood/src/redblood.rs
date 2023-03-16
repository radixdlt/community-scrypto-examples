use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct NFData {}

#[blueprint]
mod mod_redblood {

    struct RedBlood {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault,

        // my own component address
        component_address: Option<ComponentAddress>,
    }

    impl RedBlood {

        pub fn instantiate() -> ComponentAddress {

            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "PeasantToken")
                .metadata("symbol", "PT")
                .mint_initial_supply(1000);
            
            let access_rules: AccessRules = AccessRules::new()
                .default(rule!(allow_all), LOCKED);

            let mut component: RedBloodComponent = Self {
                sample_vault: Vault::with_bucket(my_bucket),
                component_address: None,
            }
            .instantiate();
    
            component.add_access_check(access_rules);
                  
            // Storing the component address of this instantiation
            let globalized_component: ComponentAddress = component.globalize();
            let global_ref: RedBloodGlobalComponentRef = globalized_component.into();
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
            but calling could cost both Package and Component Royalties.
         */
        pub fn free_token(&mut self) -> Bucket {
            info!("My balance is: {} PeasantToken. Now giving away a token!", self.sample_vault.amount());
            self.sample_vault.take(1)
        }
    }
}