use scrypto::prelude::*;

blueprint! {

    struct TokenBurn {
        token_address: Address,
        burnt_amount: Decimal,
    }

    impl TokenBurn {
        pub fn new(token_address: Address) -> Component {
            Self {
                token_address: token_address,
                burnt_amount: Decimal::zero(),
            }
            .instantiate()
        }

        pub fn burn_tokens(&mut self, tokens_to_burn: Bucket) {
            self.burnt_amount += tokens_to_burn.amount();
            Vault::new(ResourceDef::from(self.token_address)).put(tokens_to_burn);
        }

        pub fn get_burnt_amount(&self) -> (Address, Decimal) {
            (self.token_address, self.burnt_amount)
        }
    }
}
