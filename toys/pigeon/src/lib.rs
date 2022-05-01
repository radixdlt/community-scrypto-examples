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
        pub fn new() -> (ComponentAddress, Vec<Bucket>) {
            let head: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Pigeon Head")
                .initial_supply(1);

            let body: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Pigeon Body")
                .initial_supply(1);
                
            let wings: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Pigeon Wing")
                .initial_supply(2);

            let legs: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Pigeon Leg")
                .initial_supply(2);

            let component = Self {
                head_vault: Vault::new(head.resource_address()),
                body_vault: Vault::new(body.resource_address()),
                leg_vault: Vault::new(legs.resource_address()),
                wing_vault: Vault::new(wings.resource_address())
            }
            .instantiate();

            info!("Oh no! The pigeon got scattered all accross the shard space ! Help it !");

            (component.globalize(), vec![head, body, wings, legs])
        }

        // Add a part to the pigeon
        pub fn add_part(&mut self, part: Bucket) {
            if part.resource_address() == self.head_vault.resource_address() {
                self.head_vault.put(part);
            } else if part.resource_address() == self.body_vault.resource_address() {
                self.body_vault.put(part);
            } else if part.resource_address() == self.leg_vault.resource_address() {
                self.leg_vault.put(part);
            } else if part.resource_address() == self.wing_vault.resource_address() {
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

            info!("The pigeon is flying successfully !")
        }
    }
}
