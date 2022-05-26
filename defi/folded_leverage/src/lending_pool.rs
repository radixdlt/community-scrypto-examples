use scrypto::prelude::*;
use crate::user_management::*;

// Still need to figure out how to calculate fees and interest rate

// Should Lending Pool track how much has been borrowed?

blueprint! {
    struct LendingPool {
        // Vault for lending pool
        vaults: HashMap<ResourceAddress, Vault>,
        // Vault for tracking borrowed amounts in lending pool
        borrowed_vaults: HashMap<ResourceAddress, Vault>,
        // Badge for minting tracking tokens
        tracking_token_admin_badge: Vault,
        // Tracking tokens to be stored in borrowed_vaults whenever liquidity is removed from deposits
        tracking_token_address: ResourceAddress,
        fees: Vault,
        user_management_address: ComponentAddress,
    }

    impl LendingPool {
        pub fn new(user_auth: Proof, initial_funds: Bucket) -> ComponentAddress {

            assert_ne!(
                borrow_resource_manager!(initial_funds.resource_address()).resource_type(), ResourceType::NonFungible,
                "[Pool Creation]: Asset must be fungible."
            );

            assert!(
                !initial_funds.is_empty(), 
                "[Pool Creation]: Can't deposit an empty bucket."
            );

            // Define the resource address of the fees collected
            let funds_resource_def = initial_funds.resource_address();

            // Creating the admin badge of the liquidity pool which will be given the authority to mint and burn the
            // tracking tokens issued to the liquidity providers.
            let tracking_token_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Tracking Token Admin Badge")
                .metadata("symbol", "TTAB")
                .metadata("description", "This is an admin badge that has the authority to mint and burn tracking tokens")
                .metadata("lp_id", format!("{}", initial_funds.resource_address()))
                .initial_supply(1);

            // Creating the tracking tokens and minting the amount owed to the initial liquidity provider
            let tracking_tokens: ResourceAddress = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("Borrowed Tracking Token"))
                .metadata("symbol", "TT")
                .metadata("description", "A tracking token used to track the percentage ownership of liquidity providers over the liquidity pool")
                .metadata("lp_id", format!("{}", initial_funds.resource_address()))
                .mintable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .burnable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .no_initial_supply();
            
            //Inserting pool info into HashMap
            let lending_pool: Bucket = initial_funds;

            let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            let mut borrowed_vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            vaults.insert(lending_pool.resource_address(), Vault::with_bucket(lending_pool));
            borrowed_vaults.insert(lending_pool.resource_address(), Vault::new(tracking_tokens));

            //Instantiate lending pool component

            let lending_pool: ComponentAddress = Self {
                vaults: vaults,
                borrowed_vaults: borrowed_vaults,
                tracking_token_address: tracking_tokens,
                tracking_token_admin_badge: Vault::with_bucket(tracking_token_admin_badge),
                fees: Vault::new(funds_resource_def),
                user_management_address: UserManagement::new(),
            }
            .instantiate().globalize();
            return lending_pool;
        }

        // Mint tracking tokens every time there's a borrow and puts it in the borrowed vault
        fn mint_borrow(&mut self, token_address: ResourceAddress, amount: Decimal) {
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let tracking_tokens: Bucket = self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens_manager.mint(amount)
            });
            self.borrowed_vaults.get_mut(&token_address).unwrap().put(tracking_tokens)
        }

        // Burn tracking tokens every time there's a repayment
        fn burn_borrow(&mut self, token_address: ResourceAddress, amount: Decimal) {
            let burn_amount: Bucket = self.borrowed_vaults.get_mut(&token_address).unwrap().take(amount);
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let tracking_tokens = self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens_manager.burn(burn_amount)
            });
        }

        // Require token resource address?
        pub fn deposit(&mut self, user_auth: Proof, deposit_amount: Bucket) {

            let user_management: UserManagement = self.user_management_address.into();
            user_management.assert_user_exists(user_auth, String::from("User doesn't belong to this lending protocol."));


            // Deposits collateral
            self.vaults.get_mut(&deposit_amount.resource_address()).unwrap().put(deposit_amount);

        }

        /// Gets the resource addresses of the tokens in this liquidity pool and returns them as a `Vec<ResourceAddress>`.
        /// 
        /// # Returns:
        /// 
        /// `Vec<ResourceAddress>` - A vector of the resource addresses of the tokens in this liquidity pool.
        pub fn addresses(&self) -> Vec<ResourceAddress> {
            return self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>();
        }

        pub fn belongs_to_pool(
            &self, 
            address: ResourceAddress
        ) -> bool {
            return self.vaults.contains_key(&address);
        }

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

        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            // Performing the checks to ensure tha the withdraw can actually go through
            self.assert_belongs_to_pool(resource_address, String::from("Withdraw"));
            
            // Getting the vault of that resource and checking if there is enough liquidity to perform the withdraw.
            let vault: &mut Vault = self.vaults.get_mut(&resource_address).unwrap();
            assert!(
                vault.amount() >= amount,
                "[Withdraw]: Not enough liquidity available for the withdraw."
            );
            
            self.mint_borrow(resource_address, amount);
            return vault.take(amount);
        }

        pub fn borrow(&mut self, user_auth: Proof, token_address: ResourceAddress, amount: Decimal) -> Bucket {

            // Check if the NFT belongs to this lending protocol.
            let user_management: UserManagement = self.user_management_address.into();
            user_management.assert_user_exists(user_auth, String::from("User doesn't belong to this lending protocol."));

            // Minting tracking tokens to be deposited to borrowed_vault to track borrows from this pool
            self.mint_borrow(token_address, amount);

            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let borrow_amount: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() - amount);

            return borrow_amount
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
        pub fn redeem(&mut self, user_auth: Proof, token_address: ResourceAddress, amount: Decimal) -> Bucket {

            // Check if the NFT belongs to this lending protocol.
            let user_management: UserManagement = self.user_management_address.into();
            user_management.assert_user_exists(user_auth, String::from("User doesn't belong to this lending protocol."));

            // Check if deposit withdrawal request has no lien
            user_management.check_lien(user_auth, token_address);
            
            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let bucket: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() - amount);

            return bucket;
        }

        pub fn repay(&mut self, user_auth: Proof, token_address: ResourceAddress, amount: Bucket) {
            // Check if the NFT belongs to this lending protocol.
            let user_management: UserManagement = self.user_management_address.into();
            user_management.assert_user_exists(user_auth, String::from("User doesn't belong to this lending protocol."));

            // Burns the tracking token for borrowed amounts
            {let amount = amount.amount();
            self.burn_borrow(token_address, amount);}

            // Deposits the repaid loan back into the supply
            self.vaults.get_mut(&amount.resource_address()).unwrap().put(amount);
        }

        pub fn check_liquidity(&self, token_address: ResourceAddress) -> Decimal {

            let vault: &mut Vault = self.vaults.get_mut(&token_address).unwrap();
            let borrowed_vault: &mut Vault = self.borrowed_vaults.get_mut(&token_address).unwrap();
            let liquidity_amount: Decimal = borrowed_vault.amount() - vault.amount();
            return liquidity_amount
        }

        pub fn check_utilization_rate(&self, token_address: ResourceAddress) -> Decimal {
            let vault: &mut Vault = self.vaults.get_mut(&token_address).unwrap();
            let borrowed_vault: &mut Vault = self.borrowed_vaults.get_mut(&token_address).unwrap();
            let liquidity_amount: Decimal = borrowed_vault.amount() / vault.amount();
            return liquidity_amount
        }

        pub fn check_total_supplied(&self, token_address: ResourceAddress) -> Decimal {
            let vault: &mut Vault = self.vaults.get_mut(&token_address).unwrap();
            return vault.amount()
        }
        
        pub fn check_total_borrowed(&self, token_address: ResourceAddress) -> Decimal {
            let borrowed_vault: &mut Vault = self.borrowed_vaults.get_mut(&token_address).unwrap();
            return borrowed_vault.amount()
        }

    }
}


