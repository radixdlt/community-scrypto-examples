use scrypto::prelude::*;

blueprint! {
    struct Escrow {
        token_a: Vault,
        token_b: Vault,
        account_a_badge: ResourceDef,
        account_b_badge: ResourceDef,
        account_a_accepted: bool,
        account_b_accepted: bool,
        trade_canceled: bool
    }

    impl Escrow {
        pub fn new(token_a_address: Address, token_b_address: Address) -> (Component, Bucket, Bucket) {
            // Create the badges that will allow the component to authenticate the users
            let account_a_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE A")
                .new_badge_fixed(1);

            let account_b_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE B")
                .new_badge_fixed(1);

            let account_a_badge_resource = account_a_badge.resource_def();
            let account_b_badge_resource = account_b_badge.resource_def();

            let component = Self {
                token_a: Vault::new(token_a_address),
                token_b: Vault::new(token_b_address),
                account_a_badge: account_a_badge_resource,
                account_b_badge: account_b_badge_resource,
                account_a_accepted: false,
                account_b_accepted: false,
                trade_canceled: false
            }
            .instantiate();

            (component, account_a_badge, account_b_badge)
        }

        // Allow the users to put the tokens they want to trade inside the component's vault
        #[auth(account_a_badge, account_b_badge)]
        pub fn put_tokens(&mut self, tokens: Bucket) {
            let user_id = Self::get_user_id(&auth);

            scrypto_assert!(!(self.account_a_accepted || self.account_b_accepted) , "Can't add more tokens when someone accepted");
            scrypto_assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge.address() {
                self.token_a.put(tokens);
            } else {
                self.token_b.put(tokens);
            }
        }

        #[auth(account_a_badge)]
        pub fn withdraw_all(&mut self) -> Bucket {
            self.token_a.take_all()
        }

        // Allow the users to withdraw their tokens after both parties accepted or canceled
        #[auth(account_a_badge, account_b_badge)]
        pub fn withdraw(&mut self) -> Bucket {
            let user_id = Self::get_user_id(&auth);
            scrypto_assert!(self.trade_canceled || (self.account_a_accepted && self.account_b_accepted), "The trade must be accepted or canceled");

            if user_id == self.account_a_badge.address() {
                if self.trade_canceled {
                    // Return added tokens
                    self.token_a.take_all()
                } else {
                    // Trade was accepted from both parties, take the token B
                    self.token_b.take_all()
                }
            } else {
                if self.trade_canceled {
                    // Return added tokens
                    self.token_b.take_all()
                } else {
                    // Trade was accepted from both parties, take the token A
                    self.token_a.take_all()
                }
            }
        }

        // Allow the users to accept the trade, after both parties added their tokens to the vault
        #[auth(account_a_badge, account_b_badge)]
        pub fn accept(&mut self) {
            let user_id = Self::get_user_id(&auth);

            scrypto_assert!(self.token_a.amount() > 0.into() && self.token_b.amount() > 0.into(), "Both parties must add their tokens before you can accept");
            scrypto_assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge.address() {
                scrypto_assert!(!self.account_a_accepted, "You already accepted the offer !");
                self.account_a_accepted = true;
            } else {
                scrypto_assert!(!self.account_b_accepted, "You already accepted the offer !");
                self.account_b_accepted = true;
            }
        }

        // Cancel the trade, must be done before both parties accepted
        #[auth(account_a_badge, account_b_badge)]
        pub fn cancel(&mut self) {
            scrypto_assert!(!(self.account_a_accepted && self.account_b_accepted), "The trade is already over, everyone accepted");
            scrypto_assert!(!self.trade_canceled, "The trade is already canceled");

            self.trade_canceled = true;
        }

        // Get user id from the provided badge
        fn get_user_id(badge: &BucketRef) -> Address {
            scrypto_assert!(badge.amount() > 0.into(), "Invalid user proof");
            let user_id = badge.resource_address();
            user_id
        }
    }
}
