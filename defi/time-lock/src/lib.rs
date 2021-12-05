use scrypto::prelude::*;


blueprint! {
    struct TimeLock {
        /// Mint authorization to TL badges.
        tl_minter_vault: Vault,
        tl_minter_badge: ResourceDef,
        // Minted badges
        minted: HashMap<Address, (Decimal, u64)>,

        // Collected fees in XRD.
        collected_fees: Vault,
        
        // Locked XRD
        locked_xrd: Vault,

        // fee in percents
        fee_percent: Decimal,
    }

    impl TimeLock {
       
        pub fn new(fee: Decimal) -> (Component, Bucket) {

            let tl_minter_bucket = ResourceBuilder::new()
                .metadata("name", "TL Badge Mint Auth")
                .new_badge_fixed(2);

            let tl_minter_resource_def = tl_minter_bucket.resource_def();
            let tl_minter_return_bucket: Bucket = tl_minter_bucket.take(1); // Return this badge to the caller
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

            (component, tl_minter_return_bucket)
        }

        
        /// Lock XRD for a certain time.
        pub fn lock(&mut self, lock_tokens: Bucket, duration: u64) -> Bucket{

            let amount = lock_tokens.amount();

            scrypto_assert!(amount != Decimal::zero(), "You cannot lock zero amount");
            scrypto_assert!(duration != 0, "You cannot lock with a zero duration");

            
            // Setup the end time.
            let end_time = Context::current_epoch() + duration;

            // fees calculation
            let fee_amount = amount * self.fee_percent/100;
            // setup fees to be taken from the payment
            let fee_tokens = lock_tokens.take(fee_amount);

            let available = lock_tokens.amount();
            
            // Put fees in collected XRD.
            self.collected_fees.put(fee_tokens);

            // Mint TL badge with locked amount and end epoch as metadata
            let tl_resource_def = ResourceBuilder::new()
                .metadata("name", "Time lock badge")
                .metadata("amount", available.to_string())
                .metadata("ends", end_time.to_string())
                .new_badge_mutable(self.tl_minter_vault.resource_def());

            let tl_badge = self.tl_minter_vault.authorize(|badge| {
                tl_resource_def
                    .mint(1, badge)
            });

            // store new badge address
            self.minted.insert(tl_resource_def.address(), (available, end_time));

            // put the rest amount of tokens to the locked vault
            self.locked_xrd.put(lock_tokens);
            tl_badge
        }

        pub fn release(&mut self, tl_badge: Bucket) -> Bucket {
            let resource_def = tl_badge.resource_def();
    
            let returns = Bucket::new(RADIX_TOKEN);
            match self.minted.get(&resource_def.address()) {
                Some(&value) => {
                info!("current epoch {}", Context::current_epoch());
                scrypto_assert!(Context::current_epoch() > value.1, "Release time not yet over, wait for a bit longer"); 
                scrypto_assert!(value.0 > Decimal::zero(), "Release amount is zero");

                // Burn the TL badge
                self.tl_minter_vault.authorize(|badge| {
                    tl_badge.burn(badge);
                });
                // update mapping
                self.minted.remove(&resource_def.address());

                returns.put(self.locked_xrd.take(value.0));
                },
                _ => scrypto_abort("no mints found with provided badge")
            }
            
            // Return the withdrawn tokens
            returns
        }

        // #[auth(tl_minter_badge)]
        pub fn claim(&mut self) -> Bucket {
            self.collected_fees.take_all()
        }
    }
}
