use scrypto::prelude::*;

blueprint! {
    struct Escrow {
        token_a: Vault,
        token_b: Vault,
        token_a_account: Account,
        token_b_account: Account,
        account_a_badge_ressource: ResourceDef,
        account_b_badge_ressource: ResourceDef,
        account_a_accepted: bool,
        account_b_accepted: bool,
        trade_canceled: bool
    }

    impl Escrow {
        pub fn new(token_a_address: Address, token_b_address: Address, token_a_account_address: Address, token_b_account_address: Address) -> Component {
            let account_a_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE A")
                .new_badge_fixed(1);

            let account_b_badge = ResourceBuilder::new()
                .metadata("symbol", "BADGE B")
                .new_badge_fixed(1);
            
            let a_resource_def = account_a_badge.resource_def();
            let b_resource_def = account_b_badge.resource_def();

            let token_a_account = Account::from(token_a_account_address);
            let token_b_account = Account::from(token_b_account_address);

            token_a_account.deposit(account_a_badge);
            token_b_account.deposit(account_b_badge);

            Self {
                token_a: Vault::new(ResourceDef::from(token_a_address)),
                token_b: Vault::new(ResourceDef::from(token_b_address)),
                token_a_account: token_a_account,
                token_b_account: token_b_account,
                account_a_badge_ressource: a_resource_def,
                account_b_badge_ressource: b_resource_def,
                account_a_accepted: false,
                account_b_accepted: false,
                trade_canceled: false
            }
            .instantiate()
        }

        fn authorize(&self, badge: &BucketRef) -> bool {
            return badge.resource_def() == self.account_a_badge_ressource 
                || badge.resource_def() == self.account_b_badge_ressource
        }

        fn can_add_tokens(&self, badge_resource: ResourceDef) -> bool {
            (self.token_a.is_empty() && badge_resource == self.account_a_badge_ressource) 
            || (self.token_b.is_empty() && badge_resource == self.account_b_badge_ressource)
        }

        fn all_parties_accepted(&self) -> bool {
            self.account_a_accepted && self.account_b_accepted
        }

        fn send_tokens(&self) {
            if self.all_parties_accepted() {
                self.token_a_account.deposit(self.token_b.take_all());
                self.token_b_account.deposit(self.token_a.take_all());
            }
        }

        pub fn put_tokens(&self, tokens: Bucket, badge: BucketRef) {
            scrypto_assert!(self.authorize(&badge), "You are not authorized !");

            scrypto_assert!(
                !self.trade_canceled,
                "The trade was canceled"
            );

            scrypto_assert!(
                self.can_add_tokens(badge.resource_def()),
                "Both tokens have already been added to the escrow"
            );

            if badge.resource_def() == self.account_a_badge_ressource {
                self.token_a.put(tokens);
            } else if badge.resource_def() == self.account_b_badge_ressource {
                self.token_b.put(tokens);
            }

            badge.drop();
        }

        pub fn accept(&mut self, badge: BucketRef) {
            scrypto_assert!(self.authorize(&badge), "You are not authorized !");

            scrypto_assert!(
                !self.trade_canceled,
                "The trade was canceled"
            );
            
            scrypto_assert!(
                !self.can_add_tokens(badge.resource_def()),
                "Both tokens have already been added to the escrow"
            );

            if badge.resource_def() == self.account_a_badge_ressource {
                scrypto_assert!(!self.account_a_accepted, "You already accepted the offer !");
                self.account_a_accepted = true;
            } else if badge.resource_def() == self.account_b_badge_ressource {
                scrypto_assert!(!self.account_b_accepted, "You already accepted the offer !");
                self.account_b_accepted = true;
            }

            self.send_tokens();
            badge.drop();
        }

        #[auth(account_a_badge_ressource, account_b_badge_ressource)]
        pub fn cancel(&mut self) {
            scrypto_assert!(
                !(self.account_a_accepted && self.account_b_accepted),
                "The trade is already over, everyone accepted"
            );

            scrypto_assert!(
                !self.trade_canceled,
                "The trade is already canceled"
            );

            self.trade_canceled = true;

            // Give back the stored tokens
            self.token_a_account.deposit(self.token_a.take_all());
            self.token_b_account.deposit(self.token_b.take_all());
        }
    }
}
