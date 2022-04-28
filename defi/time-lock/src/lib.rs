use scrypto::prelude::*;


blueprint! {
    struct TimeLock {
        /// Mint authorization to TL badges.
        tl_minter_vault: Vault,
        tl_minter_badge: ResourceAddress,
        // Minted badges
        minted: HashMap<ResourceAddress, (Decimal, u64)>,

        // Collected fees in XRD.
        collected_fees: Vault,
        
        // Locked XRD
        locked_xrd: Vault,

        // fee in percents
        fee_percent: Decimal,
    }

    impl TimeLock {
       
        pub fn new(fee: Decimal) -> (ComponentAddress, Bucket) {

            let mut tl_minter_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "TL Badge Mint Auth")
                .initial_supply(2);

            let tl_minter_resource_def = tl_minter_bucket.resource_address();
            let tl_minter_return_bucket: Bucket = tl_minter_bucket.take(1); // Return this badge to the caller

            let access_rules = AccessRules::new()
                .method("claim", rule!(require(tl_minter_bucket.resource_address())))
                .default(rule!(allow_all));

            // Instantiate the Time Lock component.
            let component = Self {
                tl_minter_vault: Vault::with_bucket(tl_minter_bucket),
                tl_minter_badge: tl_minter_resource_def,
                minted: HashMap::new(),
                collected_fees: Vault::new(RADIX_TOKEN),
                locked_xrd: Vault::new(RADIX_TOKEN),
                fee_percent: fee
            }
            .instantiate();

            (component.add_access_check(access_rules).globalize(), tl_minter_return_bucket)
        }

        /// Lock XRD for a certain time.
        pub fn lock(&mut self, mut lock_tokens: Bucket, duration: u64) -> Bucket{

            let amount = lock_tokens.amount();

            assert!(amount != Decimal::zero(), "You cannot lock zero amount");
            assert!(duration != 0, "You cannot lock with a zero duration");

            
            // Setup the end time.
            let end_time = Runtime::current_epoch() + duration;

            // fees calculation
            let fee_amount = amount * self.fee_percent/ dec!("100");
            // setup fees to be taken from the payment
            let fee_tokens = lock_tokens.take(fee_amount);

            let available = lock_tokens.amount();
            
            // Put fees in collected XRD.
            self.collected_fees.put(fee_tokens);

            // Mint TL badge with locked amount and end epoch as metadata
            let mut tl_resource_def = ResourceBuilder::new_fungible()
                .metadata("name", "Time lock badge")
                .metadata("amount", available.to_string())
                .metadata("ends", end_time.to_string())
                .mintable(rule!(require(self.tl_minter_vault.resource_address())), LOCKED)
                .burnable(rule!(require(self.tl_minter_vault.resource_address())), LOCKED)
                .no_initial_supply();

            let tl_badge = self.tl_minter_vault.authorize(|| {
                borrow_resource_manager!(tl_resource_def).mint(1)
            });

            // store new badge address
            self.minted.insert(tl_resource_def, (available, end_time));

            // put the rest amount of tokens to the locked vault
            self.locked_xrd.put(lock_tokens);
            tl_badge
        }

        pub fn release(&mut self, tl_badge: Bucket) -> Bucket {
            let resource_def = tl_badge.resource_address();
    
            let mut returns = Bucket::new(RADIX_TOKEN);
            match self.minted.get(&resource_def) {
                Some(&value) => {
                info!("current epoch {}", Runtime::current_epoch());
                assert!(Runtime::current_epoch() > value.1, "Release time not yet over, wait for a bit longer"); 
                assert!(value.0 > Decimal::zero(), "Release amount is zero");

                // Burn the TL badge
                self.tl_minter_vault.authorize(|| {
                    tl_badge.burn();
                });
                // update mapping
                self.minted.remove(&resource_def);

                returns.put(self.locked_xrd.take(value.0));
                },
                _ => {
                    info!("no mints found with provided badge");
                    std::process::abort();
                }
            }
            
            // Return the withdrawn tokens
            returns
        }

        pub fn claim(&mut self) -> Bucket {
            self.collected_fees.take_all()
        }
    }
}