use scrypto::prelude::*;
use std::collections::HashMap;
use AccessRule:: {AllowAll, DenyAll};

blueprint! {
    struct User {
        resources: Vec<ResourceAddress>,
    }

    impl User {
        pub fn new_user() -> (UserComponent, Bucket) {
            let badge = ResourceBuilder::new_fungible()
                .metadata("name", "Token manager badge")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            let mut component = Self {
                resources: Vec::new(),
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("mint", rule!(require(badge.resource_address())), DenyAll)
                .default(AllowAll, DenyAll);

            component.add_access_check(access_rules);

            (component, badge)
        }

        pub fn create_token(
            &mut self,
            metadata: HashMap<String, String>,
            initial_supply: Decimal,
            divisibility: u8
        ) -> Bucket {
            let mut builder = ResourceBuilder::new_fungible();
            for (key, value) in metadata {
                builder = builder.metadata(key, value);
            }

            let tokens = builder
                .divisibility(divisibility)
                .initial_supply(initial_supply);

            self.resources.push(tokens.resource_address());
            tokens
        }

        pub fn mint(&self, amount: Decimal, token_address: ResourceAddress) -> Bucket {
            borrow_resource_manager!(token_address).mint(amount)
        }

        pub fn burn(&self, tokens: Bucket) {
            borrow_resource_manager!(tokens.resource_address());
        }
    }
}
