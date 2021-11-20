use scrypto::prelude::*;

blueprint! {
    struct Escrow {
        token_a: Vault,
        token_b: Vault,
        token_a_account: Address,
        token_b_account: Address,
        account_a_badge: ResourceDef,
        account_b_badge: ResourceDef,
        account_a_accepted: bool,
        account_b_accepted: bool,
        trade_canceled: bool
    }

    impl Escrow {
        pub fn new(token_a_address: Address, token_b_address: Address, token_a_account_address: Address, token_b_account_address: Address) -> Component {
            // Create the badges that will allow the component to authenticate the users
            let account_a_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE A")
                .new_badge_fixed(1);

            let account_b_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE B")
                .new_badge_fixed(1);

            let account_a_badge_resource = account_a_badge.resource_def();
            let account_b_badge_resource = account_b_badge.resource_def();

            // Send the authentication badges to both users
            Account::from(token_a_account_address).deposit(account_a_badge);
            Account::from(token_b_account_address).deposit(account_b_badge);

            Self {
                token_a: Vault::new(ResourceDef::from(token_a_address)),
                token_b: Vault::new(ResourceDef::from(token_b_address)),
                token_a_account: token_a_account_address,
                token_b_account: token_b_account_address,
                account_a_badge: account_a_badge_resource,
                account_b_badge: account_b_badge_resource,
                account_a_accepted: false,
                account_b_accepted: false,
                trade_canceled: false
            }
            .instantiate()
        }

        // Make sure the user_id matches either account_a or account_b badge
        fn authorize(&self, user_id: Address) -> bool {
            return user_id == self.account_a_badge.address()
                || user_id == self.account_b_badge.address()
        }

        // Check if the user can add their tokens
        fn can_add_tokens(&self, user_id: Address) -> bool {
            (self.token_a.is_empty() && user_id == self.account_a_badge.address()) 
            || (self.token_b.is_empty() && user_id == self.account_b_badge.address())
        }

        // Allow the users to put the tokens they want to trade inside the component's vault
        pub fn put_tokens(&mut self, tokens: Bucket, badge: BucketRef) {
            let user_id = Self::get_user_id(badge);

            scrypto_assert!(self.authorize(user_id), "You are not authorized !");
            scrypto_assert!(self.can_add_tokens(user_id), "You have already added you tokens to the escrow");
            scrypto_assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge.address() {
                self.token_a.put(tokens);
            } else if user_id == self.account_b_badge.address() {
                self.token_b.put(tokens);
            } else {
                scrypto_abort("Not authorized !");
            }
        }

        // Allow the users to accept the trade, after both parties added their tokens to the vault
        pub fn accept(&mut self, badge: BucketRef) {
            let user_id = Self::get_user_id(badge);

            scrypto_assert!(self.authorize(user_id), "You are not authorized !");
            scrypto_assert!(!self.can_add_tokens(user_id), "Both parties must add their tokens before you can accept");
            scrypto_assert!(!self.trade_canceled, "The trade was canceled");

            if user_id == self.account_a_badge.address() {
                scrypto_assert!(!self.account_a_accepted, "You already accepted the offer !");
                self.account_a_accepted = true;
            } else if user_id == self.account_b_badge.address() {
                scrypto_assert!(!self.account_b_accepted, "You already accepted the offer !");
                self.account_b_accepted = true;
            } else {
                scrypto_abort("Not authorized !");
            }

            // Send the tokens if both parties accepted
            if self.account_a_accepted && self.account_b_accepted {
                Account::from(self.token_a_account).deposit(self.token_b.take_all());
                Account::from(self.token_b_account).deposit(self.token_a.take_all());
            }
        }

        // Cancel the trade, must be done before both parties accepted
        #[auth(account_a_badge, account_b_badge)]
        pub fn cancel(&mut self) {
            scrypto_assert!(!(self.account_a_accepted && self.account_b_accepted), "The trade is already over, everyone accepted");
            scrypto_assert!(!self.trade_canceled, "The trade is already canceled");

            self.trade_canceled = true;

            // Give back the stored tokens to their original owner
            Account::from(self.token_a_account).deposit(self.token_a.take_all());
            Account::from(self.token_b_account).deposit(self.token_b.take_all());
        }

        // Get user id from the provided badge
        fn get_user_id(badge: BucketRef) -> Address {
            scrypto_assert!(badge.amount() > 0.into(), "Invalid user proof");
            let user_id = badge.resource_address();
            badge.drop();
            user_id
        }
    }
}
