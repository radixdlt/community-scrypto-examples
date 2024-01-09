use scrypto::prelude::*;

#[blueprint]
mod coindispenser {
    // setup of roles and list of the mehtods and their access restrictions.
    enable_method_auth! { 
        roles { 
            admin => updatable_by: [OWNER]; 
        },
        methods { 
            deposit => PUBLIC;
            redeem_coin => PUBLIC;
            keystore_swap => restrict_to: [admin];
            withdrawal => restrict_to: [admin];
            set_redeem_pair => restrict_to: [admin];
            reset_dispenser => restrict_to: [admin];
        }
    }
	
    struct CoinDispenser {
        // vaults containing any kind of incomming coins
        primairyvaults: KeyValueStore<ResourceAddress, Vault>,
        // redemption vault contains any kind of outgoing coins
        secondairyvaults: KeyValueStore<ResourceAddress, Vault>,
        // the swap ratio of the active coins
        ratio: Decimal,
        // active incomming coin // set to None to disable swap
        incomming: Option<ResourceAddress>,
        // active outgoing coin
        outgoing: Option<ResourceAddress>,
        // adminvault for badge
        adminvault: Vault,
    }

    impl CoinDispenser {

        pub fn instantiate() -> (Global<CoinDispenser>, FungibleBucket) {
            
            let mut my_admin_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
		   	    .metadata (metadata! { 
//					roles {
//						metadata_setter => OWNER;
//						metadata_setter_updater => rule!(deny_all);
//                      metadata_locker => rule!(deny_all);
//                      metadata_locker_updater => rule!(deny_all);
//					},	
					init {
						"name" => "CoinDispenser Admin Badge", locked;
						"description" => "Badge for CoinDispenser Component", locked;
                        "symbol" => "CDAB", locked;
					}
                })
                .mint_initial_supply(2);

            // put one admin badge and put in in the admin vault
            // just in case future actions require this badge to be part of the dApp.
            let local_admin_badge: FungibleBucket = my_admin_badge.take(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));

             // Instantiate a CoinDispenser component, 
            let component: Global<CoinDispenser> = Self {
                adminvault: Vault::with_bucket(local_admin_badge.into()),
                ratio: dec!("0.99"),
                incomming: None,
				outgoing: None,
                primairyvaults: KeyValueStore::new(),
                secondairyvaults: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
			.roles( 
                roles!(
                    admin => admin_rule;
                )
            )
            .globalize();
            (component, my_admin_badge)
        }

        // this is an admin function to move resources between keyvaluestores.
        // the secondairy vaults are the hot-vaults for the coin dispensor
        // more to primairy vaults for admin retreival.
        pub fn keystore_swap(&mut self, targetaddress: ResourceAddress) {
            assert!(!self.secondairyvaults.get(&targetaddress).is_none(),
                "The specified coin does not exist in the redemption vault");
		    let mut vin: KeyValueEntryRefMut<'_, Vault> = 
                self.secondairyvaults.get_mut(&targetaddress).unwrap();

            let cashout: Bucket = vin.take_all();

            if self.primairyvaults.get(&targetaddress).is_none() {
                let vout: Vault = Vault::with_bucket(cashout);
                self.primairyvaults.insert(targetaddress, vout);
            } else { 
                let mut vout: KeyValueEntryRefMut<'_, Vault> = 
                                self.primairyvaults.get_mut(&targetaddress).unwrap();
                vout.put(cashout);
            }
        }

        // this is an admin function to withdrawal a specific resource from the primairy vaults
        pub fn withdrawal(&mut self, targetaddress: ResourceAddress) -> Bucket{
            assert!(!self.primairyvaults.get(&targetaddress).is_none(),
                "The specified coin does not exist in this component");
		    let mut v: KeyValueEntryRefMut<'_, Vault> = 
                self.primairyvaults.get_mut(&targetaddress).unwrap();
            v.take_all()
        }

        // Deposit any resource in the secondairy vaults.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            if self.secondairyvaults.get(&resource_address).is_none() {
                let v: Vault = Vault::with_bucket(bucket);
                self.secondairyvaults.insert(resource_address, v);
            } else { 
                let mut v: KeyValueEntryRefMut<'_, Vault> = 
                                self.secondairyvaults.get_mut(&resource_address).unwrap();
                v.put(bucket);
            }
        }

        // admin function to define the redeem/receive combination and ratio
        // the first resourceaddress is the incomming coin; going to the primairy vault
        // the second resourceaddress is the outgoing coin; going out of the secondairy vault
        // The ratio is a reference of the amount of secondairy coins vs 1 primairy coin. 
        pub fn set_redeem_pair(&mut self, to_redeem: ResourceAddress,
                                          to_receive: ResourceAddress,
                                          ratio: Decimal){
            assert!(!self.secondairyvaults.get(&to_receive).is_none(),
                   "Outgoing vault does not exits, please deposit first");
            self.incomming = Some(to_redeem);
            self.outgoing = Some(to_receive);
            self.ratio = ratio;
        }

        // admin function to reset the dispenser
        pub fn reset_dispenser(&mut self){
            self.incomming = None;
            self.outgoing =  None;
            self.ratio = dec!("0.99");
        }

// public function to redeem one coin for another.
        // the actual public swap is done using this method.
        pub fn redeem_coin(&mut self, redeem: Bucket) -> Bucket {
            assert!((self.incomming == Some(redeem.resource_address())),
                "Coin does not match the selected redeem coin");
            assert!(!self.secondairyvaults.get(&self.outgoing.unwrap()).is_none(),
                    "No return coin selected!");          
            let total: Decimal = redeem.amount().checked_mul(self.ratio).unwrap();
//            let total: Decimal = redeem.amount() * self.ratio;

            let resource_address = redeem.resource_address();
            if self.primairyvaults.get(&resource_address).is_none() {
                let v: Vault = Vault::with_bucket(redeem.into());
                self.primairyvaults.insert(resource_address, v);
            } else { 
                let mut v: KeyValueEntryRefMut<'_, Vault> = 
                            self.primairyvaults.get_mut(&resource_address).unwrap();
                v.put(redeem);
            }
            let mut vout: KeyValueEntryRefMut<'_, Vault> = 
                            self.secondairyvaults.get_mut(&self.outgoing.unwrap()).unwrap();
            assert!((total < vout.amount()),
                "Not enough coin in the return wallet to complete the transaction");
            vout.take(total)
        }
    }
}
