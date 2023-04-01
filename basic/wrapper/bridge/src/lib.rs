use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
// this struck could hold timestamps/epoch inforamtion 
// or other tracking to make the subscription expire.
pub struct Subscription {
    #[mutable]
    issued_unix_epoch: i64,
    #[mutable]
    expire_unix_epoch: i64,
}

#[blueprint]
mod mod_bridge {
    struct Bridge {
        // vault to store radix, buy-in go here.
        radix_vault: Vault,

        // vault to store badge for the service we are bridging to.
        service_vault: Vault,
        
        // resourceaddress of the NFT ticket, used to subscribe to this bridge
        subscription_ticket: ResourceAddress,

        // admin vault, this contains a badge for various inner dApp permission handling
        admin_vault: Vault,

        // keep track of the number of subscriptions issued
        subscriptions_issued: u64,

        // address of the component we are bridging to 
        awesome_service: ComponentAddress,

    }

    impl Bridge {
        // Implement the functions and methods which will manage those resources and data
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate(extcomponent: ComponentAddress, 
                           extbadge: ResourceAddress) -> (ComponentAddress, Bucket) {

            // creating our admin badges
            // use one badge for internal admin stuff, and send one to instantiate wallet address.
            let mut my_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin Badge for Bridge")
                .mint_initial_supply(2);

            // put one admin badge and put in in the admin vault
            let local_admin_badge: Bucket = my_admin_badge.take(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));

            // Create our Ticket NFT
            let subscription_ticket: ResourceAddress = ResourceBuilder::new_integer_non_fungible::<Subscription>()
                .metadata("name", "Ticket for Bridge Subscription")
                .burnable(admin_rule.clone(), LOCKED)
                .mintable(rule!(require(my_admin_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(admin_rule.clone(), LOCKED)
                .restrict_withdraw(rule!(allow_all), LOCKED)
                .restrict_deposit(AccessRule::AllowAll, LOCKED)
                .create_with_no_initial_supply();

            let _subscription_rule: AccessRule = rule!(require(subscription_ticket));

            // set the access rules for the Admin-only and internal functions.
            let access_rules: AccessRulesConfig = AccessRulesConfig::new()
                .method("retreive_badge", admin_rule.clone(), AccessRule::DenyAll)
                .method("withdrawal_all", admin_rule.clone(), rule!(deny_all))
                .method("admin_subscription_ticket", admin_rule.clone(), rule!(deny_all))
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            let component: BridgeComponent = Self {
                radix_vault: Vault::new(RADIX_TOKEN),
                service_vault: Vault::new(extbadge),
                subscription_ticket,
                admin_vault: Vault::with_bucket(local_admin_badge),
                subscriptions_issued : 0u64,
                awesome_service: extcomponent,
            }
          
            .instantiate();
            let component: ComponentAddress = component.globalize_with_access_rules(access_rules);

            // Return the instantiated component and the admin badge that was just minted
            (component, my_admin_badge)
        }

        /*
            simple bridge to the awesome service public method.
        */
        pub fn simple_bridge(&mut self, auth: Proof) -> i8 {

            let validated_proof: ValidatedProof = auth.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.subscription_ticket)
            ).expect("invalid proof");

            let auth_id: NonFungibleLocalId = validated_proof.non_fungible_local_id();
        
            let resource_manager: ResourceManager = 
                borrow_resource_manager!(self.subscription_ticket);
        
            let subscription_data: Subscription = resource_manager.get_non_fungible_data(&auth_id);

            let now: i64 = Clock::current_time(scrypto::blueprints::clock::TimePrecision::Minute).seconds_since_unix_epoch;
            
            assert!((subscription_data.expire_unix_epoch - now).is_positive(), 
                "Subscription has expired");

            let component: GlobalComponentRef = borrow_component!(self.awesome_service);
            let result: i8 = component.call::<i8>("deep_thought", scrypto_args![]);
            result
        }

        /*
            bridge to the awesome service requiring System Level Authentication.
        */
        pub fn restricted_bridge(&mut self,  auth: Proof) -> i8 {

            let validated_proof: ValidatedProof = auth.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.subscription_ticket)
            ).expect("invalid proof");

            let auth_id:NonFungibleLocalId = validated_proof.non_fungible_local_id();
        
            let resource_manager: ResourceManager = 
                borrow_resource_manager!(self.subscription_ticket);
        
            let subscription_data: Subscription = resource_manager.get_non_fungible_data(&auth_id);

            let now: i64 = Clock::current_time(scrypto::blueprints::clock::TimePrecision::Minute).seconds_since_unix_epoch;
            
            assert!((subscription_data.expire_unix_epoch - now).is_positive(), 
                "Subscription has expired");

            assert!(!self.service_vault.is_empty(), "Need the service badge!");
            
            let component: GlobalComponentRef = borrow_component!(self.awesome_service);
            
            let result: i8 = self.service_vault.authorize(
                ||component.call::<i8>("awesome_service", scrypto_args![]));

            result
        }

        /*
            bridge to the awesome service requiring Application Level Authentication.
        */
        pub fn second_bridge(&mut self,  auth: Proof) -> i8 {

            let validated_proof: ValidatedProof = auth.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.subscription_ticket)
            ).expect("invalid proof");

            let auth_id: NonFungibleLocalId = validated_proof.non_fungible_local_id();
        
            let resource_manager: ResourceManager = 
                borrow_resource_manager!(self.subscription_ticket);
        
            let subscription_data: Subscription = resource_manager.get_non_fungible_data(&auth_id);

            let now: i64 = Clock::current_time(scrypto::blueprints::clock::TimePrecision::Minute).seconds_since_unix_epoch;
            
            assert!((subscription_data.expire_unix_epoch - now).is_positive(), 
                "Subscription has expired");

            assert!(!self.service_vault.is_empty(), "Need the service badge!");
            
            let component: GlobalComponentRef = borrow_component!(self.awesome_service);
            let my_proof: Proof = self.service_vault.create_proof_by_amount(dec!("1"));
            let result: i8 = component.call::<i8>("second_service", scrypto_args![my_proof]);
            result
        }

        /*
            Deposit badge used a access-proof for the service we bridge to.
        */
        pub fn deposit_badge(&mut self, mut deposit: Bucket) -> Bucket {

            assert!(
                deposit.resource_address() == self.service_vault.resource_address(),
                "The The badge can only be for the service we bridge to"
            );

            self.service_vault.put(deposit.take(1));

            // return the bucket incase of a surplus of tokens
            deposit
        }

        /*
            Withdrawal all XRD coin from the main wallet 
            Admin only function.
        */
        pub fn withdrawal_all(&mut self) -> Bucket {
            let xrd_withdrawal: Bucket = self.radix_vault.take_all();
            xrd_withdrawal
        }

        /*
            retreive badge used a accessproof for the service we bridge to.
            requires Admin Access badge
        */
        pub fn retreive_badge(&mut self) -> Bucket {

            let service_badge: Bucket = self.service_vault.take_all();
            service_badge
        }

        /*
            Admin can retreive a free subscriptionticket 
            Admin only function
            function is also called by the buy_token function.
        */
        pub fn admin_subscription_ticket(&mut self) -> Bucket {
          
            self.subscriptions_issued = self.subscriptions_issued + 1;

            let now: i64 = Clock::current_time(scrypto::blueprints::clock::TimePrecision::Minute).seconds_since_unix_epoch;
            let expire: i64 = &now + 8674560i64; // add nr seconds for 31 days

            let service_data: Subscription = Subscription {
                issued_unix_epoch: now,
                expire_unix_epoch: expire, 
            };

            let subscription_bucket: Bucket = self.admin_vault.authorize(||{
                borrow_resource_manager!(self.subscription_ticket).mint_non_fungible(
                &NonFungibleLocalId::Integer(self.subscriptions_issued.into()),
                service_data
                )
            });

            subscription_bucket
        }

        /*
            Buy one RaDiceX ticket for 1 XRD, mint a NFT and send back
        */
        pub fn buy_subscription_ticket(&mut self, mut buyin: Bucket) -> (Bucket, Bucket) {

            // check if the buy-in bucket is XRD type, and hold enough coin
            assert!(
                buyin.resource_address() == self.radix_vault.resource_address(),
                "The Buy-in can only be done with Radix tokens"
            );
            assert!(buyin.amount()>dec!("1"), "There are not enough tokens in your account");
            
            let amount: Decimal = dec!("1");             
 
            let subscription_bucket: Bucket = self.admin_subscription_ticket();

            let xrd_buy_in: Bucket = buyin.take(amount);
            self.radix_vault.put(xrd_buy_in);
 
            (subscription_bucket, buyin)
        }

        /*
            Burning of a Subscription Ticket 
        */
        pub fn burn_subscription_ticket(&mut self, subscription_ticket: Bucket) {
            assert!(
                subscription_ticket.resource_address() == self.subscription_ticket,
                "The supplied bucket does not contain the correct Ticket address"
            );
            assert!(!subscription_ticket.is_empty(), "The supplied bucket is empty");
            assert!(subscription_ticket.amount()==dec!("1"), "Only one (1) ticket per call is supported");

            let resource_manager: ResourceManager = 
                borrow_resource_manager!(self.subscription_ticket);
    
            self.admin_vault.authorize(|| resource_manager.burn(subscription_ticket));
        }

    }
}
