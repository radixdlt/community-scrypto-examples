use scrypto::prelude::*;
use crate::foldedleverage::User;
use crate::user_management::*;

// Still need to figure out how to calculate fees and interest rate


blueprint! {
    struct LendingPool {
        vaults: HashMap<ResourceAddress, Vault>,
        fees: Vault,
        users: HashMap<ResourceAddress, User>,
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

            //Inserting pool info into HashMap
            let lending_pool: Bucket = initial_funds;

            let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            vaults.insert(lending_pool.resource_address(), Vault::with_bucket(lending_pool));

            //Instantiate lending pool component

            let lending_pool: ComponentAddress = Self {
                vaults: vaults,
                fees: Vault::new(funds_resource_def),
                users: HashMap::new(),
                user_management_address: UserManagement::new(),
            }
            .instantiate().globalize();
            return lending_pool;
        }

        // Contribute tokens to the pool in exchange of an LP token
        pub fn deposit(&mut self, user_auth: Proof, deposit_amount: Bucket) {

            let user_management: UserManagement = self.user_management_address.into();
            user_management.assert_user_exists(user_auth, String::from("User doesn't belong to this lending protocol."));

            assert!(self.users.contains_key(&user_auth.resource_address()), 
            "User does not belong to this lending protocol."
        );

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

        // Give LP token back to get portion of the pool and fees
        // Need to think about how to withdraw
        pub fn borrow(&mut self, user_auth: Proof, borrow_amount: Decimal) -> Bucket {

            // Check if the NFT belongs to this lending protocol.
            assert!(self.users.contains_key(&user_auth.resource_address()), 
            "User does not belong to this lending protocol."
        );

            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let borrow_amount: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() - borrow_amount);


            borrow_amount
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
        pub fn redeem(
            &mut self,
            user_auth: Proof,
            token_requested: ResourceAddress,
            redeem_amount: Decimal,
        ) -> Bucket {

            // Check if the NFT belongs to this lending protocol.
            assert!(self.users.contains_key(&user_auth.resource_address()), 
            "User does not belong to this lending protocol.");
            
            let user_management: UserManagement = self.user_management_address.into();

            // Check if deposit withdrawal request has no lien
            user_management.check_lien(user_auth, token_requested);
            
            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let bucket1: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() - redeem_amount);

            return bucket1;
        }
    }
}


