use scrypto::prelude::*;


blueprint! {
    struct Insurance {
        /// Mint authorization to TL tokens.
        org_vault: Vault,
        org_badge: ResourceDef,

        // Non locked assets
        assets_pool: Vault,
        // Locked assets by policies
        locked_pool: Vault,
       
        // HashMap of policy address and details
        policies: HashMap<Address, Vault>,
        // HashMap of insurer and policy vaults HashMap
        purchases: HashMap<Address, HashMap<Address, Vault>>,

        // A vector which we use to store all of the dead vaults
        dead_vaults: Vec<Vault>
    }

    impl Insurance {
        
        // Create new Insurance component
        pub fn new(base_assets: Bucket) -> (Component, Bucket) {
            assert!(base_assets.amount() > Decimal::zero(), "Base assets cannot be zero");
            assert!(base_assets.resource_def() == RADIX_TOKEN.into(), "You must use Radix (XRD).");

            // Org/Minter badge
            let mut org_bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Org Mint Auth")
                .initial_supply_fungible(2);

            let org_resource_def = org_bucket.resource_def();
            let org_return_bucket: Bucket = org_bucket.take(1); // Return this badge to the caller

            let assets_def = base_assets.resource_def();
            let component = Self {
                org_vault: Vault::with_bucket(org_bucket),
                org_badge: org_resource_def,
                assets_pool: Vault::with_bucket(base_assets),
                locked_pool: Vault::new(assets_def),
                policies:HashMap::new(),
                purchases: HashMap::new(),
                dead_vaults: Vec::new(),
            }
            .instantiate();
            (component,org_return_bucket)
        }

        // Create policy
        // #[auth(org_badge)]
        pub fn make_policy(&mut self, policy_type: String, coverage: Decimal, price:Decimal, duration:u64, supply: Decimal){
            assert!(coverage > Decimal::zero(), "Coverage cannot be zero");
            assert!(price > Decimal::zero(), "Price cannot be zero");
            assert!(duration > 0, "Duration cannot be zero");
            assert!(supply > Decimal::zero(), "Supply cannot be zero");

            assert!(self.assets_pool.amount() >= coverage * supply, "You don't have enough assets to cover this supply");

            // policy badge
            let mut ip_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                 .metadata("name", "Insurance Policy badge")
                 .metadata("symbol", policy_type)
                 .metadata("price", price.to_string())
                 .metadata("coverage", coverage.to_string())
                 .metadata("duration", duration.to_string())
                 .flags(MINTABLE | BURNABLE)
                 .badge(self.org_vault.resource_def(), MAY_MINT | MAY_BURN)
                 .no_initial_supply();

            // mint badges
            let bucket = self.org_vault.authorize(|badge| {
                ip_resource_def
                    .mint(supply, badge)
            });

            // new vault
            let vault = Vault::with_bucket(bucket);
            self.policies.insert(ip_resource_def.address(), vault);

            // lock assets
            let locked = self.assets_pool.take(coverage * supply);
            self.locked_pool.put(locked)
        }

        // Purchase specific policy by address
        pub fn purchase(&mut self, policy_address: Address, insurer: Address, mut bucket: Bucket) -> Bucket {
            assert!(self.policies.contains_key(&policy_address), "No policy found");
            assert!(bucket.resource_def() == RADIX_TOKEN.into(), "You must purchase policies with Radix (XRD).");
            
            // Don't allow an insurer to buy the exact same policy again
            let purchases = self.purchases.entry(insurer).or_insert(HashMap::new());
            assert!(!purchases.contains_key(&policy_address), "The insurer already has this policy");

            // check if we have free policy in the supply
            let policies = self.policies.get_mut(&policy_address).unwrap();
            assert!(!policies.is_empty(), "No available policies");

            // take one policy from the supply
            let policy = policies.take(1);
            // get metadata
            let metadata = policy.resource_def().metadata();
            // check price
            let price:Decimal = metadata["price"].parse().unwrap();
            assert!(bucket.amount() >= price, "Not enough amount to purchase this policy");

            // check duration and setup the end epoch
            let duration:u64 = metadata["duration"].parse().unwrap();
            let expires = Context::current_epoch() + duration;

            // get coverage
            let coverage:Decimal = metadata["coverage"].parse().unwrap();

            // build and mint policy badge with supply equals to coverage
            let mut ip_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Insurance Purchase Badge")
                .metadata("symbol", "IPB")
                .metadata("expires", expires.to_string())
                .metadata("policy", policy_address.to_string())
                .flags(MINTABLE | BURNABLE)
                .badge(self.org_vault.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let ip_badge = self.org_vault.authorize(|badge| {
                ip_resource_def
                    .mint(coverage, badge)
            });
            

            // burn taken policy
            self.org_vault.authorize(|badge| {
                policy.burn_with_auth(badge);
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
        pub fn approve(&mut self, insurer: Address, policy_address: Address, amount: Decimal){
            assert!(self.purchases.contains_key(&insurer), "No vault found for this insurer");

            let purchases = self.purchases.get_mut(&insurer).unwrap();
            assert!(purchases.contains_key(&policy_address), "No such policy found for this insurer");

            let purchase = purchases.get_mut(&policy_address).unwrap();
            let bucket = purchase.take(amount);

            // get metadata and check if policy is not expired
            let metadata = bucket.resource_def().metadata();
            let expires:u64 = metadata["expires"].parse().unwrap();
            assert!(Context::current_epoch() <= expires, "Policy is expired");

            let supply = bucket.amount();
            // Burn purchase badges
            self.org_vault.authorize(|badge| {
                bucket.burn_with_auth(badge);
            });
        
            // send XRD from locked pool to the insurer
            Component::from(insurer).call::<()>("deposit", vec![scrypto_encode(&self.locked_pool.take(supply))]);
        }

        // Burn expired purchases to release locked assets
        //#[auth(org_badge)]
        pub fn burn_purchases(&mut self, insurer: Address, policy_address: Address) {
            assert!(self.purchases.contains_key(&insurer), "No vault found for this insurer");

            let purchases = self.purchases.get_mut(&insurer).unwrap();
            assert!(purchases.contains_key(&policy_address), "No such policy found for this insurer");

            // get all free badges from supply
            let purchase = purchases.get_mut(&policy_address).unwrap();
            let bucket = purchase.take_all();

            // prevent burning non-expired badges
            let metadata = bucket.resource_def().metadata();
            let expires:u64 = metadata["expires"].parse().unwrap();
            assert!(Context::current_epoch() > expires, "Policy is not expired");

            let supply = bucket.amount();
            // Burn the the rest purchases
            self.org_vault.authorize(|badge| {
                bucket.burn_with_auth(badge);
            });

            // unlock assets
            self.assets_pool.put(self.locked_pool.take(supply));

            // clear purchases
            self.dead_vaults.push(purchases.remove(&policy_address).unwrap());
        }

        // Burn unused policies
        //#[auth(org_badge)]
        pub fn burn_policies(&mut self, policy_address: Address) {
            info!("Policy Address: {}", policy_address);
            info!("Policies: {:?}", self.policies);
            assert!(self.policies.contains_key(&policy_address), "No policy found");

            // take the rest supply of policy badges
            let policies = self.policies.get_mut(&policy_address).unwrap();
            let bucket = policies.take_all();

            // get coverage and calculate the volume of XRD that should be released
            let metadata = bucket.resource_def().metadata();
            let coverage:Decimal = metadata["coverage"].parse().unwrap();
            let volume = bucket.amount()*coverage;

            // Burn the the rest purchases
            self.org_vault.authorize(|badge| {
                bucket.burn_with_auth(badge);
            });

            // unlock assets
            self.assets_pool.put(self.locked_pool.take(volume));

            // clear policies
            self.dead_vaults.push(self.policies.remove(&policy_address).unwrap());
        }

        /// Org assets methods
        // #[auth(org_badge)]
        pub fn deposit(&mut self, bucket: Bucket) {
            assert!(bucket.resource_def() == RADIX_TOKEN.into(), "You must deposit with Radix (XRD).");
            assert!(bucket.amount() > Decimal::zero(), "You cannot deposit zero amount");

            self.assets_pool.put(bucket)
        }

        // #[auth(org_badge)]
        pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
            assert!(self.assets_pool.amount() >= amount, "Withdraw amount is bigger than available assets");

            self.assets_pool.take(amount)
        }

        // #[auth(org_badge)]
        pub fn assets(&mut self) -> Decimal {
            self.assets_pool.amount()
        }

        // #[auth(org_badge)]
        pub fn locked(&mut self) -> Decimal {
            self.locked_pool.amount()
        }
    }
}
