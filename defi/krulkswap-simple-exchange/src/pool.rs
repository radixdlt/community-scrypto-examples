use scrypto::prelude::*;
use std::collections::HashMap;

// Satellite blueprint for the Exchange main component blueprint.
blueprint! {
    struct Pool {
        // Hashmap stores the two vaults of the tokens in the token pair.
        vaults: HashMap<ResourceAddress, Vault>,
        // Internal minting badge for the LP tokens.
        lp_badge: Vault,
        // ResourceAddress of the LP token.
        lp_token: ResourceAddress,
        // Initial supply of the LP tokens
        lp_initial_supply: Decimal,
        // Current total supply of the LP tokens.
        lp_total_supply: Decimal,
        // Special sauce K constant of CFMM. (explained below)
        k_constant: Decimal

        // Many CFMMs (Constant Function Market Makers) are based on the
        // Constant Product. We can define this with the equation:
        // k = a * b
        // where:
        // k is a constant
        // a is the pool supply of token A
        // b is the pool supply of token B

        // When making a swap, we can use this equation
        // to calculate how many tokens of token B to give
        // back for an amount of input tokens of token A.
    }

    impl Pool {
        // Instantiate a PoolComponent
        // Returns the instance and a bucket of LP-tokens.
        pub fn new_pool(a: Bucket, b: Bucket) -> (PoolComponent, Bucket) {
            let a_address = a.resource_address();
            let b_address = b.resource_address();
            // Mint the pool's LP minting badge.
            let mint_badge = ResourceBuilder::new_fungible()
                .metadata("name", "KrulkSwap ".to_owned() + " LP mint badge")
                .initial_supply(1);
            // Create a definition for the LP tokens, but don't mint any yet.
            let lp_def = ResourceBuilder::new_fungible()
                .metadata("name", "KrulkSwap ".to_owned() + " LP token")
                .metadata("symbol", "LP")
                // Require the internal mint badge to mint more LP tokens.
                .mintable(rule!(require(mint_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // We choose to define k as the product of the two token amounts here.
            let k = a.amount() * b.amount();
            // let's define the initial amount of LP-tokens arbitrarily as the sum of the intitial amounts.
            let lp_total_supply = a.amount() + b.amount();
            // Mint LP-tokens using the lp_def definition.
            let lp_tokens = borrow_resource_manager!(lp_def).mint(lp_total_supply);

            // Create a hashmap for the vaults.
            let mut vaults = HashMap::new();
            // Insert the buckets with their keys into the hashmap.
            vaults.insert(a_address, Vault::with_bucket(a));
            vaults.insert(b_address, Vault::with_bucket(b));

            // Create the instance.
            (Self {
                vaults: vaults,
                lp_badge: Vault::with_bucket(mint_badge),
                lp_token: lp_def,
                lp_initial_supply: k,
                lp_total_supply: lp_total_supply,
                k_constant: k
            }
            // Instantiate the instance. and return the instance together with the LP-tokens.
            .instantiate(), lp_tokens)
        }

        // Add some liquidity to this pool.
        // Returns a Bucket of LP tokens for this pool.
        pub fn add_liquidity(&mut self, a: Bucket, b: Bucket) -> Bucket {
            let a_vault_balance = self.vaults[&a.resource_address()].amount();
            let b_vault_balance = self.vaults[&b.resource_address()].amount();


            // Calculate the ratio between the amount of given tokens and the tokens in the pool.
            // These must be the same when adding liquidity, or the "price" will change.
            let a_ratio = a.amount() / a_vault_balance;
            let b_ratio = b.amount() / b_vault_balance;

            // Check if the ratios are equal.
            assert!(
                a_ratio == b_ratio,
                "Input token ratio is incorrect"
            );

            // Calculate the amount of LP-tokens to mint in return.
            // This should represent your share of the pool.
            let amount_of_lp = a_ratio * (a_vault_balance + b_vault_balance);

            // Put the tokens in the pool.
            self.put(a);
            self.put(b);

            // Update the total supply of LP tokens.
            self.lp_total_supply += amount_of_lp;
            // Update the k-value of the pool.
            self.k_constant = a_vault_balance * b_vault_balance;
            // Mint the LP-tokens and return them.
            borrow_resource_manager!(self.lp_token).mint(amount_of_lp)
        }

        // Redeem LP-tokens and return the liquidity.
        pub fn take_liquidity(&mut self, lp: Bucket) -> (Bucket, Bucket) {
            // Get the ResourceAddresses of the tokens in the pool.
            let addresses = self.addresses();
            // Get the balances of the vaults in the pool.
            let a_vault_balance = self.vaults[&addresses[0]].amount();
            let b_vault_balance = self.vaults[&addresses[1]].amount();

            // Calculate the amount to take from each pool.
            let factor_of_pool = lp.amount() / self.lp_total_supply;
            let amount_of_a = factor_of_pool * a_vault_balance;
            let amount_of_b = factor_of_pool * b_vault_balance;

            // Update the LP total supply.
            self.lp_total_supply -= lp.amount();
            // Update the k value of the pool.
            self.k_constant = a_vault_balance * b_vault_balance;

            // Burn the LP tokens and return a tuple with the token Buckets.
            borrow_resource_manager!(self.lp_token).burn(lp);
            (self.take(self.vaults[&addresses[0]].resource_address(), amount_of_a),
            self.take(self.vaults[&addresses[0]].resource_address(), amount_of_b))
        }

        // Swap one token for the target token.
        // Returns a bucket with the target tokens.
        pub fn swap(&mut self, input: Bucket, target_resource: ResourceAddress) -> Bucket {
            // Get the balances of the vaults.
            let input_vault_balance = self.vaults[&input.resource_address()].amount();
            let output_vault_balance = self.vaults[&target_resource].amount();
            // Calculate the new balance that A will have.
            let new_input_amount = input_vault_balance + input.amount();
            // Calculate the new balance token B should have.
            // Note that we use a form of the k = a * b equation here: b = k / a
            let new_output_vault_balance = self.k_constant / new_input_amount;
            // To get the amount of B to give, we substract the balance of B from the old
            // balance of B, which gives us the difference.
            let amount_to_give = output_vault_balance - new_output_vault_balance;
            self.take(target_resource, amount_to_give)
        }

        // Put some tokens into the pool.
        fn put(&mut self, bucket: Bucket) {
            assert!(
                !self.vaults.contains_key(&bucket.resource_address()),
                "This asset is not in this pool."
            );
            self.vaults.get_mut(&bucket.resource_address()).unwrap().put(bucket);
        }

        // Take some tokens out of the pool.
        fn take(&mut self, resource: ResourceAddress, amount: Decimal) -> Bucket {
            assert!(
                !self.vaults.contains_key(&resource),
                "This asset is not in this pool."
            );
            self.vaults.get_mut(&resource).unwrap().take(amount)
        }
        pub fn addresses(&self) -> Vec<ResourceAddress> {
            self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>()
        }
    }
}