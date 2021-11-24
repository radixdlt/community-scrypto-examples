use scrypto::prelude::*;

/*
A blueprint for creating and managing a utility token.
*/

blueprint! {
    struct UtilityTokenFactory {
        ut_minter_vault: Vault,
        ut_minter_badge: ResourceDef,
        available_ut: Vault,            // The available supply of utility tokens which are used to pay for some services.
        ut_token_price: Decimal,        // How many XRD the UT Tokens cost. (Fractional amounts are not recommended at this time.)
        collected_xrd: Vault,           // proceeds from selling UT tokens
        ut_max_buy : u32,               // Maximum number of UT tokens that can be purchased at a time.
        ut_mint_size: u32,              // How many UT tokens to mint in each batch.
        total_claimed: Decimal,         // Total XRD claimed during the contract lifetime.
        total_minted: u32,              // How many utility tokens have been minted.
        total_redeemed: Decimal,        // How many utility tokens have been redeemed and burned.
    }

    impl UtilityTokenFactory {

        pub fn new( my_id: String, ut_name: String, ut_symbol: String, ut_description: String, price: u32, mint_size: u32, max_buy: u32 ) -> (Component, Bucket) {
            let ut_minter_bucket = ResourceBuilder::new()
                .metadata("name", my_id)
                .new_badge_fixed(2);
            let ut_minter_resource_def = ut_minter_bucket.resource_def();
            let ut_minter_return_bucket: Bucket = ut_minter_bucket.take(1); // Return this badge to the caller

            let ut_resource_def = ResourceBuilder::new()
                .metadata("name", ut_name)
                .metadata("symbol", ut_symbol)
                .metadata("description", ut_description)
                .new_token_mutable(ut_minter_bucket.resource_address());
            let ut_tokens = ut_resource_def.mint(mint_size, ut_minter_bucket.borrow());

            scrypto_assert!(mint_size > 0, "You must specify a non-zero number for the mint_size.");
            scrypto_assert!(max_buy <= mint_size, "The single purchase max buy size should be less than or equal to the mint size.");
            let component = Self {
                ut_minter_vault: Vault::with_bucket(ut_minter_bucket),
                ut_minter_badge: ut_minter_resource_def,

                available_ut: Vault::with_bucket(ut_tokens),
                ut_token_price: price.into(),
                collected_xrd: Vault::new(RADIX_TOKEN),

                ut_max_buy: max_buy,
                ut_mint_size: mint_size,

                total_claimed: 0.into(),
                total_minted: mint_size,
                total_redeemed: 0.into()
            }
            .instantiate();
            (component, ut_minter_return_bucket)
        }

        // Convenience function returns the address of the Utility Token
        //
        pub fn address(&self) -> Address {
            self.available_ut.resource_def().address()
        }

        // Purchase UT tokens
        //
        pub fn purchase(&mut self, number: u32, payment: Bucket) -> (Bucket, Bucket) {
            scrypto_assert!(payment.resource_def() == RADIX_TOKEN.into(), "You must purchase the utility tokens with Radix (XRD).");
            let ut_bucket = Bucket::new(self.address());
            let mut num = number;
            if num > self.ut_max_buy {
                num = self.ut_max_buy; // 1,000 is the max allowable purchase for now.
                info!("A max of {} tokens can be purcahsed at a time.", self.ut_max_buy);
            }
            if payment.amount() < self.ut_token_price * num {
                info!("Insufficient funds. Required payment for {} UT tokens is {} XRD.", num, self.ut_token_price * num);
            } else {
                info!("Thank you!");
                if self.available_ut.amount() < num.into() {   // if they are needed, mint more UT tokens
                    let new_tokens = self.ut_minter_vault.authorize(|badge| {
                        self.available_ut.resource_def().mint(self.ut_mint_size, badge)
                    });
                    self.available_ut.put(new_tokens);
                    self.total_minted += self.ut_mint_size;
                }
                self.collected_xrd.put(payment.take(self.ut_token_price * num));
                ut_bucket.put(self.available_ut.take(num));
            }
            (payment, ut_bucket)
        }

        #[auth(ut_minter_badge)]
        pub fn show_bank(&self) {
            let metadata = self.available_ut.resource_def().metadata();
            info!("Available {}: {}", metadata["symbol"], self.available_ut.amount());
            info!("Claimable XRD: {}", self.collected_xrd.amount());
            info!("Total XRD Claimed: {}", self.total_claimed);
            info!("Total {} Minted: {}", metadata["symbol"], self.total_minted);
            info!("Total {} Redeemed: {}", metadata["symbol"], self.total_redeemed);
        }

        #[auth(ut_minter_badge)]
        pub fn claim(&mut self) -> Bucket {
            self.total_claimed += self.collected_xrd.amount();
            self.collected_xrd.take_all()
        }

        pub fn redeem(&mut self, used_tokens: Bucket) {
            if used_tokens.amount() > 0.into() {
                scrypto_assert!(used_tokens.resource_def() == self.available_ut.resource_def(), "You can only redeem the expected utility tokens.");
                self.total_redeemed += used_tokens.amount();
                self.ut_minter_vault.authorize(|badge| {
                    used_tokens.burn(badge);
                })
            }
        }
    }
}
