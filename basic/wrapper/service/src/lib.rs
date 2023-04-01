use scrypto::prelude::*;

#[blueprint]
mod mod_service {
    struct Service {
        my_range: u32,
        random_seed: [u8;16],
       // resourceaddress of the admin badge
        admin_badge_address: ResourceAddress,

    }

    impl Service {
        // Implement the functions and methods which will manage those resources and data
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate() -> (ComponentAddress, Bucket) {

            // creating our admin badges
            // use one badge for internal admin stuff, and send one to instantiate wallet address.
            let my_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin Badge for Service")
                .mint_initial_supply(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));

            // set the access rules for the Admin-only and internal functions.
            let access_rules:AccessRulesConfig = AccessRulesConfig::new()
                .method("awesome_service", admin_rule.clone(), AccessRule::DenyAll)
                .default(AccessRule::AllowAll, AccessRule::DenyAll);
                
            let component: ServiceComponent = Self {
                my_range: u32::MAX,
                random_seed: scrypto::crypto::hash(Runtime::transaction_hash().to_vec()).lower_16_bytes(),
                admin_badge_address: my_admin_badge.resource_address(),
            }
          
            .instantiate();
            let component: ComponentAddress = component.globalize_with_access_rules(access_rules);

            // Return the instantiated component and the admin badge that was just minted
            (component, my_admin_badge)
        }
        /*
            Attempts to generate a random number generator.
            private function to this blueprint.
        */
        fn generate_random(&mut self) -> u128 {
            let mut data: Vec<u8> = self.random_seed.as_ref().to_vec();
            let my_time: i64 = Clock::current_time(scrypto::blueprints::clock::TimePrecision::Minute).seconds_since_unix_epoch;
            self.my_range = self.my_range.wrapping_add(1);
            data.extend(self.my_range.to_le_bytes());
            data.extend(my_time.to_le_bytes());
            data.extend(Runtime::transaction_hash().to_vec());
            self.random_seed = scrypto::crypto::hash(data).lower_16_bytes();
            u128::from_le_bytes(self.random_seed)
        }

        /*
            Restricted access service
            a call to this method requires System Level Authentication
        */
        pub fn awesome_service(&mut self) -> i8 {
            loop{
                let mut random:u128 = self.generate_random();
//                let mut random:u128 = Runtime::generate_uuid();
                while random > 0{
                    let myval: u128 = random & 0x7;
                    // if 0x6 or 0x7 throw away result and redo check on 3 new bits.
                    if myval < 0x6{ 
                        return (myval+1) as i8 ;
                    }
                    // get 3 new bits but shift 4 because 128/3 != integer
                    random = random >> 4; 
                }
            }
        }

        /*
            Restricted access service
            a call to this method requires Application Level Authentication
        */
        pub fn second_service(&mut self, auth: Proof) -> i8 {

            let _validated_proof: ValidatedProof = auth.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.admin_badge_address)
            ).expect("invalid proof");

            loop{
                let mut random:u128 = self.generate_random();
                while random > 0{
                    let myval: u128 = random & 0x7;
                    // if 0x6 or 0x7 throw away result and redo check on 3 new bits.
                    if myval < 0x6{ 
                        return (myval+1) as i8 ;
                    }
                    // get 3 new bits but shift 4 because 128/3 != integer
                    random = random >> 4; 
                }
            }
        }

        /*
            Free access serive. 
        */
        pub fn deep_thought(&mut self) -> i8 {

            42

        }
    }
}
