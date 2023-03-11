use scrypto::prelude::*;

#[derive(NonFungibleData, Clone)]
pub struct Info4Contract {
    contract_adress: ComponentAddress,
    mediator_address: ComponentAddress,
    #[mutable]
    validated: bool,
}

#[derive(NonFungibleData)]
pub struct Info4Mediator {
    contract_adress: ComponentAddress,
}


#[blueprint]
mod mod_contract {
    struct Contract {

        admin_vault: Vault,
        my_buyer_badge: ResourceAddress,     
        my_seller_badge: ResourceAddress,
        my_mediator_badge: ResourceAddress,
        calling_component: ComponentAddress,
        component_address: Option<ComponentAddress>,    

    }

    impl Contract { 
        /*
            Dummy function, placeholder for the royalty setting, no further use.
         */
        pub fn dummy(){}

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new(my_caller: ComponentAddress) -> (ComponentAddress, Bucket, Bucket, Bucket) {
            /*
                This royalty configuration of the dummy function has a byproduct 
                that calling the instantiation requires proof of owner-badge.
                Since the owner-badge will be send to this component to be used for the 
                same owner-badge-proof setup of the Mediator accredited contract
                we are guaranteed only one component can be made from this blueprint.
             */
            borrow_package!(Runtime::package_address()).set_royalty_config(BTreeMap::from([(
                "Contract".to_owned(),
                RoyaltyConfigBuilder::new()
                    .add_rule("dummy", 0u32)
                    .default(0),
            )]));

            let my_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Internal Admin")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let admin_rule: AccessRule = rule!(require(my_admin_badge.resource_address()));
                
            let my_mediator_badge: ResourceAddress = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Contract Mediator Badge")
                .metadata("description", "Mediator Badge for Validated Contract Component")
                .mintable(admin_rule.clone(), rule!(deny_all))
                .updateable_non_fungible_data(admin_rule.clone(), rule!(deny_all))
                .create_with_no_initial_supply();

            let my_buyer_badge: ResourceAddress = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Contract Buyer Badge")
                .metadata("description", "Buyer Badge for Validated Contract Component")
                .mintable(admin_rule.clone(), rule!(deny_all))
                .updateable_non_fungible_data(admin_rule.clone(), rule!(deny_all))
                .create_with_no_initial_supply();

            let my_seller_badge: ResourceAddress = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Contract Seller Badge")
                .metadata("description", "Seller Badge for Validated Contract Component")
                .mintable(admin_rule.clone(), rule!(deny_all))
                .updateable_non_fungible_data(admin_rule.clone(), rule!(deny_all))
                .create_with_no_initial_supply();

            let _mediator_rule: AccessRule = rule!(require(my_mediator_badge));
            let _buyer_rule: AccessRule = rule!(require(my_buyer_badge));
            let _seller_rule: AccessRule = rule!(require(my_seller_badge));


            let access_rules: AccessRules = AccessRules::new()
                .method("mint_and_init", rule!(allow_all), rule!(deny_all))
//                .method("activate_seller_badge", seller_rule.clone(), rule!(deny_all))
//                .method("activate_buyer_badge", buyer_rule.clone(), rule!(deny_all))
//                .method("set_mediator_address", client_rule.clone(), rule!(deny_all))
                .default(rule!(allow_all), LOCKED);

            let mut component:ContractComponent = Self {
                admin_vault: Vault::with_bucket(my_admin_badge),
                my_buyer_badge,    
                my_seller_badge,
                my_mediator_badge,     
                calling_component: my_caller,
                component_address: None,  
            }
            .instantiate();
            component.add_access_check(access_rules);

            // Setting up component address of this component
            let globalized_component = component.globalize();
            let global_ref: ContractGlobalComponentRef = globalized_component.into();
            let(seller_bucket, buyer_bucket, mediator_bucket) = 
                global_ref.mint_and_init(globalized_component);

            (globalized_component, mediator_bucket, seller_bucket, buyer_bucket)

        }

        pub fn mint_and_init(&mut self, my_component_addres: ComponentAddress) -> (Bucket, Bucket, Bucket){
            
            match self.component_address {
                None => self.component_address = Some(my_component_addres),
                Some(_) => panic!()
            }

            let nft_data = Info4Contract {
                contract_adress: my_component_addres,
                mediator_address: self.calling_component,
                validated: false,
            };

            let seller_bucket = self.admin_vault.authorize( || {
                borrow_resource_manager!(self.my_seller_badge).mint_uuid_non_fungible(
                nft_data.clone()
                )
            });

            let buyer_bucket = self.admin_vault.authorize( || {
                borrow_resource_manager!(self.my_buyer_badge).mint_uuid_non_fungible(
                nft_data.clone()
                )
            });

            let mediator_data = Info4Mediator {
                contract_adress: my_component_addres,
            };

            let mediator_bucket = self.admin_vault.authorize( || {
                borrow_resource_manager!(self.my_mediator_badge).mint_uuid_non_fungible(
                mediator_data
                )
            });

            (seller_bucket, buyer_bucket, mediator_bucket)
        }

        /*
            Method to activate the seller badge
            If a seller received an already activate seller badge,
            the contract should not be accepted.
         */
        pub fn activate_seller_badge(&mut self, my_badge: Proof) {
            
            let validated_proof = my_badge.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.my_seller_badge)
            ).expect("invalid proof");

            let nft_id = validated_proof.non_fungible_local_id();
        
            let resource_manager: &mut ResourceManager = 
                borrow_resource_manager!(self.my_seller_badge);
        
            let mut local_data: Info4Contract = resource_manager.get_non_fungible_data(&nft_id);

            local_data.validated = true;

            self.admin_vault.authorize(|| resource_manager.update_non_fungible_data(
                &nft_id, 
                local_data
            ));

        }

        /*
            Method to activate the buyer badge
            If a buyer received an already activate buyer badge,
            the contract should not be accepted.
         */

        pub fn activate_buyer_badge(&mut self, my_badge: Proof) {
            
            let validated_proof = my_badge.validate_proof(
                ProofValidationMode::ValidateResourceAddress(self.my_buyer_badge)
            ).expect("invalid proof");

            let nft_id = validated_proof.non_fungible_local_id();
        
            let resource_manager: &mut ResourceManager = 
                borrow_resource_manager!(self.my_buyer_badge);
        
            let mut local_data: Info4Contract = resource_manager.get_non_fungible_data(&nft_id);

            local_data.validated = true;

            self.admin_vault.authorize(|| resource_manager.update_non_fungible_data(
                &nft_id, 
                local_data
            ));

        }
/*
        A lot more contractual methods and functions with access for buyer/seller
        and mediatior proof can be made.
 */

    }
}