use crate::user::*;
use scrypto::prelude::*;
use std::collections::HashMap;

blueprint! {
    struct TokenCreator {
        users: HashMap<ResourceAddress, UserComponent>,
    }

    impl TokenCreator {
        pub fn new() -> ComponentAddress {
            Self {
                users: HashMap::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn new_user(&mut self) -> Bucket {
            let (component, badge) = UserComponent::new_user();
            self.users.insert(badge.resource_address(), component);
            badge
        }

        pub fn create_token(
            &self,
            badge_address: ResourceAddress,
            metadata: HashMap<String, String>,
            initial_supply: Decimal,
            divisibility: u8
        ) -> Bucket {
            self.assert_account_exists(badge_address);
            let user = &self.users[&badge_address];
            user.create_token(
                metadata,
                initial_supply,
                divisibility
            )
        }

        pub fn mint(
            &self,
            badge_address: ResourceAddress,
            amount: Decimal,
            token_address: ResourceAddress,
        ) -> Bucket {
            self.assert_account_exists(badge_address);
            let user = &self.users[&badge_address];

            user.mint(amount, token_address)
        }

        pub fn burn(&self, badge_address: ResourceAddress, tokens: Bucket) {
            self.assert_account_exists(badge_address);
            let user = &self.users[&badge_address];

            user.burn(tokens);
        }

        pub fn account_exists(&self, badge_address: ResourceAddress) -> bool {
            self.users.contains_key(&badge_address)
        }

        pub fn assert_account_exists(&self, badge_address: ResourceAddress) {
            assert!(
                self.account_exists(badge_address),
                "This badge is not affiliated with an account."
            )
        }
    }
}
