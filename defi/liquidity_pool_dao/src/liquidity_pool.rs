use scrypto::prelude::*;
use crate::utils::*;
use crate::liquidity_pool_dao::*;

blueprint!{
    /// This is a struct used to define a liquidity pool in the RaDEX decentralized exchange. A typical liquidity pool 
    /// is made up of two vaults which are used to store the two tokens being traded against one another. In addition to 
    /// that, a number of methods are implemented for this struct in order to make it easier to add, and remove 
    /// liquidity into and from the pool, swap assets, and calculate the k value in the constant market maker function 
    /// `x * y = k`.
    struct LiquidityPool {
        /// These are the vaults where the reserves of Token A and Token B will be stored. The choice of storing the two
        /// vaults in a hashmap instead of storing them in two different struct variables is made to allow for an easier
        /// and more dynamic way of manipulating and making use of the vaults inside the liquidity pool. Keep in mind 
        /// that this `vaults` hashmap will always have exactly two vaults for the two assets being traded against one 
        /// another.
        vaults: HashMap<ResourceAddress, Vault>,

        /// When liquidity providers provide liquidity to the liquidity pool, they are given a number of tokens that is
        /// equivalent to the percentage ownership that they have in the liquidity pool. The tracking token is the token
        /// that the liquidity providers are given when they provide liquidity to the pool and this is the resource 
        /// address of the token.
        tracking_token_address: ResourceAddress,

        /// The tracking tokens are mutable supply tokens that may be minted and burned when liquidity is supplied or 
        /// removed from the liquidity pool. This badge is the badge that has the authority to mint and burn the tokens
        /// when need be.
        tracking_token_admin_badge: Vault,

        /// This is a decimal value between 0 and 100 which defines the amount of fees paid to the liquidity pool (and
        /// in turn the liquidity providers) when a swap is made through this liquidity pool.
        fee_to_pool: Decimal,
        nft_proposal_admin: Vault,
        token_1_weight: Decimal,
        token_2_weight: Decimal,
        total_token_weight: Decimal,
        nft_proposal_address: Vec<ResourceAddress>,

        liquidity_pool_dao_address: ComponentAddress,
    }

    impl LiquidityPool {
        /// Creates a new liquidity pool of the two token types passed to this function.
        /// 
        /// This method is used to instantiate a new liquidity pool of the two token types that were passed to this
        /// function in the two buckets. This token pair may be swapped and all swaps will have a fee of `fee` imposed
        /// on it. 
        /// 
        /// This function does a number of checks before a Liquidity Pool is created, these checks are:
        /// 
        /// * **Check 1:** Checks that `token1` and `token2` are not of the same type.
        /// * **Check 2:** Checks that both `token1` and `token2` are fungible tokens.
        /// * **Check 3:** Checks that neither of the buckets are empty.
        /// * **Check 4:** Checks that the fee is between 0 and 100.
        /// 
        /// If these checks are successful, then a new liquidity pool is created from the two buckets passed to this 
        /// function and tracking tokens are minted for the creator of this liquidity pool. Keep in mind that this 
        /// function has now way of checking if a liquidity pool of this kind exists or not. So, it is the job of the 
        /// RaDEX component to only call this function if there does not already exist a liquidity pool for the given
        /// token pair.
        /// 
        /// # Arguments: 
        /// 
        /// * `token1` (Bucket) - A bucket containing the amount of the first token used to initialize the pool.
        /// * `token2` (Bucket) - A bucket containing the amount of the second token used to initialize the pool.
        /// * `fee` (Decimal) - A decimal value of the fee imposed on all swaps from this liquidity pool. This should be
        /// a value between 0 and 100.
        /// 
        /// # Returns:
        /// 
        /// * `Component` - A LiquidityPool component of the newly created liquidity pool.
        /// * `Bucket` - A bucket containing the tracking tokens issued to the creator of the liquidity pool.
        pub fn new(
            token_1_weight: Decimal,
            token_2_weight: Decimal,
            token1: Bucket,
            token2: Bucket,
            fee_to_pool: Decimal
        ) -> (ComponentAddress, Bucket) 
        {
            // Performing the checks to see if this liquidity pool may be created or not.
            assert_ne!(
                token1.resource_address(), token2.resource_address(),
                "[Pool Creation]: Liquidity pools may only be created between two different tokens."
            );

            assert_ne!(
                borrow_resource_manager!(token1.resource_address()).resource_type(), ResourceType::NonFungible,
                "[Pool Creation]: Both assets must be fungible."
            );
            assert_ne!(
                borrow_resource_manager!(token2.resource_address()).resource_type(), ResourceType::NonFungible,
                "[Pool Creation]: Both assets must be fungible."
            );

            assert!((token_1_weight + token_2_weight) <= dec!("1.0"), 
                "[Pool Creation]: The weight of both tokens cannot exceed {:?}.", dec!("1.0")
            );

            assert!(
                !token1.is_empty() & !token2.is_empty(), 
                "[Pool Creation]: Can't create a pool from an empty bucket."
            );

            assert!(
                (fee_to_pool >= Decimal::zero()) & (fee_to_pool <= dec!("100")), 
                "[Pool Creation]: Fee must be between 0 and 100"
            );

            let total_amount = token1.amount() + token2.amount();

            let total_token_weight = token_1_weight + token_2_weight;

            let token1_bucket_weight = token1.amount() / total_amount;

            let token2_bucket_weight = token2.amount() / total_amount;

            assert_eq!(token1_bucket_weight + token2_bucket_weight, token_1_weight + token_2_weight,
                "[Pool Creation]: The amount provided exceeds the total weighted amount.
                The total weight is {:?}.
                You've provided {:?} of weight for Bucket 1 and {:?} for Bucket 2.",
                total_token_weight, token1_bucket_weight, token2_bucket_weight);
            
            assert_eq!(token1_bucket_weight, token_1_weight,
                "[Pool Creation]: You've inputed {:?} as your weight for Token 1, but allocated {:?} for Token 1.
                The amount provided for Token 1 exceeds the weighted requirement.",
                token_1_weight, token1_bucket_weight);

            assert_eq!(token2_bucket_weight, token_2_weight,
                "[Pool Creation]: You've inputed {:?} as your weight for Token 2, but allocated {:?} for Token 2.
                The amount provided for Token 1 exceeds the weighted requirement.",
                token_2_weight, token2_bucket_weight);   

            // At this point, we know that the pool creation can indeed go through. 
            
            // Sorting the buckets and then creating the hashmap of the vaults from the sorted buckets
            let (bucket1, bucket2): (Bucket, Bucket) = sort_buckets(token1, token2);
            let addresses: (ResourceAddress, ResourceAddress) = (bucket1.resource_address(), bucket2.resource_address());
            
            let lp_id: String = format!("{}-{}", addresses.0, addresses.1);
            let pair_name: String = address_pair_symbol(addresses.0, addresses.1);

            info!(
                "[Pool Creation]: Creating new pool between tokens: {}, of name: {}, Ratio: {}:{}", 
                lp_id, pair_name, bucket1.amount(), bucket2.amount()
            );

            let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            vaults.insert(bucket1.resource_address(), Vault::with_bucket(bucket1));
            vaults.insert(bucket2.resource_address(), Vault::with_bucket(bucket2));


            // Creating the admin badge of the liquidity pool which will be given the authority to mint and burn the
            // tracking tokens issued to the liquidity providers.
            let tracking_token_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Tracking Token Admin Badge")
                .metadata("symbol", "TTAB")
                .metadata("description", "This is an admin badge that has the authority to mint and burn tracking tokens")
                .metadata("lp_id", format!("{}", lp_id))
                .initial_supply(1);

            // Creating the tracking tokens and minting the amount owed to the initial liquidity provider
            let tracking_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("{} LP Tracking Token", pair_name))
                .metadata("symbol", "TT")
                .metadata("description", "A tracking token used to track the percentage ownership of liquidity providers over the liquidity pool")
                .metadata("lp_id", format!("{}", lp_id))
                .mintable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .burnable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .initial_supply(100);

            let mut nft_proposal_admin: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "NFT Proposal Admin Badge")
                .metadata("symbol", "NPAB")
                .metadata("description", "This is an admin badge that has authority to mint and burn NFT proposals.")
                .initial_supply(2);

            let nft_proposal_admin2: Bucket = nft_proposal_admin.take(1);

            // Creating the liquidity pool component and instantiating it
            let liquidity_pool: ComponentAddress = Self { 
                vaults: vaults,
                tracking_token_address: tracking_tokens.resource_address(),
                tracking_token_admin_badge: Vault::with_bucket(tracking_token_admin_badge),
                fee_to_pool: fee_to_pool,
                nft_proposal_admin: Vault::with_bucket(nft_proposal_admin),
                token_1_weight: token_1_weight,
                token_2_weight: token_2_weight,
                total_token_weight: total_token_weight,
                nft_proposal_address: Vec::new(),
                liquidity_pool_dao_address: LiquidityPoolDAO::new(tracking_tokens.resource_address(), nft_proposal_admin2)
            }
            .instantiate()
            .globalize();

            return (liquidity_pool, tracking_tokens);
        }

        /// Checks if the given address belongs to this pool or not.
        /// 
        /// This method is used to check if a given resource address belongs to one of the tokens in this liquidity pool
        /// or not. A resource belongs to a liquidity pool if its address is in the addresses in the `vaults` HashMap.
        /// 
        /// # Arguments:
        /// 
        /// * `address` (ResourceAddress) - The address of the resource that we wish to check if it belongs to the pool.
        /// 
        /// # Returns:
        /// 
        /// * `bool` - A boolean of whether the address belongs to this pool or not.
        pub fn belongs_to_pool(
            &self, 
            address: ResourceAddress
        ) -> bool {
            return self.vaults.contains_key(&address);
        }

        /// Asserts that the given address belongs to the pool.
        /// 
        /// This is a quick assert method that checks if a given address belongs to the pool or not. If the address does
        /// not belong to the pool, then an assertion error (panic) occurs and the message given is outputted.
        /// 
        /// # Arguments:
        /// 
        /// * `address` (ResourceAddress) - The address of the resource that we wish to check if it belongs to the pool.
        /// * `label` (String) - The label of the method that called this assert method. As an example, if the swap 
        /// method were to call this method, then the label would be `Swap` so that it's clear where the assertion error
        /// took place.
        pub fn assert_belongs_to_pool(
            &self, 
            address: ResourceAddress, 
            label: String
        ) {
            assert!(
                self.belongs_to_pool(address), 
                "[{}]: The provided resource address does not belong to the pool.", 
                label
            );
        }

        /// Gets the resource addresses of the tokens in this liquidity pool and returns them as a `Vec<ResourceAddress>`.
        /// 
        /// # Returns:
        /// 
        /// `Vec<ResourceAddress>` - A vector of the resource addresses of the tokens in this liquidity pool.
        pub fn addresses(&self) -> Vec<ResourceAddress> {
            return self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>();
        }

        /// Gets the name of the given liquidity pool from the symbols of the two tokens.
        /// 
        /// # Returns:
        /// 
        /// `String` - A string of the pair symbol
        pub fn name(&self) -> String {
            let addresses: Vec<ResourceAddress> = self.addresses();
            return address_pair_symbol(addresses[0], addresses[1]);
        }

        /// Gets the address of the other resource if the passed resource address belongs to the pool.
        /// 
        /// This method takes in a resource address and if this resource address belongs to the pool it returns the 
        /// address of the other token in this liquidity pool.
        /// 
        /// This method performs a number of checks before resource address is obtained:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// 
        /// # Arguments
        /// 
        /// * `resource_address` (ResourceAddress) - The resource address for a token from the pool.
        /// 
        /// # Returns:
        /// 
        /// * `ResourceAddress` - The address of the other token in this pool.
        pub fn other_resource_address(
            &self,
            resource_address: ResourceAddress
        ) -> ResourceAddress {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(resource_address, String::from("Other Resource ResourceAddress"));

            // Checking which of the addresses was provided as an argument and returning the other address.
            let addresses: Vec<ResourceAddress> = self.addresses();
            return if addresses[0] == resource_address {addresses[1]} else {addresses[0]};
        }

        /// Retrieves pool parameters.
        pub fn get_pool_info(
            &self
        )
        {
            info!("Token 1 weight: {:?}", self.token_1_weight);
            info!("Token 2 weight: {:?}", self.token_2_weight);
            info!("Pool fee: {:?}", self.fee_to_pool);
        }

        /// Calculates the k in the constant market maker equation: `x * y = k`.
        /// 
        /// # Returns:
        /// 
        /// `Decimal` - A decimal value of the reserves amount of Token A and Token B multiplied by one another.
        pub fn k(&self) -> Decimal {
            let addresses: Vec<ResourceAddress> = self.addresses();
            return self.vaults[&addresses[0]].amount() * self.vaults[&addresses[1]].amount()
        }

        /// Calculates the amount of output that can be given for for a given amount of input.
        /// 
        /// This method calculates the amount of output tokens that would be received for a given amount of an input
        /// token. This is calculated through the constant market maker function `x * y = k`. 
        /// 
        /// This method performs a number of checks before the calculation is done:
        /// 
        /// * **Check 1:** Checks that the provided resource address belongs to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `input_resource_address` (ResourceAddress) - The resource address of the input token.
        /// * `input_amount` (Decimal) - The amount of input tokens to calculate the output for.
        /// 
        /// # Returns:
        /// 
        /// * `Decimal` - The output amount for the given input.
        /// 
        /// # Note:
        /// 
        /// This method is equivalent to finding `dy` in the equation `(x + rdx)(y - dy) = xy` where the symbols used
        /// mean the following:
        /// 
        /// * `x` - The amount of reserves of token x (the input token)
        /// * `y` - The amount of reserves of token y (the output token)
        /// * `dx` - The amount of input tokens
        /// * `dy` - The amount of output tokens
        /// * `r` - The fee modifier where `r = (100 - fee) / 100`
        pub fn calculate_output_amount(
            &self,
            input_resource_address: ResourceAddress,
            input_amount: Decimal
        ) -> Decimal {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(input_resource_address, String::from("Calculate Output"));

            let x: Decimal = self.vaults[&input_resource_address].amount();
            let y: Decimal = self.vaults[&self.other_resource_address(input_resource_address)].amount();
            let dx: Decimal = input_amount;
            let r: Decimal = (dec!("100") - self.fee_to_pool) / dec!("100");

            let dy: Decimal = (dx * r * y) / ( x + r * dx );
            return dy;
        }

        /// Calculates the amount of input required to receive the specified amount of output tokens.
        /// 
        /// This method calculates the amount of input tokens that would be required to receive the specified amount of
        /// output tokens. This is calculated through the constant market maker function `x * y = k`. 
        /// 
        /// This method performs a number of checks before the calculation is done:
        /// 
        /// * **Check 1:** Checks that the provided resource address belongs to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `output_resource_address` (ResourceAddress) - The resource address of the output token.
        /// * `output_amount` (Decimal) - The amount of output tokens to calculate the input for.
        /// 
        /// # Returns:
        /// 
        /// * `Decimal` - The input amount for the given output.
        /// 
        /// # Note:
        /// 
        /// This method is equivalent to finding `dx` in the equation `(x + rdx)(y - dy) = xy` where the symbols used
        /// mean the following:
        /// 
        /// * `x` - The amount of reserves of token x (the input token)
        /// * `y` - The amount of reserves of token y (the output token)
        /// * `dx` - The amount of input tokens
        /// * `dy` - The amount of output tokens
        /// * `r` - The fee modifier where `r = (100 - fee) / 100`
        pub fn calculate_input_amount(
            &self,
            output_resource_address: ResourceAddress,
            output_amount: Decimal
        ) -> Decimal {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(output_resource_address, String::from("Calculate Input"));

            let x: Decimal = self.vaults[&self.other_resource_address(output_resource_address)].amount();
            let y: Decimal = self.vaults[&output_resource_address].amount();
            let dy: Decimal = output_amount;
            let r: Decimal = (dec!("100") - self.fee_to_pool) / dec!("100");

            let dx: Decimal = (dy * x) / (r * (y - dy));
            return dx;
        }

        /// Deposits a bucket of tokens into this liquidity pool.
        /// 
        /// This method determines if a given bucket of tokens belongs to the liquidity pool or not. If it's found that
        /// they belong to the pool, then this method finds the appropriate vault to store the tokens and deposits them
        /// to that vault.
        /// 
        /// This method performs a number of checks before the deposit is made:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `bucket` (Bucket) - A buckets of the tokens to deposit into the liquidity pool
        fn deposit(
            &mut self,
            bucket: Bucket 
        ) {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(bucket.resource_address(), String::from("Deposit"));

            self.vaults.get_mut(&bucket.resource_address()).unwrap().put(bucket);
        }

        /// Withdraws tokens from the liquidity pool.
        /// 
        /// This method is used to withdraw a specific amount of tokens from the liquidity pool. 
        /// 
        /// This method performs a number of checks before the withdraw is made:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// * **Check 2:** Checks that the there is enough liquidity to perform the withdraw.
        /// 
        /// # Arguments:
        /// 
        /// * `resource_address` (ResourceAddress) - The address of the resource to withdraw from the liquidity pool.
        /// * `amount` (Decimal) - The amount of tokens to withdraw from the liquidity pool.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the withdrawn tokens.
        fn withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal
        ) -> Bucket {
            // Performing the checks to ensure tha the withdraw can actually go through
            self.assert_belongs_to_pool(resource_address, String::from("Withdraw"));
            
            // Getting the vault of that resource and checking if there is enough liquidity to perform the withdraw.
            let vault: &mut Vault = self.vaults.get_mut(&resource_address).unwrap();
            assert!(
                vault.amount() >= amount,
                "[Withdraw]: Not enough liquidity available for the withdraw."
            );

            return vault.take(amount);
        }

        /// Adds liquidity to this liquidity pool in exchange for liquidity provider tracking tokens.
        /// 
        /// This method calculates the appropriate amount of liquidity that may be added to the liquidity pool from the
        /// two token buckets provided in this method call. This method then adds the liquidity and issues tracking 
        /// tokens to the liquidity provider to keep track of their percentage ownership over the pool. 
        /// 
        /// This method performs a number of checks before liquidity is added to the pool:
        /// 
        /// * **Check 1:** Checks that the buckets passed are of tokens that belong to this liquidity pool.
        /// * **Check 2:** Checks that the buckets passed are not empty.
        /// 
        /// From the perspective of adding liquidity, these are all of the checks that need to be done. The RaDEX 
        /// component does not need to perform any additional checks when liquidity is being added.
        /// 
        /// # Arguments:
        /// 
        /// * `token1` (Bucket) - A bucket containing the amount of the first token to add to the pool.
        /// * `token2` (Bucket) - A bucket containing the amount of the second token to add to the pool.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the remaining tokens of the `token1` type.
        /// * `Bucket` - A bucket of the remaining tokens of the `token2` type.
        /// * `Bucket` - A bucket of the tracking tokens issued to the liquidity provider.
        /// 
        /// # Note:
        /// 
        /// This method uses the ratio of the tokens in the reserve to the ratio of the supplied tokens to determine the
        /// appropriate amount of tokens which need to be supplied. To better explain it, let's use some symbols to make
        /// the ratios a little bit clearer. Say that `m` and `n` are the tokens reserves of the two tokens stored in 
        /// the vaults respectively. Say that `dm` and `dn` are positive non-zero `Decimal` numbers of the amount of 
        /// liquidity which the provider wishes to add to the liquidity pool. If `(m / n)/(dm / dn) = 1` then all of the
        /// tokens sent in the transactions will be added to the liquidity. However, what about the other cases where 
        /// this is not equal to one? We could say that we have three cases in total:
        /// 
        /// * `(m / n) = (dm / dn)` - There is no excess of tokens and all of the tokens given to the method may be 
        /// added to the liquidity pool. The no excess on both sides case could also happen if a liquidity pool has been
        /// emptied out and this is the new round of new liquidity being added. In this case, the buckets of tokens will
        /// be taken all with no excess or anything remaining.
        /// * `(m / n) < (dm / dn)` - In this case, there would be an excess of `dm` meaning that `dn` would be consumed
        /// fully while `dm` would be consumed partially.
        /// * `(m / n) > (dm / dn)` - In this case, there would be an excess of `dn` meaning that `dm` would be consumed
        /// fully while `dn` would be consumed partially.
        /// 
        /// This method takes into account all three of these cases and appropriately accounts for them.
        pub fn add_liquidity(
            &mut self,
            token1: Bucket,
            token2: Bucket,
        ) -> (Bucket, Bucket, Bucket) {
            // Checking if the tokens belong to this liquidity pool.
            self.assert_belongs_to_pool(token1.resource_address(), String::from("Add Liquidity"));
            self.assert_belongs_to_pool(token2.resource_address(), String::from("Add Liquidity"));

            // Checking that the buckets passed are not empty
            assert!(!token1.is_empty(), "[Add Liquidity]: Can not add liquidity from an empty bucket");
            assert!(!token2.is_empty(), "[Add Liquidity]: Can not add liquidity from an empty bucket");
            info!(
                "[Add Liquidity]: Requested adding liquidity of amounts, {}: {}, {}: {}", 
                token1.resource_address(), token1.amount(), token2.resource_address(), token2.amount()
            );

            let total_amount = token1.amount() + token2.amount();

            let token1_bucket_weight = token1.amount() / total_amount;

            let token2_bucket_weight = token2.amount() / total_amount;

            assert_eq!(token1_bucket_weight + token2_bucket_weight, self.token_1_weight + self.token_2_weight,
                "[Add Liquidity]: The amount provided exceeds the total weighted amount.
                The total weight is {:?}.
                You've provided {:?} of weight for Bucket 1 and {:?} for Bucket 2.",
                self.total_token_weight, token1_bucket_weight, token2_bucket_weight);
            
            assert_eq!(token1_bucket_weight, self.token_1_weight,
                "[Add Liquidity]: You've inputed {:?} as your weight for Token 1, but allocated {:?} for Token 1.
                The amount provided for Token 1 exceeds the weighted requirement.",
                self.token_1_weight, token1_bucket_weight);

            assert_eq!(token2_bucket_weight, self.token_2_weight,
                "[Add Liquidity]: You've inputed {:?} as your weight for Token 2, but allocated {:?} for Token 2.
                The amount provided for Token 1 exceeds the weighted requirement.",
                self.token_2_weight, token2_bucket_weight);   

            // Sorting out the two buckets passed and getting the values of `dm` and `dn`.
            let (mut bucket1, mut bucket2): (Bucket, Bucket) = sort_buckets(token1, token2);
            let dm: Decimal = bucket1.amount();
            let dn: Decimal = bucket2.amount();

            // Getting the values of m and n from the liquidity pool vaults
            let m: Decimal = self.vaults[&bucket1.resource_address()].amount();
            let n: Decimal = self.vaults[&bucket2.resource_address()].amount();
            info!(
                "[Add Liquidity]: Current reserves: {}: {}, {}: {}",
                bucket1.resource_address(), m, bucket2.resource_address(), n
            );

            // Computing the amount of tokens to deposit into the liquidity pool from each one of the buckets passed
            let (amount1, amount2): (Decimal, Decimal) = if ((m == Decimal::zero()) | (n == Decimal::zero())) | ((m / n) == (dm / dn)) { // Case 1
                (dm, dn)
            } else if (m / n) < (dm / dn) { // Case 2
                (dn * m / n, dn)
            } else { // Case 3
                (dm, dm * n / m)
            };
            info!(
                "[Add Liquidity]: Liquidity amount to add: {}: {}, {}: {}", 
                bucket1.resource_address(), amount1, bucket2.resource_address(), amount2
            );

            // Depositing the amount of tokens calculated into the liquidity pool
            self.deposit(bucket1.take(amount1));
            self.deposit(bucket2.take(amount2));

            // Computing the amount of tracking tokens that the liquidity provider is owed and minting them. In the case
            // that the liquidity pool has been completely emptied out (tracking_tokens_manager.total_supply() == 0)  
            // then the first person to supply liquidity back into the pool again would be given 100 tracking tokens.
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let tracking_amount: Decimal = if tracking_tokens_manager.total_supply() == Decimal::zero() { 
                dec!("100.00") 
            } else {
                amount1 * tracking_tokens_manager.total_supply() / m
            };
            let tracking_tokens: Bucket = self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens_manager.mint(tracking_amount)
            });
            info!("[Add Liquidity]: Owed amount of tracking tokens: {}", tracking_amount);

            // Returning the remaining tokens from `token1`, `token2`, and the tracking tokens
            return (bucket1, bucket2, tracking_tokens);
        }

        /// Removes the percentage of the liquidity owed to this liquidity provider.
        /// 
        /// This method is used to calculate the amount of tokens owed to the liquidity provider and take them out of
        /// the liquidity pool and return them to the liquidity provider. If the liquidity provider wishes to only take
        /// out a portion of their liquidity instead of their total liquidity they can provide a `tracking_tokens` 
        /// bucket that does not contain all of their tracking tokens (example: if they want to withdraw 50% of their
        /// liquidity, they can put 50% of their tracking tokens into the `tracking_tokens` bucket.). When the liquidity
        /// provider is given the tokens that they are owed, the tracking tokens are burned.
        /// 
        /// This method performs a number of checks before liquidity removed from the pool:
        /// 
        /// * **Check 1:** Checks to ensure that the tracking tokens passed do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tracking_tokens` (Bucket) - A bucket of the tracking tokens that the liquidity provider wishes to 
        /// exchange for their share of the liquidity.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A Bucket of the share of the liquidity provider of the first token.
        /// * `Bucket` - A Bucket of the share of the liquidity provider of the second token.
        pub fn remove_liquidity(
            &mut self,
            tracking_tokens: Bucket
        ) -> (Bucket, Bucket) {
            // Checking the resource address of the tracking tokens passed to ensure that they do indeed belong to this
            // liquidity pool.
            assert_eq!(
                tracking_tokens.resource_address(), self.tracking_token_address,
                "[Remove Liquidity]: The tracking tokens given do not belong to this liquidity pool."
            );

            // Calculating the percentage ownership that the tracking tokens amount corresponds to
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let percentage: Decimal = tracking_tokens.amount() / tracking_tokens_manager.total_supply();

            // Burning the tracking tokens
            self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens.burn();
            });

            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let bucket1: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() * percentage);
            let bucket2: Bucket = self.withdraw(addresses[1], self.vaults[&addresses[1]].amount() * percentage);

            return (bucket1, bucket2);
        }

        /// Performs the swap of tokens and takes the pool fee in the process
        /// 
        /// This method is used to perform the swapping of one token with another token. This is a low level method
        /// that does not perform a lot of checks on the tokens being swapped, slippage, or things of that sort. It is
        /// up to the caller of the this method (typically another method / function) to perform the checks needed. 
        /// When swaps are performed through this method, the associated fee of the pool is taken when this swap method
        /// is called.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the input tokens that will be swapped for other tokens.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        pub fn swap(
            &mut self,
            tokens: Bucket
        ) -> Bucket {
            // Checking if the tokens belong to this liquidity pool.
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap"));
            info!("[Swap]: K before swap: {}", self.k());

            // Calculating the output amount for the given input amount of tokens and withdrawing it from the vault
            let output_amount: Decimal = self.calculate_output_amount(tokens.resource_address(), tokens.amount());
            let output_tokens: Bucket = self.withdraw(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );

            // Depositing the tokens into the liquidity pool and returning a bucket of the swapped tokens.
            self.deposit(tokens);
            info!("[Swap]: K after swap: {}", self.k());
            return output_tokens;
        }

        /// Swaps all of the given tokens for the other token.
        /// 
        /// This method is used to swap all of the given token (let's say Token A) for their equivalent amount of the
        /// other token (let's say Token B). This method supports slippage in the form of the `min_amount_out` where
        /// the caller is given the option to specify the minimum amount of Token B that they're willing to accept for
        /// the swap to go through. If the output amount does not satisfy the `min_amount_out` specified by the user 
        /// then this method fails and all of the parties involved get their tokens back.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the input tokens that will be swapped for other tokens.
        /// * `min_amount_out` (Decimal) - The minimum amount of tokens that the caller is willing to accept before the 
        /// method fails.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        pub fn swap_exact_tokens_for_tokens(
            &mut self,
            tokens: Bucket,
            min_amount_out: Decimal
        ) -> Bucket {
            // Checking that the bucket passed does indeed belong to this liquidity pool
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap Exact"));
            
            // Performing the token swap and checking if the amount is suitable for the caller or not. This is one of 
            // the best and coolest things that I have seen in Scrypto so far. Even though in the `self.swap(tokens)` 
            // line to took the tokens from the vault and are now ready to give it to the user, if the assert statement
            // fails then everything that took place in this method call goes back to how it was before hand. 
            // Essentially reverting history and going back in time to say that the withdraw from the vault never took
            // place and that the funds are still in the vault.
            let output_tokens: Bucket = self.swap(tokens);
            assert!(output_tokens.amount() >= min_amount_out, "[Swap Exact]: min_amount_out not satisfied.");

            return output_tokens;
        }

        /// Swaps tokens for a specific amount of tokens
        /// 
        /// This method is used when the user wants to swap a token for a specific amount of another token. This method
        /// calculates the input amount required to get the desired output and if the amount required is provided in the
        /// tokens bucket then the swap takes place and the user gets back two buckets: a bucket of the remaining input
        /// tokens and another bucket of the swapped tokens.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the tokens that the user wishes to swap.
        /// * `output_amount` (Decimal) - A decimal of the specific amount of output that the user wishes to receive 
        /// from this swap.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        /// * `Bucket` - A bucket of the remaining input tokens.
        pub fn swap_tokens_for_exact_tokens(
            &mut self,
            mut tokens: Bucket,
            output_amount: Decimal
        ) -> (Bucket, Bucket) {
            // Checking that the bucket passed does indeed belong to this liquidity pool
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap For Exact"));

            // Calculating the amount of input tokens that would be required to produce the desired amount of output 
            // tokens
            let input_required: Decimal = self.calculate_input_amount(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );
            assert!(
                tokens.amount() >= input_required,
                "[Swap For Exact]: Not enough input for the desired amount of output."
            );

            // Depositing the amount of input required into the vaults and taking out the requested amount
            info!("[Swap For Exact]: K before swap: {}", self.k());
            self.deposit(tokens.take(input_required));
            let output_tokens: Bucket = self.withdraw(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );
            info!("[Swap For Exact]: K after swap: {}", self.k());
            info!("[Swap For Exact]: Amount gievn out: {}", output_tokens.amount());
            return (output_tokens, tokens);
        }

        /// This method is not called by the user but through the LiquidityPoolDAO component.
        /// 
        /// It is used to act on the resolution of the governance proposal.
        /// 
        /// This method performs a few checks:
        /// 
        /// * **Check 1:** - Checks that the Proposal NFT belongs to this protocol.
        /// * **Check 2:** - Checks that bucket is not empty.
        /// 
        /// There are a few scenarios:
        /// 
        /// * **Scenario 1:** - The proposal has been passed.
        /// 
        /// * **Resolution:** - The pool parameters has changed and the Proposal NFT is burnt.
        /// 
        /// * **Scenario 2:** - The proposal has failed.
        /// 
        /// * **Resolution:** - The Proposal NFT is simply burnt.
        /// 
        /// * **Scenario 3:** - The proposal is still in process.
        /// 
        /// * **Resolution:** - Nothing happens.
        /// 
        /// # Notes:
        /// 
        /// * When the pool parameter is changed, nothing happens to the current supply in the pool.
        /// * One asset would be more expensive or cheaper priced than the rest of the market when the weight changes. 
        /// * Market forces takes place and arbitrageurs can take advantage of the free value until it meets equilibrium with the rest of the market.
        pub fn resolve_proposal(
            &mut self,
            proposal: Bucket
        )
        {
            assert_eq!(self.nft_proposal_address.contains(&proposal.resource_address()), true,
                "The NFT does not belong to this protocol.");

            assert_ne!(proposal.is_empty(), true,
                "Cannot pass an empty bucket.");
            
            // Retrieves the Proposal NFT data.
            let proposal_data = proposal.non_fungible::<Proposal>().data();

            let resolution = proposal_data.resolution;

            // Matches the resolution to its respective selections.
            match resolution {
                // Scenario 1
                Resolution::Passed => {
                    // Pool parameter changes.
                    self.token_1_weight = proposal_data.token_1_weight;
                    self.token_2_weight = proposal_data.token_2_weight;
                    self.fee_to_pool = proposal_data.fee_to_pool;
          
                    // Proposal NFT is burnt.
                    self.nft_proposal_admin.authorize(|| proposal.burn());
                }
                // Scenario 2
                Resolution::Failed => {
                    // Proposal NFT is burnt.
                    self.nft_proposal_admin.authorize(|| proposal.burn());

                }
                // Scenario 3
                Resolution::InProcess => {
                    info!("The vote is still in progress.");
                }
            }
        }

        /// Pushes the Proposal NFT resource address to the component state.
        pub fn push_nft_proposal_address(
            &mut self,
            nft_proposal_address: ResourceAddress,
        )
        {
            self.nft_proposal_address.push(nft_proposal_address);
        }
        
        /// Retrieves component state data.
        pub fn token_1_weight(
            &self,
        ) -> Decimal
        {
            return self.token_1_weight
        }

        /// Retrieves component state data.
        pub fn token_2_weight(
            &self,
        ) -> Decimal
        {
            return self.token_2_weight
        }

        /// Retrieves component state data.
        pub fn fee_to_pool(
            &self,
        ) -> Decimal
        {
            return self.fee_to_pool
        }

        /// Retrieves component state data.
        pub fn tracking_token_address(
            &self
        ) -> ResourceAddress
        {
            return self.tracking_token_address
        }
    }
}