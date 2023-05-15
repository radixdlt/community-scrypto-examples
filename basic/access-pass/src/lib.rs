use scrypto::prelude::*;

#[blueprint]
mod access_pass {
    struct AccessPassManager {
        pass: Vault,
        comp_addr: Option<ComponentAddress>,
    }

    impl AccessPassManager {
        pub fn new( // Args
            rule: AccessRule, // The Access Control on the `get_pass` method.
            locked: bool, // Specify if rule mutability is to be locked when the rule is applied.
            pass_royalty: u32, // Royalty to be paid for utilizing this pass.
            dapp_def: Option<Address>, // Optional dApp definition for this Access Pass Manager.
            name: Option<String>, // Optional name for access pass.
            description: Option<String>, // Optional description for access pass.
        ) -> ( // Returns
            ComponentAddress, // Address of new Access Pass Manager.
            ResourceAddress, // Address of new Access Pass Resource.
            Option<Bucket> // Optional owner badge for mutating the `get_pass` rule.
        ) {
            // Create Access Pass
            let pass_builder = ResourceBuilder::new_fungible() // Create fungible non-divisible.
                .divisibility(DIVISIBILITY_NONE);
            // Set name metadata
            let pass_builder = match name.clone() {
                None => pass_builder.metadata("name", "Access Pass"),
                Some(name) => pass_builder.metadata("name", format!("Access Pass: {}", name))
            };
            // Set description
            let pass_builder = match description.clone() {
                None => pass_builder.metadata("description", "The holder of a Proof of this Pass can access protected actions."),
                Some(description) => pass_builder.metadata("description", description)
            };
            // Finish building resource and mint token
            let pass: Bucket = pass_builder.restrict_withdraw(AccessRule::DenyAll, LOCKED) // The token never leaves the vault.
                .mint_initial_supply(1);

            let pass_address = pass.resource_address();
            
            // Create component Owner Badge
            let owner_badge: Option<Bucket>;
            if !locked {
                owner_badge = Some(ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata("name", "Access Pass Manager Owner Badge")
                    .metadata("description", "Owner Badge for an Access Pass Manager that has priviledges to 
                    mutate the `get_pass` method rule.")
                    .burnable(AccessRule::AllowAll, LOCKED)
                    .mint_initial_supply(1));
            } else {
                owner_badge = None;
            }

            // Access Control
            // Create rules config
            let access_rules_config = match locked {
                // set AccessRules for `get_pass`
                true => AccessRulesConfig::new().method("get_pass", rule, LOCKED),
                false => AccessRulesConfig::new().method("get_pass", rule, rule!(require(owner_badge.as_ref().unwrap().resource_address()))),
            }.default(AccessRule::AllowAll, LOCKED); // set default rule, return config.
            // Create access rules module
            let access_rules = AccessRules::new(access_rules_config);

            // Create Metadata module
            let metadata = Metadata::new();

            if dapp_def.is_some() {
                metadata.set("dapp_definition", dapp_def.unwrap());
            }
            if name.is_some() {
                metadata.set("name", format!("Access Pass Manager: {}", name.unwrap()));
            }
            if description.is_some() {
                metadata.set("description", description.unwrap());
            }

            // Create Royalty config
            let royalty_config = RoyaltyConfigBuilder::new()
                .add_rule("get_pass", pass_royalty)
                .default(0);
            // Create Royalty module
            // This cannot be changed after instantiation to reduce uncertainty for those building on top of this component.
            let royalty = Royalty::new(royalty_config);

            let component = Self {
                pass: Vault::with_bucket(pass),
                comp_addr: None
            }.instantiate().globalize_with_modules(access_rules, metadata, royalty);

            // Store component address in component state
            let global_ref: AccessPassManagerGlobalComponentRef = component.into();
            global_ref.set_component_address(component);

            (component, pass_address, owner_badge)
        }



        // Emit a pass proof which can be used for other dapps that utilize this pass.
        pub fn get_pass(&self) -> Proof {
            self.pass.create_proof()
        } // ACCESS CONTROL: custom rule set by Access Pass Manager.


        // A new pass rule can be set using an external method or with this direct method call to the manager
        pub fn set_pass_rule(&mut self, rule: AccessRule, owner_badge: Bucket) -> Bucket {
            let global_ref: AccessPassManagerGlobalComponentRef = self.comp_addr.unwrap().into();
            let mut access_rules = global_ref.access_rules();
            // Apply new rule
            owner_badge.authorize(|| {access_rules.set_method_auth("get_pass", rule)});
            owner_badge
        }


        // The address of the pass resource can be retrieved from the manager
        pub fn get_pass_rsrc(&self) -> ResourceAddress {
            self.pass.resource_address()
        }



        pub fn set_component_address(&mut self, address: ComponentAddress) {
            match self.comp_addr {
                None => self.comp_addr = Some(address),
                Some(_) => panic!("component address already set.")
            }
        }
    }
}
