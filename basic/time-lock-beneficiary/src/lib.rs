use scrypto::prelude::*;

#[blueprint]
mod locker {
    struct Lock {
        xrd_vault: Vault,
        admin_badge_vault: Vault,
        admin_badge_resource_address: ResourceAddress,
        beneficiary_badge_resource_address: ResourceAddress,
        timer: u64,
    }

    impl Lock {
        pub fn instantiate_lock() -> (ComponentAddress, Bucket) {
            //Create a new admin token/badge with initial supply of 2, one for the caller and one for the component
            let mut admin_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(2);

            //Create a bucket to return to caller
            let return_admin_bucket: Bucket = admin_bucket.take(1);
            //Create a varible for the resource address
            let return_admin_address: ResourceAddress = admin_bucket.resource_address();

            //Create a new beneficiary token/badge with no initial supply. This will be minted later.
            let beneficiary_address = ResourceBuilder::new_fungible()
                .metadata("name", "Beneficiary Badge")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(admin_bucket.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();

            let component: LockComponent = Self {
                xrd_vault: Vault::new(RADIX_TOKEN),
                admin_badge_vault: Vault::with_bucket(admin_bucket),
                admin_badge_resource_address: return_admin_address,
                beneficiary_badge_resource_address: beneficiary_address,
                timer: 0,
            }
            .instantiate();
            
            //return_badge_bucket
            (component.globalize(), return_admin_bucket)
        }
        
        //Mint a new Beneficiary Badge token as admin. This should be sent to the beneficiary address.
        pub fn mint_badge(&mut self, badge_proof: Proof) -> Bucket {
            // Validate the admin badge proof
            badge_proof.validate_proof(self.admin_badge_resource_address).expect("Invalid proof");
            
            // Authorize minting of the Beneficiary Badge token
            self.admin_badge_vault.authorize(|| {
                let resource_manager = borrow_resource_manager!(self.beneficiary_badge_resource_address);
                let return_bucket = resource_manager.mint(1);
                return_bucket
            })
        }

        //Deposit XRD, anyone can deposit
        pub fn deposit_xrd(&mut self, xrd_bucket: Bucket) {
            self.xrd_vault.put(xrd_bucket)
        }

        //Withdraw XRD as admin
        pub fn withdraw_xrd_by_amount(&mut self, amount: Decimal, badge_proof: Proof) -> Bucket {
            //require admin badge
            badge_proof.validate_proof(self.admin_badge_resource_address).expect("Invalid proof");
            //return xrd to caller
            self.xrd_vault.take(amount)
        }

        //Update timer as admin
        pub fn update_timer(&mut self, duration: u64, badge_proof: Proof) {
            //require admin badge
            badge_proof.validate_proof(self.admin_badge_resource_address).expect("Invalid proof");
            //Set end time as current epoch + duration
            let end_time = Runtime::current_epoch() + duration;
            //Update the timer in struct 
            self.timer = end_time;
        }

        //Get timer value
        pub fn get_timer(&self) -> u64 {
            self.timer
        }
    
        //Claim xrd as beneficiary if you can prove you have the badge and the timer has expired
        pub fn claim_xrd(&mut self, badge_proof: Proof) -> Bucket {

            badge_proof.validate_proof(self.beneficiary_badge_resource_address).expect("Invalid proof");
            
            let current_epoch = Runtime::current_epoch();

            assert!(current_epoch >= self.timer, "You cannot claim yet");
            self.xrd_vault.take(self.xrd_vault.amount())
        }

    }
}