use scrypto::prelude::*;
use crate::foldedleverage::*;

blueprint! {
    struct LendingPool {
        vaults: HashMap<ResourceAddress, Vault>,
        fees: Vault,
        tracking_token_admin_badge: Vault,
        tracking_token_address: ResourceAddress,
        users: HashMap<(ResourceAddress,Decimal), User>

    }

    impl LendingPool {
        pub fn new(initial_funds: Bucket) -> (ComponentAddress, Bucket) {

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

            // Create badge that will be used to mint and burn LP tokens
            let tracking_token_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Tracking Token Admin Badge")
                .metadata("symbol", "TTAB")
                .metadata("description", "This is an admin badge that has the authority to mint and burn tracking tokens")
                .initial_supply(1);

            // Creating token address identifier for tracking_tokens
            let token_address: ResourceAddress = initial_funds.resource_address();

            // Creating the tracking tokens and minting the amount owed to the initial liquidity provider
            let tracking_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("{} LP Tracking Token", token_address))
                .metadata("symbol", "TT")
                .metadata("description", "A tracking token used to track the percentage ownership of liquidity providers over the liquidity pool")
                .mintable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .burnable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .initial_supply(100);

            //Inserting pool info into HashMap
            let lending_pool: Bucket = initial_funds;

            let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            vaults.insert(lending_pool.resource_address(), Vault::with_bucket(lending_pool));

            //Instantiate lending pool component
            let lending_pool: ComponentAddress = Self {
                vaults: vaults,
                fees: Vault::new(funds_resource_def),
                tracking_token_admin_badge: Vault::with_bucket(tracking_token_admin_badge),
                tracking_token_address: tracking_tokens.resource_address(),
            }
            .instantiate().globalize();
            return (lending_pool, tracking_tokens);
        }

        // Contribute tokens to the pool in exchange of an LP token
        pub fn deposit(&mut self, collateral: Bucket) -> Bucket {

            // LP Tracking tokens
            let amount: Decimal = self.vaults[&collateral.resource_address()].amount();

            // Computing the amount of tracking tokens that the liquidity provider is owed and minting them. In the case
            // that the liquidity pool has been completely emptied out (tracking_tokens_manager.total_supply() == 0)  
            // then the first person to supply liquidity back into the pool again would be given 100 tracking tokens.
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(self.tracking_token_address); //Why did he write it this way? What does the resource manager do?
            let tracking_amount: Decimal = if tracking_tokens_manager.total_supply() == Decimal::zero() { 
                dec!("100.00") 
            } else {
                amount / tracking_tokens_manager.total_supply()
            };
            let tracking_tokens: Bucket = self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens_manager.mint(tracking_amount)
            });
            info!("[Add Liquidity]: Owed amount of tracking tokens: {}", tracking_amount);

            // Deposits collateral
            self.vaults.get_mut(&collateral.resource_address()).unwrap().put(collateral);


            return tracking_tokens;
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
        pub fn borrow(&mut self, user_auth: Bucket, tracking_tokens: Bucket) -> Bucket {

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
            let collateral: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() * percentage);


            collateral
        }
    }
}


