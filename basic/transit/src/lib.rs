use scrypto::prelude::*;

blueprint! {
    struct Transit {
        /// Transit rider public key/epoch
        riders: LazyMap<Vec<Address>, u64>,
        /// Ticket Resource Definition and Mint/Burn Authorization
        ticket_resource_def: ResourceDef,
        ticket_minter: Vault,
        /// Enable/Disable rides
        american_rides: bool,
        european_rides: bool,
        /// Ticket/Ride prices
        ticket_price: Decimal,
        ride_price: Decimal,
        /// Control rides/Withdraw payments
        american_host_badge: ResourceDef,
        european_host_badge: ResourceDef,
        /// Collected revenue from ticket sales
        collected_dollars: Vault,
        collected_euros: Vault
    }

    impl Transit {
        pub fn new(
            price_per_ticket: Decimal,
            price_per_ride: Decimal,
            dollar: Address,
            euro: Address
        ) -> (Component, Bucket, Bucket) {

            let valid_cli_arguments = price_per_ticket > 0.into() && price_per_ride > 0.into();
            assert!(valid_cli_arguments, "Invalid CLI arguments");

            let american_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "American Host Badge")
                .metadata("symbol", "APB")
                .metadata("description", "A badge that grants american host privileges")
                .initial_supply_fungible(1);

            let european_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "European Host Badge")
                .metadata("symbol", "EPB")
                .metadata("description", "A badge that grants european host privileges")
                .initial_supply_fungible(1);

            let ticket_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Ticket Mint Auth")
                .initial_supply_fungible(1);

            let ticket_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Ticket")
                .metadata("symbol", "TK")
                .metadata("description", "A ticket used for rides")
                .flags(MINTABLE | BURNABLE)
                .badge(ticket_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let component = Self {
                riders: LazyMap::new(),
                ticket_resource_def,
                ticket_minter: Vault::with_bucket(ticket_minter),
                american_rides: true,
                european_rides: true,
                ticket_price: price_per_ticket,
                ride_price: price_per_ride,
                american_host_badge: american_badge.resource_def(),
                european_host_badge: european_badge.resource_def(),
                collected_dollars: Vault::new(dollar),
                collected_euros: Vault::new(euro)
            }
            .instantiate();

            (component, american_badge, european_badge)
        }

        /// american_host_badge can take_all the collected_dollars and control american_rides
        #[auth(american_host_badge)]
        pub fn withdraw_dollars(&mut self, availability: bool) -> Bucket {
            self.american_rides = availability;
            self.collected_dollars.take_all()
        }

        /// european_host_badge can take_all the collected_euros and control european_rides
        #[auth(european_host_badge)]
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
            let ticket = self.ticket_minter.authorize(|badge| {
                self.ticket_resource_def.mint(1, badge)
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
            let valid_currency = payment.resource_address() == self.ticket_resource_def.address();
            let valid_price_per_ride = payment.amount() == self.ride_price;

            assert!(valid_ride, "Invalid ride");
            assert!(valid_currency, "Invalid currency");
            assert!(valid_price_per_ride, "Invalid price per ride");

            // Burn ticket token
            self.ticket_minter.authorize(|badge| {
                payment.burn_with_auth(badge);
            });

            info!("Welcome to the transit, have fun!")
        }
    }
}
