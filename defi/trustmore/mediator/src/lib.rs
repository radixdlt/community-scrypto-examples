use scrypto::prelude::*;

// Define the functions on the Mediator blueprint

#[blueprint]
mod mod_mediator {
    struct Mediator {
        // Define what resources and data will be managed by Mediator components
        radix_vault: Vault,
        // referece to all the contracts instantiated from this component.
        contracts: KeyValueStore<ComponentAddress, Vault>,
        // Buy cost of a contract
        price_of_contract: Decimal,
        
        // Contract package address
        contract_package_address: PackageAddress,

        // my own component address
        component_address: Option<ComponentAddress>,

        // Contract instantiation token
        contract_instatiation_token: KeyValueStore<PackageAddress, Vault>,

        // admin vault, this contains a badge for various inner dApp permission handling
        admin_vault: Vault,


    }

    impl Mediator {
        /*
            Dummy function, placeholder for the royalty setting, no further use.
         */
        pub fn dummy(){}

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate() -> (ComponentAddress, Bucket) {
            /*
                This royalty configuration of the dummy function has a byproduct 
                that calling the instantiation requires proof of owner-badge.
                Since the owner-badge will be send to this component to be used for the 
                same owner-badge-proof setup of the Mediator accredited contract
                we are guaranteed only one component can be made from this blueprint.
             */
            borrow_package!(Runtime::package_address()).set_royalty_config(BTreeMap::from([(
                "Mediator".to_owned(),
                RoyaltyConfigBuilder::new()
                    .add_rule("dummy", 0u32)
                    .default(0),
            )]));

            let mut my_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Mediator Admin Badge")
                .metadata("description", "Badge for Mediator Subscription Component")
                .mint_initial_supply(2);

            // put one admin badge and put in in the admin vault
            let local_admin_badge: Bucket = my_admin_badge.take(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));

            let access_rules: AccessRulesConfig = AccessRulesConfig::new()
                .method("withdrawal",admin_rule.clone(), rule!(deny_all))
                .method("set_contract_package_info",admin_rule.clone(), rule!(deny_all))
                .method("return_owner_badge",admin_rule.clone(), rule!(deny_all))
                .default(rule!(allow_all), LOCKED);

//            let access_rules: AccessRules = AccessRules::new()
//                .method("withdrawal",admin_rule.clone(), rule!(deny_all))
//                .method("set_contract_package_info",admin_rule.clone(), rule!(deny_all))
//                .method("return_owner_badge",admin_rule.clone(), rule!(deny_all))
//                .default(rule!(allow_all), LOCKED);
 
            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let component: MediatorComponent = Self {
                radix_vault: Vault::new(RADIX_TOKEN),
                admin_vault: Vault::with_bucket(local_admin_badge),
                contracts: KeyValueStore::new(),
                price_of_contract: dec!("9.99"),
                component_address: None,
                contract_package_address: Runtime::package_address(),
                contract_instatiation_token: KeyValueStore::new(),
            }
            .instantiate();
//            component.add_access_check(access_rules);
        
            // Setting up component address of this component
            let globalized_component: ComponentAddress = component.globalize_with_access_rules(access_rules);
            let global_ref: MediatorGlobalComponentRef = globalized_component.into();
            global_ref.set_component_address(globalized_component); 

            // Return the instantiated component and the admin badge that was just minted
            (globalized_component, my_admin_badge)

        }


        pub fn set_component_address(&mut self, address: ComponentAddress) {
            match self.component_address {
                None => self.component_address = Some(address),
                Some(_) => panic!("This method can only be called once")
            }
        }

        /*
            Get all the money out of the radix vault
         */
        pub fn withdrawal(&mut self) -> Bucket {
            self.radix_vault.take_all()
        }

        pub fn set_contract_package_info(&mut self, contractaddr: PackageAddress, 
                                          contractbadge: Bucket) {
            self.contract_package_address = contractaddr.clone();
            
            if self.contract_instatiation_token.get(&contractaddr).is_none() {
                let v: Vault = Vault::with_bucket(contractbadge);
                self.contract_instatiation_token.insert(contractaddr, v);
            } else {
                if self.contract_instatiation_token.get(&contractaddr).unwrap().amount() == dec!("0"){
                    self.contract_instatiation_token.get_mut(&contractaddr).unwrap().put(contractbadge);
                }else{
                    panic!("There should only be one owner badge, and we alreay have it");
                }
            }    
        }

        /*
            New contract instantiation, can only done with this call
         */
        pub fn get_new_contract(&mut self, mut payment: Bucket) -> 
            (ComponentAddress, Bucket, Bucket, Bucket) {

            // check if the payment bucket is XRD type, and hold enough coin
            assert!(
                payment.resource_address() == self.radix_vault.resource_address(),
                "The contract can only be bought Radix tokens"
            );
            assert!(!(payment.amount()<self.price_of_contract), 
                "There are not enough tokens in your account");
        
            let xrd_buy_in: Bucket = payment.take(self.price_of_contract);
            self.radix_vault.put(xrd_buy_in);
 
            let my_packet: BorrowedPackage = borrow_package!(self.contract_package_address);

            let v:DataRef<Vault> = self.contract_instatiation_token.get(&self.contract_package_address).unwrap();

            let my_component_addres: ComponentAddress = self.component_address.unwrap();

            let (contract_component, contractbadge, 
                buyer_token, seller_token) 
                = v.authorize(|| my_packet.call::
                        <(ComponentAddress, Bucket, Bucket, Bucket)>
                        ("Contract", "new", scrypto_args![my_component_addres]));

            if self.contracts.get(&contract_component).is_none() {
                let v: Vault = Vault::with_bucket(contractbadge);
                self.contracts.insert(contract_component, v);
            } else {
                panic!("There should only be one owner badge, and we alreay have it");
            }    

            (contract_component, seller_token, buyer_token, payment)
        }

        pub fn return_owner_badge(&mut self)-> Bucket {
            
            let mut v: DataRefMut<Vault> = 
                self.contract_instatiation_token.get_mut(&self.contract_package_address).unwrap();
            
            v.take_all()

        }

    }
}