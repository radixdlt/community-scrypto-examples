use scrypto::prelude::*;

/*
 *  This is an example of a component that is requesting a
 *  flash loan from another component to make more XRD and repay the loan
 */

blueprint! {
    struct Caller {
        // Address of the flashloan component
        loaner_component: Address,
        // Used to store payment for the opportunity
        hidden_vault: Vault,
        // Used to represent tokens that can be accessed if the user gets the loan
        opportunity: Vault
    }

    impl Caller {
        pub fn new(loaner_component_address: Address, opportunity: Bucket) -> Component {
            assert!(opportunity.amount() > 1000.into(), "The amount of the opportunity must be bigger than 1000");
            assert!(opportunity.resource_address() == RADIX_TOKEN, "The tokens for the opportunity must be XRD");

            Self {
                loaner_component: loaner_component_address,
                hidden_vault: Vault::new(RADIX_TOKEN),
                opportunity: Vault::with_bucket(opportunity)
            }
            .instantiate()
        }

        pub fn call(&self, this_component_address: Address) -> Bucket {
            // Get a loan of 1000 XRD
            let amount:Decimal = dec!("1000");
            let args = vec![
                scrypto_encode(&amount),
                scrypto_encode(&this_component_address)
            ];

            // Call request loan and return the change back to the user
            Component::from(self.loaner_component).call::<Bucket>("request_loan", args).into()
        }

        // Used to simulate an opportunity to make some XRD by paying 1000 XRD
        fn get_opportunity(&mut self, payment: Bucket) -> Bucket {
            assert!(payment.amount() >= 1000.into(), "You need to pay 1000 XRD !");
            self.hidden_vault.put(payment);

            self.opportunity.take_all()
        }

        pub fn execute(&mut self, loan: Bucket) -> Bucket {
            info!("Received {} from loaner !", loan.amount());
            info!("Found opportunity to make {} XRD !", self.opportunity.amount());

            let opportunity_result = self.get_opportunity(loan);

            // Repay the loan (you get the change back)
            opportunity_result
        }
    }
}
