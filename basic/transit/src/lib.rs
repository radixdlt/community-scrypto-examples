use scrypto::prelude::*;

blueprint! {
    struct Transit {
        /// Ticket Resource Definition and Mint/Burn Authorization
        ticket_resource_address: ResourceAddress,
        ticket_minter: Vault,
        /// Enable/Disable rides
        american_rides: bool,
        european_rides: bool,
        /// Ticket/Ride prices
        ticket_price: Decimal,
        ride_price: Decimal,
        /// Control rides/Withdraw payments
        american_host_badge: ResourceAddress,
        european_host_badge: ResourceAddress,
        /// Collected revenue from ticket sales
        collected_dollars: Vault,
        collected_euros: Vault
    }

    impl Transit {
        pub fn new(
            price_per_ticket: Decimal,
            price_per_ride: Decimal,
            dollar: ResourceAddress,
            euro: ResourceAddress
        ) -> (ComponentAddress, Bucket, Bucket) {

            let valid_cli_arguments = price_per_ticket > 0.into() && price_per_ride > 0.into();
            assert!(valid_cli_arguments, "Invalid CLI arguments");

            let american_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "American Host Badge")
                .metadata("symbol", "APB")
                .metadata("description", "A badge that grants american host privileges")
                .initial_supply(1);

            let european_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "European Host Badge")
                .metadata("symbol", "EPB")
                .metadata("description", "A badge that grants european host privileges")
                .initial_supply(1);

            let ticket_minter = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Ticket Mint Auth")
                .initial_supply(1);

            let ticket_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Ticket")
                .metadata("symbol", "TK")
                .metadata("description", "A ticket used for rides")
                .mintable(rule!(require(ticket_minter.resource_address())), LOCKED)
                .burnable(rule!(require(ticket_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let access_rules = AccessRules::new()
                .method("withdraw_dollars", rule!(require(american_badge.resource_address())))
                .method("withdraw_euros", rule!(require(european_badge.resource_address())))
                .default(rule!(allow_all));

            let component = Self {
                ticket_resource_address,
                ticket_minter: Vault::with_bucket(ticket_minter),
                american_rides: true,
                european_rides: true,
                ticket_price: price_per_ticket,
                ride_price: price_per_ride,
                american_host_badge: american_badge.resource_address(),
                european_host_badge: european_badge.resource_address(),
                collected_dollars: Vault::new(dollar),
                collected_euros: Vault::new(euro)
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (component, american_badge, european_badge)
        }

        /// american_host_badge can take_all the collected_dollars and control american_rides
        pub fn withdraw_dollars(&mut self, availability: bool) -> Bucket {
            self.american_rides = availability;
            self.collected_dollars.take_all()
        }

        /// european_host_badge can take_all the collected_euros and control european_rides
        pub fn withdraw_euros(&mut self, availability: bool) -> Bucket {
            self.european_rides = availability;
            self.collected_euros.take_all()
        }

        /// Accounts can buy tickets with dollars and euros
        pub fn buy_ticket(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {

            let dollars = payment.resource_address() == self.collected_dollars.resource_address();
            let euros = payment.resource_address() == self.collected_euros.resource_address();

            let valid_ticket_price = payment.amount() >= self.ticket_price;
            let valid_currency = dollars || euros;

            assert!(valid_ticket_price, "Invalid ticket price");
            assert!(valid_currency, "Invalid currency");

            // Mint ticket token
            let ticket = self.ticket_minter.authorize(|| {
                borrow_resource_manager!(self.ticket_resource_address).mint(1)
            });

            // Put payment into designated collection vault
            if dollars {
                self.collected_dollars.put(payment.take(self.ticket_price));
            } else {
                self.collected_euros.put(payment.take(self.ticket_price));
            }

            // Return minted ticket along with payment change (e.g. ticket=10, payment=15, change=5)
            (ticket, payment)
        }

        /// Accounts can go on rides with tickets they bought (minted)
        pub fn ride(&mut self, payment: Bucket, ride_type: String) {

            let valid_ride = (ride_type.eq("American") && self.american_rides) || (ride_type.eq("European") && self.european_rides);
            let valid_currency = payment.resource_address() == self.ticket_resource_address;
            let valid_price_per_ride = payment.amount() == self.ride_price;

            assert!(valid_ride, "Invalid ride");
            assert!(valid_currency, "Invalid currency");
            assert!(valid_price_per_ride, "Invalid price per ride");

            // Burn ticket token
            self.ticket_minter.authorize(|| {
                payment.burn();
            });

            info!("Welcome to the transit, have fun!")
        }
    }
}
