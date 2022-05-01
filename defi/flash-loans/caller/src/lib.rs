use scrypto::prelude::*;

/*
 *  This is an example of a component that is requesting a
 *  flash loan from another component to make more XRD and repay the loan
 */

blueprint! {
    struct Caller {
        // Address of the flashloan component
        loaner_component: ComponentAddress,
        // Used to store payment for the opportunity
        hidden_vault: Vault,
        // Used to represent tokens that can be accessed if the user gets the loan
        opportunity: Vault
    }

    impl Caller {
        pub fn new(loaner_component_address: ComponentAddress, opportunity: Bucket) -> ComponentAddress {
            assert!(opportunity.amount() > dec!("1000"), "The amount of the opportunity must be bigger than 1000");
            assert_eq!(opportunity.resource_address(), RADIX_TOKEN, "The tokens for the opportunity must be XRD");

            Self {
                loaner_component: loaner_component_address,
                hidden_vault: Vault::new(RADIX_TOKEN),
                opportunity: Vault::with_bucket(opportunity)
            }
            .instantiate()
            .globalize()
        }

        pub fn call(&self, this_component_address: ComponentAddress) -> Bucket {
            // Get a loan of 1000 XRD
            let amount:Decimal = dec!("1000");

            // Call request loan and return the change back to the user
            borrow_component!(self.loaner_component).call::<Bucket>("request_loan", args![
                amount, 
                this_component_address
            ]).into()
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
