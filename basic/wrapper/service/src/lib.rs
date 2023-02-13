use scrypto::prelude::*;

blueprint! {
    struct Service {
        my_range: u32,
        my_random: [u8;16],
    }

    impl Service {
        // Implement the functions and methods which will manage those resources and data
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate() -> (ComponentAddress, Bucket) {

            // creating our admin badges
            // use one badge for internal admin stuff, and send one to instantiate wallet address.
            let my_admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin Badge for Service")
                .initial_supply(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));

            // set the access rules for the Admin-only and internal functions.
            let access_rules = AccessRules::new()
                .method("awesome_service", admin_rule.clone(), AccessRule::DenyAll)
//                .method("generate_random", AccessRule::DenyAll, AccessRule::DenyAll)
                .default(AccessRule::AllowAll, AccessRule::DenyAll);
                
            let mut component = Self {
                my_range: u32::MAX,
                my_random: Runtime::generate_uuid().to_le_bytes(),
//                my_random: [0u8; 16],
            }
          
            .instantiate();
            component.add_access_check(access_rules);
            let component = component.globalize();

            // Return the instantiated component and the admin badge that was just minted
            (component, my_admin_badge)
        }
        /*
            Attempts to generate a random number generator.
            private function to this blueprint.
        */
        fn generate_random(&mut self) -> u128 {
            let mut data = self.my_random.as_ref().to_vec();
            // this random function will work less deterministic when time can be obtained in ns.
            let my_time = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            self.my_range = self.my_range.wrapping_add(1);
            data.extend(self.my_range.to_le_bytes());
            data.extend(my_time.to_le_bytes());
            self.my_random = sha256_twice(data).lower_16_bytes();
            u128::from_le_bytes(self.my_random)
        }
        /*
            Restricted access service
        */
        pub fn awesome_service(&mut self) -> i8 {
            loop{
                let mut random:u128 = self.generate_random();
//                let mut random:u128 = Runtime::generate_uuid();
                while random > 0{
                    let myval = random & 0x7;
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

