use scrypto::prelude::*;

blueprint! {
    struct TokenVending {
        tokens: Vault,
        admin_badge_container: Vault,
        payment: Vault,
        price: Decimal,
    }

    impl TokenVending {
        pub fn instantiate_gumball_machine(price: Decimal) -> ComponentAddress {
            let tokens = ResourceBuilder::new_fungible()
                .metadata("name", "Best Token")
                .metadata("symbol", "BEST")
                .initial_supply(100000);

            let admin_badge = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("description", "Only the admin can set the token sale price")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            
            Self {
                tokens: Vault::with_bucket(tokens),
                admin_badge_container: Vault::with_bucket(admin_badge),
                payment: Vault::new(RADIX_TOKEN),
                price: price,
            }
            .instantiate()
            .globalize()
        }

        pub fn set_price(&mut self, _price: Decimal) {
            self.admin_badge_container.authorize(|| {
                self.price = _price
            });
        }

        pub fn buy_token(&mut self, mut payment: Bucket, amount: Decimal) -> (Bucket, Bucket) {
            self.payment.put(payment.take(self.price * amount));
            (self.tokens.take(amount), payment)
        }
    }
}