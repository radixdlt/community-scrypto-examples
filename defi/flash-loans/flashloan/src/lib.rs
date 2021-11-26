use scrypto::prelude::*;

blueprint! {
    struct FlashLoan {
        interest: Decimal,
        vault: Vault
    }

    impl FlashLoan {
        /*
         * Generate a FlashLoan component with a specified interest and a specified bucket as funds
         */
        pub fn new(tokens: Bucket, interest: Decimal) -> Component {
            Self {
                vault: Vault::with_bucket(tokens),
                interest: interest,
            }
            .instantiate()
        }

        pub fn request_loan(&self, amount: Decimal, component_address: Address) -> Bucket {
            scrypto_assert!(amount < self.vault.amount(), "Not enough funds to loan");

            // Call the execute method at the specified component's address with the requested funds
            let args = vec![
                scrypto_encode(&self.vault.take(amount))
            ];

            let returned_bucket: Bucket = Component::from(component_address).call::<Bucket>("execute", args).into();

            // Make sure they repaid in loan in full
            let amount_to_take = amount * ((self.interest / 100) + 1);
            scrypto_assert!(returned_bucket.amount() >= amount_to_take, "You have to return more than {}", amount_to_take);

            self.vault.put(returned_bucket.take(amount_to_take));

            // Return the change back to the component
            return returned_bucket;
        }
    }
}
