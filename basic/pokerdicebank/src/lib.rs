use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
pub struct Details {
    #[mutable]
    balance: Decimal,
    #[mutable]
    locked: bool,
    #[mutable]
    activetoken: Option<ResourceAddress>,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct Empty {
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct SwapTokenDetails{
    tokenvault: Vault,
    tokenswapratio: Decimal,
    activeswap: bool,
}

#[blueprint]
mod pokerdicebank {

    // setup of roles and list of the mehtods and their access restrictions.
    enable_method_auth! { 
        methods { 
            get_tokens => PUBLIC;
            redeem_tokens => PUBLIC;
            deposit_new => PUBLIC;
            deposit_add => PUBLIC;
            withdrawal => PUBLIC;
            admin_pokerdicebank_activate => restrict_to: [OWNER];
            admin_global_swap_activate => restrict_to: [OWNER];
            admin_token_swap_activate => restrict_to: [OWNER];
            admin_swap_config => restrict_to: [OWNER];
            admin_set_active_token => restrict_to: [OWNER];
            admin_swap_reset => restrict_to: [OWNER];
            admin_deposit_primairy => restrict_to: [OWNER];
            admin_deposit_secondairy => restrict_to: [OWNER];
            admin_withdrawal_primairy => restrict_to: [OWNER];
            admin_withdrawal_secondairy => restrict_to: [OWNER];
            admin_update_keystore => restrict_to: [OWNER];
            admin_collect_royalty => restrict_to: [OWNER];
            admin_set_royalty => restrict_to: [OWNER];
        }
    }
	
    struct PokerDiceBank {
        // my address
        myownaddress: Global<AnyComponent>,
        // vaults containing any kind of incomming coins
        xrdvault: Vault,
        // redemption vault contains any kind of outgoing coins
        tokenvaults: KeyValueStore<ResourceAddress, SwapTokenDetails>,
        // the swap ratio of the active coins
        playerledger: KeyValueStore<NonFungibleLocalId, Details>,
        // active incomming coin // set to None to disable swap
        activetoken: Option<ResourceAddress>,
        // resource manager for the nft
        depositor_nft_manager: ResourceManager,
        // keep track of the deposits also the NFT id.
        deposit_counter: u64,
        // adminvault for badge
        owner_vault: Vault,
        // Toggle on off swap function
        activeswap: bool,
        // disable this component
        pokerdiceactive: bool,
    }

    impl PokerDiceBank {
        pub fn instantiate() -> (Global<PokerDiceBank>, FungibleBucket) {
            let mut my_owner_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
		   	    .metadata (metadata! { 
					roles {
    					metadata_setter => OWNER;
						metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
					},	
					init {
						"name" => "PokerDiceBank Owner Badge", locked;
						"description" => "Owner Badge for PokerDiceBank Component", locked;
                        "symbol" => "PKDB", locked;
					}
                })
                .mint_initial_supply(3);

            // take one admin badge and put in in the admin vault
            let local_owner_badge: FungibleBucket = my_owner_badge.take(1);

            let owner_rule: AccessRule = rule!(require(my_owner_badge.resource_address()));
            
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(PokerDiceBank::blueprint_id()); 

            let depositor_nft_mng: ResourceManager = ResourceBuilder::new_integer_non_fungible::<Empty>(OwnerRole::None)
                .metadata (metadata! { 
					init {
						"name" => "PokerDice Deposit NFT", locked;
						"description" => "NFT to verify balance", locked;
                        "symbol" => "PDDNFT", locked;
					}
                })
                .mint_roles(mint_roles! (
                    minter => owner_rule.clone(); // rule!(require(global_caller(component_address)));
//                    minter => OWNER;
                    minter_updater => OWNER;
                ))
                .create_with_no_initial_supply();

             // Instantiate a CoinDispenser component, 
            let component: Globalizing<PokerDiceBank> = Self {
                myownaddress: component_address.into(),
                owner_vault: Vault::with_bucket(local_owner_badge.into()),
                xrdvault: Vault::new(XRD),
                activetoken: None,
//                activetokenratio: dec!(1),
                depositor_nft_manager: depositor_nft_mng,
                deposit_counter: 0,
                tokenvaults: KeyValueStore::new(),
                playerledger: KeyValueStore::new(),
                activeswap: false,
                pokerdiceactive: false,
            }
            .instantiate()
            .prepare_to_globalize(
                OwnerRole::Fixed(owner_rule.clone())
            )
            .with_address(address_reservation)
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_setter => OWNER;
                    royalty_setter_updater => OWNER;
                    royalty_locker => OWNER;
                    royalty_locker_updater => rule!(deny_all);
                    royalty_claimer => OWNER;
                    royalty_claimer_updater => rule!(deny_all);
                },
                init {
                    deposit_new => Xrd(1.into()), updatable;
                    deposit_add => Xrd(1.into()), updatable;
                    get_tokens => Free, locked;
                    redeem_tokens => Free, locked;
                    withdrawal => Free, locked;
                    admin_pokerdicebank_activate => Free, locked;
                    admin_token_swap_activate => Free, locked;
                    admin_global_swap_activate => Free, locked;
                    admin_swap_config => Free, locked;
                    admin_swap_reset => Free, locked;
                    admin_set_active_token => Free, locked;                   
                    admin_deposit_primairy => Free, locked;
                    admin_deposit_secondairy => Free, locked;
                    admin_withdrawal_primairy => Free, locked;
                    admin_withdrawal_secondairy => Free, locked;
                    admin_update_keystore => Free, locked;
                    admin_collect_royalty => Free, locked;
                    admin_set_royalty => Free, locked;
                }}
            );
            let globalized_component = component.globalize();
            (globalized_component, my_owner_badge)
        }

        // if playing with play tokens get 100 play tokens,
        // the 1 XRD is just a deposit, the play tokens can be redeemed later
        pub fn get_tokens(&mut self, mut deposit: Bucket) -> (Bucket, Bucket) {
            // check if the swap action is active
            assert!(self.activeswap,
                "The swap functionality has been disabled");
            assert!(self.activetoken.is_some(),
                "No swap token set");
            assert!(self.tokenvaults.get(&self.activetoken.unwrap()).is_some(),
                "No swap token vault");
            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                                self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            // check if the bucket holds at least 1 XRD
            assert!(!(deposit.amount()<dec!(1)),
                "not enough XRD for a swap");
            assert!((mytokendetail.tokenswapratio < mytokendetail.tokenvault.amount()),
                "Not enough coin in the token vault to complete the transaction");
            assert!(mytokendetail.activeswap,
                "Swap not activated for the active token");
            // take one xrd, and return the swapratio of the play token
            self.xrdvault.put(deposit.take(dec!(1)));
            let amount = mytokendetail.tokenswapratio;
            let tokentbucket = mytokendetail.tokenvault.take(amount);
            // return the two buckets
            (tokentbucket, deposit)
        }

        // if playing with play tokens redeem all play tokens,
        // the play tokens can be redeemed for 0.01 XRD per token
        pub fn redeem_tokens(&mut self, deposit: Bucket) -> Bucket {
            // check if the swap action is active
            // todo: need to be able to swap deactivated play tokens!!
//            assert!(self.activeswap,
//                "The swap functionality has been disabled");
//            assert!(self.activetoken.is_some(),
//                "No swap token set");
//            assert!(self.activetoken.unwrap()==deposit.resource_address(),
//                "Wrong token offered to swap");
            assert!(self.tokenvaults.get(&deposit.resource_address()).is_some(),
                "No token information found for supplied token");
            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                        self.tokenvaults.get_mut(&deposit.resource_address()).unwrap();
            assert!(mytokendetail.activeswap,
                "Swap functionality of this coin has been disabled");
            let amount = deposit.amount() / mytokendetail.tokenswapratio;
            assert!(amount < self.xrdvault.amount(),
                "Not enough XRD in the token vault to complete the transaction");
            // take all tokens
            mytokendetail.tokenvault.put(deposit);
            // return the XRD
            self.xrdvault.take(amount)
        }

        // depost active play tokens this function also takes royalty!
        // return a NFT for backend synchro
        pub fn deposit_new(&mut self, deposit: Bucket) -> Bucket {
            // check if total app is active
            assert!(self.pokerdiceactive,
                "The pokerdice dapp has been disabled");
            assert!(self.activetoken.is_some(),
                "No play token set");
            assert!(self.activetoken.unwrap()==deposit.resource_address(),
                "Wrong token offered to deposit");
            assert!(self.tokenvaults.get(&self.activetoken.unwrap()).is_some(),
                "No play token vault created yet");
            let mut mytokeninfo: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                                self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            let deposit_amount = deposit.amount();
            // take all tokens
            mytokeninfo.tokenvault.put(deposit);

            self.deposit_counter = self.deposit_counter + 1;
            let nft_id: NonFungibleLocalId = self.deposit_counter.into();
            let deposit_bucket: Bucket = self.owner_vault
                .as_fungible().authorize_with_amount(dec!(1), || {
                    self.depositor_nft_manager.mint_non_fungible(
                    &nft_id.clone(),
                    Empty{}
                )
            });
            let deposit_details = Details{
                balance: deposit_amount,
                locked: true,
                activetoken: self.activetoken,
            };
            self.playerledger.insert(nft_id, deposit_details);
            let myroyalty = self.owner_vault.as_fungible().authorize_with_amount(dec!(1), || {
                self.myownaddress.claim_component_royalties()
            });
            self.xrdvault.put(myroyalty);

            deposit_bucket
        }

        // depost active play tokens this function also takes royalty!
        // reuse the NFT for backend synchro
        pub fn deposit_add(&mut self, deposit: Bucket, badge: Proof)  {
            // check if total app is active
            assert!(self.pokerdiceactive,
                "The pokerdice dapp has been disabled");
            assert!(self.activetoken.is_some(),
                "No play token set");
            assert!(self.activetoken.unwrap()==deposit.resource_address(),
                "Wrong token offered to deposit");
            assert!(self.tokenvaults.get(&self.activetoken.unwrap()).is_some(),
                "No play token vault created yet");
            let mut mytokeninfo: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                                self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            let deposit_amount = deposit.amount();
            // take all tokens
            mytokeninfo.tokenvault.put(deposit);
            assert!(badge.resource_address() == self.depositor_nft_manager.address(),
                "Incorrect NFT supplied");
            let my_badge =
                badge.check(self.depositor_nft_manager.address());
            let non_fungible: NonFungible<Empty> = my_badge
                .as_non_fungible()
                .non_fungible::<Empty>();

            let nft_id = non_fungible.local_id().clone();
            // update keystorevalue here.
            let temp_keystore_ref = self.playerledger.get_mut(&nft_id);
            let mut deposit_details = temp_keystore_ref.unwrap();
            assert!(deposit_details.activetoken.is_none() || 
                            deposit_details.activetoken.unwrap()==self.activetoken.unwrap(),
                "Old playtoken detected, please unlock and withdrawal this first");
            deposit_details.balance += deposit_amount;
            deposit_details.locked = true;
            deposit_details.activetoken=self.activetoken;

            let myroyalty = self.owner_vault.as_fungible().authorize_with_amount(dec!(1), || {
                self.myownaddress.claim_component_royalties()
            });
            self.xrdvault.put(myroyalty);
 
         }
        
        // this is an admin function to withdrawal the primairy vaults
        pub fn admin_withdrawal_primairy(&mut self) -> Bucket{
            self.xrdvault.take_all()
        }

        // this is an admin function to withdrawal a specific resource from the secondairy vaults
        pub fn admin_withdrawal_secondairy(&mut self, targetaddress: ResourceAddress) -> Bucket{
            assert!(!self.tokenvaults.get(&targetaddress).is_none(),
                "The specified coin does not exist in this component");
		    let mut mytokeninfo: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                self.tokenvaults.get_mut(&targetaddress).unwrap();
            mytokeninfo.activeswap = false;
            mytokeninfo.tokenvault.take_all()
        }

        // Deposit XRD in the main vault.
        pub fn admin_deposit_primairy(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            assert!(resource_address == self.xrdvault.resource_address(),
                "Only XRD allowed for this deposit");
            self.xrdvault.put(bucket);
        }

        // Deposit any resource in the tokenvaults.
        pub fn admin_deposit_secondairy(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            if self.tokenvaults.get(&resource_address).is_none() {
                let v: Vault = Vault::with_bucket(bucket);
                let newtokeninfo = SwapTokenDetails{
                    tokenvault: v,
                    tokenswapratio: dec!(1),
                    activeswap: false,
                };
                self.tokenvaults.insert(resource_address, newtokeninfo);
            } else { 
                let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                                self.tokenvaults.get_mut(&resource_address).unwrap();
                mytokendetail.tokenvault.put(bucket);
            }
        }

        // admin function to define the playtokens/XRD combination and ratio
        // The ratio is a amount play tokens vs 1 XRD. 
        pub fn admin_swap_config(&mut self, playtoken: ResourceAddress, ratio: Decimal){
            assert!(self.tokenvaults.get(&playtoken).is_some(),
                   "This token vault does not exits, please deposit first");
            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                    self.tokenvaults.get_mut(&playtoken).unwrap();
            mytokendetail.tokenswapratio = ratio;
        }

        // set the active token
        pub fn admin_set_active_token(&mut self, playtoken: ResourceAddress){
            assert!(self.tokenvaults.get(&playtoken).is_some(),
                   "This token vault does not exits, please deposit first");
            self.activetoken = Some(playtoken);
        }

        // admin function to reset the swap function
        pub fn admin_swap_reset(&mut self){
            assert!(self.activetoken.is_some(),
                "Swap is already reset");
            assert!(!self.tokenvaults.get(&self.activetoken.unwrap()).is_some(),
                "This token vault does not exits, please deposit first");
            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                    self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            mytokendetail.activeswap=false;
            self.activetoken = None;
            self.activeswap = false;
        }

        // admin function to (de)activate the swap function
        pub fn admin_global_swap_activate(&mut self, activate: bool){
            assert!(self.activetoken.is_some(),
                "Swap token is not set, config swap first");
//            assert!(!self.tokenvaults.get(&self.activetoken.unwrap()).is_some(),
//                "This token vault does not exits, please deposit first");
//            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
//                self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            self.activeswap = activate;
//            mytokendetail.activeswap = activate
        }

        // admin function to (de)activate the swap function
        pub fn admin_token_swap_activate(&mut self, playtoken: ResourceAddress, activate: bool){
            assert!(self.tokenvaults.get(&playtoken).is_some(),
                "This token vault does not exits, please deposit first");
            assert!(self.activetoken.is_some(),
                "Swap token is not set, config swap first");
            let mut mytokendetail: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                self.tokenvaults.get_mut(&self.activetoken.unwrap()).unwrap();
            mytokendetail.activeswap = activate
        }
        
        // admin function to (de)activate the pokerdicebank component
        pub fn admin_pokerdicebank_activate(&mut self, activate: bool){
            self.pokerdiceactive = activate;
        }

        // public function to withdrawal the play-tokens.
        pub fn withdrawal(&mut self, badge: Proof) -> Bucket {
            assert!(badge.resource_address() == self.depositor_nft_manager.address(),
                "Incorrect NFT supplied");
            let my_badge =
                badge.check(self.depositor_nft_manager.address());
            let non_fungible: NonFungible<Empty> = my_badge
                .as_non_fungible()
                .non_fungible::<Empty>();
            let nft_id = non_fungible.local_id().clone();
            let temp_keystore_ref = self.playerledger.get_mut(&nft_id);
            let mut deposit_details = temp_keystore_ref.unwrap();
            assert!(!deposit_details.locked,
                "Your coin is still locked, need to unlock first");
            assert!(deposit_details.activetoken.is_some(),
                "No active token defined in this NFT");
//            assert!(self.activetoken.is_some(),
//                "No active token is set");
            assert!(self.tokenvaults.get(&deposit_details.activetoken.unwrap()).is_some(),
                "No vault for active token found");
            let total: Decimal = deposit_details.balance;
            let mut mytokeninfo: KeyValueEntryRefMut<'_, SwapTokenDetails> = 
                            self.tokenvaults.get_mut(&deposit_details.activetoken.unwrap()).unwrap();
            assert!((total < mytokeninfo.tokenvault.amount()),
                "Not enough coin in the return vault to complete the transaction");
            deposit_details.balance = dec!(0);
            deposit_details.locked = false;
            deposit_details.activetoken = None;
            mytokeninfo.tokenvault.take(total)
        }
    
        // function to unlock and update the keyvaluestore userbalance for withdrawal.
        pub fn admin_update_keystore(&mut self, nft_id: NonFungibleLocalId, balance: Decimal){
            // this function is only called by the backend
            // to keep this wallet going, the fee is paid by the component, out of the collected royalties
            assert!(self.xrdvault.as_fungible().amount()>dec!(5),
                "Not enough funds to prepay fee");
            self.xrdvault.as_fungible().lock_contingent_fee(dec!(5));
            let temp_keystore_ref = self.playerledger.get_mut(&nft_id);
            assert!(temp_keystore_ref.is_some(),
                "No match found for the NFT ID provided");
            let mut deposit_details = temp_keystore_ref.unwrap();
            deposit_details.balance = balance;
            deposit_details.locked = false;
        }

        // Collect the royalty
        pub fn admin_collect_royalty(&self) -> Bucket {
            self.owner_vault.as_fungible().authorize_with_amount(dec!(1), || {
                self.myownaddress.claim_component_royalties()
            })
        }
    
        // A royalty setter
        pub fn admin_set_royalty(&self, amount: Decimal){
            self.owner_vault.as_fungible().authorize_with_amount(dec!(1), || {
                self.myownaddress.set_royalty("deposit_new", Xrd(amount));
                self.myownaddress.set_royalty("deposit_add", Xrd(amount));
            });
        }
    
    }
}
