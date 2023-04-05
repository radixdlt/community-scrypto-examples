use scrypto::prelude::*;

// Awesome DEX component
#[blueprint]
mod awesome_module {
    struct AwesomeDex {
        vault_a: Vault,
        vault_b: Vault,
        /// resource address for tokens representing shares in the liquidity pool
        lp_tokens_resource_address: ResourceAddress,
        /// vault containing badge with authority to mint LP tracking tokens
        lp_tokens_minter_badge: Vault,
        /// Awesome token loyalty rewards vault
        reward_tokens_vault: Vault,
        /// Amount of fee charged on swaps which goes for LP rewards
        fee: Decimal,
    }

    impl AwesomeDex {
        pub fn instantiate_awesome(
            bucket_a: Bucket,
            bucket_b: Bucket,
            fee: Decimal,
        ) -> (ComponentAddress, Bucket) {
            // Accept no empty buckets
            assert!(
                !bucket_a.is_empty() && !bucket_b.is_empty(),
                "You must pass in an initial supply of each token"
            );
            assert!(
                fee >= dec!("0") && fee <= dec!("1"),
                "Invalid fee in thousandths"
            );

            // Create badge for minting LP tracking unit token
            let lp_tokens_minter_badge: Bucket = ResourceBuilder::new_fungible()
            .metadata("name", "LP Token Mint Auth")
            .divisibility(DIVISIBILITY_NONE)
            .mint_initial_supply(1);
        
            // Create LP tracking tokens and give them all to the
            // person instantiating component
            let lp_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "LP Token")
                .metadata("symbol", "UNIT")
                .mintable(
                    rule!(require(lp_tokens_minter_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(lp_tokens_minter_badge.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(100);
            
            // Create a loyalty token called awesome where a portion of the supply
            // is given to whoever calls the swap method 
            // Set initial which is also supply to 10_000_000
            let awesome_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Awesome Token")
                .metadata("symbol", "UNIT")
                .mint_initial_supply(10_000_000);

            let awesome_address: ComponentAddress = Self {
                vault_a: Vault::with_bucket(bucket_a),
                vault_b: Vault::with_bucket(bucket_b),
                lp_tokens_resource_address: lp_tokens.resource_address(),
                lp_tokens_minter_badge: Vault::with_bucket(lp_tokens_minter_badge),
                reward_tokens_vault: Vault::with_bucket(awesome_tokens),
                fee: fee,
            }   
            .instantiate()
            .globalize();

            // return component address and LP tokens
            (awesome_address, lp_tokens)
        }

        // swap function
        // Along side output of the swap, 1 reward token is given from the vault
        pub fn swap(&mut self, input_tokens: Bucket) -> (Bucket, Bucket) {
            // resolve which token belongs to which vault
            // This is crude, for multiple liquidity pool impl we need to LP routing
            let (input_tokens_vault, output_tokens_vault): (&mut Vault, &mut Vault) = 
                if input_tokens.resource_address() == self.vault_a.resource_address() {
                    (&mut self.vault_a, &mut self.vault_b)
                } else if input_tokens.resource_address() == self.vault_b.resource_address() {
                    (&mut self.vault_b, &mut self.vault_a)
                } else {
                    panic!(
                        "Input tokens do not match this liquidity pool :o"
                    )
                };
        
            // Use constant maker function to calculate output amount
            // output_amount = (current_total_output_pool_amount * lp_fee * input_amount) / (current_total_input_pool_amount + lp_fee * input_amount)
            let output_amount: Decimal = (output_tokens_vault.amount() 
                * (dec!("1") - self.fee)
                * input_tokens.amount())
                / (input_tokens_vault.amount() + input_tokens.amount()
                * (dec!("1") - self.fee));
            
            info!("input_amount {}", input_tokens.amount());
            info!("output_amount {}", output_amount);

            // perform swapping
            input_tokens_vault.put(input_tokens);

            // output amount with a bucket
            let swap_output: Bucket= output_tokens_vault.take(output_amount);

            let swap_reward: Bucket;
            if !self.reward_tokens_vault.is_empty() {
                swap_reward = self.reward_tokens_vault.take(1);
            }
            else {
                swap_reward =  self.reward_tokens_vault.take(0);
            }

            info!("swap_reward {}", swap_reward.amount());

            (swap_output, swap_reward)
        }

        // add liquidity
        pub fn add_liquidity(
            &mut self,
            bucket_a: Bucket,
            bucket_b: Bucket,
        ) -> (Bucket, Bucket, Bucket) {
            let (mut bucket_a, mut bucket_b) : (Bucket, Bucket) = 
                if bucket_a.resource_address()
                    == self.vault_a.resource_address()
                        && bucket_b.resource_address() == self.vault_b.resource_address()
                {
                    (bucket_a, bucket_b)
                } else if bucket_a.resource_address() == self.vault_b.resource_address()
                    && bucket_b.resource_address() == self.vault_a.resource_address()
                {
                    (bucket_b, bucket_a)
                } else {
                    panic!("One of the tokens does not belong to the pool :( !")
                }; 

                // amounts in bucket a and b
                let dm: Decimal = bucket_a.amount();
                let dn: Decimal = bucket_b.amount();

                // amount in vault a and b
                let m: Decimal = self.vault_a.amount();
                let n: Decimal = self.vault_b.amount();

                // Calculate the amount of tokens which will be added to each amount of the vaults
                let (amount_a, amount_b): (Decimal, Decimal) = 
                if  ((m == Decimal::zero()) || (n == Decimal::zero()))
                    || (m / n) == (dm / dn)
                {
                    // Case 1
                    // logs
                    (dm, dn)
                } else if (m / n) < (dm / dn) {
                    // Case 2
                    // logs
                    (dn * m / n, dn)
                } else {
                    // Case 3
                    // logs
                    (dm, dm * n / m)
                };

                // depositing the amount of tokens calculated in to the liquidity pool
                self.vault_a.put(bucket_a.take(amount_a));
                self.vault_b.put(bucket_b.take(amount_b));

            // mint LP token
            let lp_tokens_manager: ResourceManager = borrow_resource_manager!(self.lp_tokens_resource_address);
            let lp_tokens_amount: Decimal = 
                // this means everyone has withdrawns LPs and we are at zero
                if lp_tokens_manager.total_supply() == Decimal::zero() {
                    dec!("100.00")
                } else {
                    amount_a * lp_tokens_manager.total_supply() / m
                };

            let lp_tokens: Bucket = self
                .lp_tokens_minter_badge
                .authorize(|| lp_tokens_manager.mint(lp_tokens_amount));

            // Return the unused tokens plus LP Token (share in the pool) to the caller
            (bucket_a, bucket_b, lp_tokens)
        }
            

        /// Remove liquidity
        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            assert!(
                lp_tokens.resource_address() == self.lp_tokens_resource_address,
                "Liquidity Pool token not recognized!"
            );

            // TODO: is this correct calculation?
            let lp_tokens_resource_manager: ResourceManager = 
                borrow_resource_manager!(self.lp_tokens_resource_address);

            // Calculate the share
            let share: Decimal = lp_tokens.amount() / lp_tokens_resource_manager.total_supply();

            // Burn the LP tokens received
            self.lp_tokens_minter_badge.authorize(|| {
                lp_tokens.burn()
            });

            // add logs 

            // Return the withdrawn token
            (
                self.vault_a.take(self.vault_a.amount() * share),
                self.vault_b.take(self.vault_b.amount() * share)
            )
        }
    }
}
