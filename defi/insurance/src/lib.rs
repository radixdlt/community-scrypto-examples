use scrypto::prelude::*;


blueprint! {
    struct Insurance {
        /// Mint authorization to TL tokens.
        org_vault: Vault,
        org_badge: ResourceAddress,

        // Non locked assets
        assets_pool: Vault,
        // Locked assets by policies
        locked_pool: Vault,
       
        // HashMap of policy address and details
        policies: HashMap<ResourceAddress, Vault>,
        // HashMap of insurer and policy vaults HashMap
        purchases: HashMap<ComponentAddress, HashMap<ResourceAddress, Vault>>,

        // A vector which we use to store all of the dead vaults
        dead_vaults: Vec<Vault>
    }

    impl Insurance {
        
        // Create new Insurance component
        pub fn new(base_assets: Bucket) -> (ComponentAddress, Bucket) {
            assert!(base_assets.amount() > Decimal::zero(), "Base assets cannot be zero");
            assert!(base_assets.resource_address() == RADIX_TOKEN.into(), "You must use Radix (XRD).");

            // Org/Minter badge
            let mut org_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Org Mint Auth")
                .initial_supply(2);

            let org_resource_address = org_bucket.resource_address();
            let org_return_bucket: Bucket = org_bucket.take(1); // Return this badge to the caller

            let assets_address = base_assets.resource_address();
            let component = Self {
                org_vault: Vault::with_bucket(org_bucket),
                org_badge: org_resource_address,
                assets_pool: Vault::with_bucket(base_assets),
                locked_pool: Vault::new(assets_address),
                policies:HashMap::new(),
                purchases: HashMap::new(),
                dead_vaults: Vec::new(),
            }
            .instantiate()
            .globalize();
            (component,org_return_bucket)
        }

        // Create policy
        pub fn make_policy(&mut self, policy_type: String, coverage: Decimal, price:Decimal, duration:u64, supply: Decimal){
            assert!(coverage > Decimal::zero(), "Coverage cannot be zero");
            assert!(price > Decimal::zero(), "Price cannot be zero");
            assert!(duration > 0, "Duration cannot be zero");
            assert!(supply > Decimal::zero(), "Supply cannot be zero");

            assert!(self.assets_pool.amount() >= coverage * supply, "You don't have enough assets to cover this supply");

            // policy badge
            let mut ip_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Insurance Policy badge")
                .metadata("symbol", policy_type)
                .metadata("price", price.to_string())
                .metadata("coverage", coverage.to_string())
                .metadata("duration", duration.to_string())
                .mintable(rule!(require(self.org_vault.resource_address())), LOCKED)
                .burnable(rule!(require(self.org_vault.resource_address())), LOCKED)
                .no_initial_supply();

            // mint badges
            let bucket = self.org_vault.authorize(|| {
                borrow_resource_manager!(ip_resource_address).mint(supply)
            });

            // new vault
            let vault = Vault::with_bucket(bucket);
            self.policies.insert(ip_resource_address, vault);

            // lock assets
            let locked = self.assets_pool.take(coverage * supply);
            self.locked_pool.put(locked)
        }

        // Purchase specific policy by address
        pub fn purchase(&mut self, policy_address: ResourceAddress, insurer: ComponentAddress, mut bucket: Bucket) -> Bucket {
            assert!(self.policies.contains_key(&policy_address), "No policy found");
            assert!(bucket.resource_address() == RADIX_TOKEN.into(), "You must purchase policies with Radix (XRD).");
            
            // Don't allow an insurer to buy the exact same policy again
            let purchases = self.purchases.entry(insurer).or_insert(HashMap::new());
            assert!(!purchases.contains_key(&policy_address), "The insurer already has this policy");

            // check if we have free policy in the supply
            let policies = self.policies.get_mut(&policy_address).unwrap();
            assert!(!policies.is_empty(), "No available policies");

            // take one policy from the supply
            let policy = policies.take(1);
            // get metadata
            let metadata = borrow_resource_manager!(policy.resource_address()).metadata();
            // check price
            let price:Decimal = metadata["price"].parse().unwrap();
            assert!(bucket.amount() >= price, "Not enough amount to purchase this policy");

            // check duration and setup the end epoch
            let duration:u64 = metadata["duration"].parse().unwrap();
            let expires = Runtime::current_epoch() + duration;

            // get coverage
            let coverage:Decimal = metadata["coverage"].parse().unwrap();

            // build and mint policy badge with supply equals to coverage
            let mut ip_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Insurance Purchase Badge")
                .metadata("symbol", "IPB")
                .metadata("expires", expires.to_string())
                .metadata("policy", policy_address.to_string())
                .mintable(rule!(require(self.org_vault.resource_address())), LOCKED)
                .burnable(rule!(require(self.org_vault.resource_address())), LOCKED)
                .no_initial_supply();

            let ip_badge = self.org_vault.authorize(|| {
                borrow_resource_manager!(ip_resource_address).mint(coverage)
            });
            
            // burn taken policy
            self.org_vault.authorize(|| {
                policy.burn();
            });

            // update the vault
            let vault = Vault::with_bucket(ip_badge);
            purchases.insert(policy_address, vault);           

            // take payment
            let payment = bucket.take(price);
            self.assets_pool.put(payment);

            // return the rest bucket
            bucket
        }

        // Approve payment with specific amount to the insurer using policy address 
        //#[auth(org_badge)]
        pub fn approve(&mut self, insurer: ComponentAddress, policy_address: ResourceAddress, amount: Decimal){
            assert!(self.purchases.contains_key(&insurer), "No vault found for this insurer");

            let purchases = self.purchases.get_mut(&insurer).unwrap();
            assert!(purchases.contains_key(&policy_address), "No such policy found for this insurer");

            let purchase = purchases.get_mut(&policy_address).unwrap();
            let bucket = purchase.take(amount);

            // get metadata and check if policy is not expired
            let metadata = borrow_resource_manager!(bucket.resource_address()).metadata();
            let expires:u64 = metadata["expires"].parse().unwrap();
            assert!(Runtime::current_epoch() <= expires, "Policy is expired");

            let supply = bucket.amount();
            // Burn purchase badges
            self.org_vault.authorize(|| {
                bucket.burn();
            });
        
            // send XRD from locked pool to the insurer
            borrow_component!(insurer).call::<()>("deposit", args![self.locked_pool.take(supply)]);
        }

        // Burn expired purchases to release locked assets
        //#[auth(org_badge)]
        pub fn burn_purchases(&mut self, insurer: ComponentAddress, policy_address: ResourceAddress) {
            assert!(self.purchases.contains_key(&insurer), "No vault found for this insurer");

            let purchases = self.purchases.get_mut(&insurer).unwrap();
            assert!(purchases.contains_key(&policy_address), "No such policy found for this insurer");

            // get all free badges from supply
            let purchase = purchases.get_mut(&policy_address).unwrap();
            let bucket = purchase.take_all();

            // prevent burning non-expired badges
            let metadata = borrow_resource_manager!(bucket.resource_address()).metadata();
            let expires:u64 = metadata["expires"].parse().unwrap();
            assert!(Runtime::current_epoch() > expires, "Policy is not expired");

            let supply = bucket.amount();
            // Burn the the rest purchases
            self.org_vault.authorize(|| {
                bucket.burn();
            });

            // unlock assets
            self.assets_pool.put(self.locked_pool.take(supply));

            // clear purchases
            self.dead_vaults.push(purchases.remove(&policy_address).unwrap());
        }

        // Burn unused policies
        //#[auth(org_badge)]
        pub fn burn_policies(&mut self, policy_address: ResourceAddress) {
            info!("Policy Address: {}", policy_address);
            info!("Policies: {:?}", self.policies);
            assert!(self.policies.contains_key(&policy_address), "No policy found");

            // take the rest supply of policy badges
            let policies = self.policies.get_mut(&policy_address).unwrap();
            let bucket = policies.take_all();

            // get coverage and calculate the volume of XRD that should be released
            let metadata = borrow_resource_manager!(bucket.resource_address()).metadata();
            let coverage:Decimal = metadata["coverage"].parse().unwrap();
            let volume = bucket.amount()*coverage;

            // Burn the the rest purchases
            self.org_vault.authorize(|| {
                bucket.burn();
            });

            // unlock assets
            self.assets_pool.put(self.locked_pool.take(volume));

            // clear policies
            self.dead_vaults.push(self.policies.remove(&policy_address).unwrap());
        }

        /// Org assets methods
        pub fn deposit(&mut self, bucket: Bucket) {
            assert!(bucket.resource_address() == RADIX_TOKEN.into(), "You must deposit with Radix (XRD).");
            assert!(bucket.amount() > Decimal::zero(), "You cannot deposit zero amount");

            self.assets_pool.put(bucket)
        }

        pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
            assert!(self.assets_pool.amount() >= amount, "Withdraw amount is bigger than available assets");

            self.assets_pool.take(amount)
        }

        pub fn assets(&mut self) -> Decimal {
            self.assets_pool.amount()
        }

        pub fn locked(&mut self) -> Decimal {
            self.locked_pool.amount()
        }
    }
}
