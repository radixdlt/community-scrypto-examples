use scrypto::prelude::*;

blueprint! {
    struct Pigeon {
        head_vault: Vault,
        body_vault: Vault,
        leg_vault: Vault,
        wing_vault: Vault
    }

    impl Pigeon {
        // Create the different pigeon parts and the component
        pub fn new() -> (Component, Vec<Bucket>) {
            let head: Bucket = ResourceBuilder::new()
                .metadata("name", "Pigeon Head")
                .new_token_fixed(1);

            let body: Bucket = ResourceBuilder::new()
                .metadata("name", "Pigeon Body")
                .new_token_fixed(1);
                
            let wings: Bucket = ResourceBuilder::new()
                .metadata("name", "Pigeon Wing")
                .new_token_fixed(2);

            let legs: Bucket = ResourceBuilder::new()
                .metadata("name", "Pigeon Leg")
                .new_token_fixed(2);

            let component = Self {
                head_vault: Vault::new(head.resource_def()),
                body_vault: Vault::new(body.resource_def()),
                leg_vault: Vault::new(legs.resource_def()),
                wing_vault: Vault::new(wings.resource_def())
            }
            .instantiate();

            (component, vec![head, body, wings, legs])
        }

        // Add a part to the pigeon
        pub fn add_part(&mut self, part: Bucket) {
            if part.resource_def() == self.head_vault.resource_def() {
                self.head_vault.put(part);
            } else if part.resource_def() == self.body_vault.resource_def() {
                self.body_vault.put(part);
            } else if part.resource_def() == self.leg_vault.resource_def() {
                self.leg_vault.put(part);
            } else if part.resource_def() == self.wing_vault.resource_def() {
                self.wing_vault.put(part);
            } else {
                info!("Cannot fit the part !");
            }        
        }

        // Try to fly. The pigeon must be fully assembled !
        pub fn fly(&self) {
            assert!(self.head_vault.amount() == 1.into(), "Missing head !");
            assert!(self.body_vault.amount() == 1.into(), "Missing body !");
            assert!(self.leg_vault.amount() == 2.into(), "Missing legs !");
            assert!(self.wing_vault.amount() == 2.into(), "Missing wings !");

            info!("The pigeon is flying sucessfuly !")
        }
    }
}
