use scrypto::prelude::*;

blueprint! {
    struct Escrow {
        token_a: Vault,
        token_b: Vault,
        account_a_badge: ResourceAddress,
        account_b_badge: ResourceAddress,
        account_a_accepted: bool,
        account_b_accepted: bool,
        trade_canceled: bool
    }

    impl Escrow {
        pub fn new(token_a_address: ResourceAddress, token_b_address: ResourceAddress) -> (ComponentAddress, Bucket, Bucket) {
            // Create the badges that will allow the component to authenticate the users
            let account_a_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("symbol", "BADGE A")
                .initial_supply(1);

            let account_b_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("symbol", "BADGE B")
                .initial_supply(1);

            let component = Self {
                token_a: Vault::new(token_a_address),
                token_b: Vault::new(token_b_address),
                account_a_badge: account_a_badge.resource_address(),
                account_b_badge: account_b_badge.resource_address(),
                account_a_accepted: false,
                account_b_accepted: false,
                trade_canceled: false
            }
            .instantiate()
            .globalize();

            (component, account_a_badge, account_b_badge)
        }

        // Allow the users to put the tokens they want to trade inside the component's vault
        pub fn put_tokens(&mut self, tokens: Bucket, auth: Proof) {
            let user_id = self.get_user_id(&auth);

            assert!(!(self.account_a_accepted || self.account_b_accepted) , "Can't add more tokens when someone accepted");
            assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge {
                self.token_a.put(tokens);
            } else {
                self.token_b.put(tokens);
            }
        }

        // Allow the users to withdraw their tokens after both parties accepted or canceled
        pub fn withdraw(&mut self, auth: Proof) -> Bucket {
            let user_id = self.get_user_id(&auth);
            assert!(self.trade_canceled || (self.account_a_accepted && self.account_b_accepted), "The trade must be accepted or canceled");

            if user_id == self.account_a_badge {
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
        pub fn accept(&mut self, auth: Proof) {
            let user_id = self.get_user_id(&auth);

            assert!(self.token_a.amount() > Decimal::zero() && self.token_b.amount() > Decimal::zero(), "Both parties must add their tokens before you can accept");
            assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge {
                assert!(!self.account_a_accepted, "You already accepted the offer !");
                self.account_a_accepted = true;
            } else {
                assert!(!self.account_b_accepted, "You already accepted the offer !");
                self.account_b_accepted = true;
            }
        }

        // Cancel the trade, must be done before both parties accepted
        pub fn cancel(&mut self, auth: Proof) {
            assert!(!(self.account_a_accepted && self.account_b_accepted), "The trade is already over, everyone accepted");
            assert!(!self.trade_canceled, "The trade is already canceled");
            assert!(
                auth.resource_address() == self.account_a_badge || auth.resource_address() == self.account_b_badge, 
                "Invalid user proof"
            );

            self.trade_canceled = true;
        }

        // Get user id from the provided badge
        fn get_user_id(&self, badge: &Proof) -> ResourceAddress {
            assert!(badge.amount() > Decimal::zero(), "Invalid user proof");
            
            let user_id = badge.resource_address();
            assert!(user_id == self.account_a_badge || user_id == self.account_b_badge, "Invalid user proof");

            user_id
        }
    }
}
