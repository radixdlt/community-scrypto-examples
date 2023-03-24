use scrypto::prelude::*;

#[blueprint]
mod gumball_machine {

    struct GumballMachine {
        gumball_vault: Vault,
        price: Decimal,
        collected_xrd: Vault,
    }

    impl GumballMachine {
        pub fn instantiate_gumball_machine(price: Decimal) -> (ComponentAddress, Bucket) {
            // Create an admin badge for the gumball owner
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("symbol", "ADMIN")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            // Create a bucket for gumballs where the owner can mint and burn
            let gumballs: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Gumball")
                .metadata("symbol", "GMB")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(rule!(require(admin_badge.resource_address())), LOCKED)
                .burnable(rule!(require(admin_badge.resource_address())), LOCKED)
                .mint_initial_supply(1000);

            let access_rules = AccessRules::new()
                .method(
                    "set_price",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "withdraw",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "mint_gumballs",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "check_gumballs",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            let mut component = Self {
                gumball_vault: Vault::with_bucket(gumballs),
                price,
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate();

            // Add the access rules to the component
            component.add_access_check(access_rules);
            let component = component.globalize();

            (component, admin_badge)
        }

        //Set new price for gumball
        pub fn set_price(&mut self, price: Decimal) {
            // Only the owner with the admin badge can set the price;
            // set the price to the new value if it is greater than 0 and less than 1
            assert!(
                price > dec!("0") && price < dec!("1"),
                "Price must be greater than 0 and less than 1"
            );
            self.price = price;
        }
        pub fn check_price(&self) -> Decimal {
            // Return the price of the gumball
            return self.price;
        }
        // withdraw collected xrd from the machine
        pub fn withdraw(&mut self, amount: Bucket) -> Bucket {
            // Only the owner with the admin badge can withdraw
            // the collected XRD
            // Withdraw the amount from the collected XRD vault
            // Transfer the amount to the owner
            assert!(
                amount.amount() <= self.collected_xrd.amount(),
                "Not enough XRD to wihdraw as requested"
            );
            let withdrawn_amount = self.collected_xrd.take(amount.amount());
             return withdrawn_amount;
        }

        // Mint more gumballs
        pub fn mint_gumballs(&mut self, number: Decimal) -> Bucket {
            // Only the owner with the admin badge can mint more gumballs
            // Mint the amount of gumballs into the gumball vault
            let new_gumballs =
                borrow_resource_manager!(self.gumball_vault.resource_address()).mint(number);
            return new_gumballs;
        }
        // check the number of gumballs in the machine
        pub fn check_gumballs(&self) -> Decimal {
            // Return the number of gumballs in the gumball vault
            let gumball_left: Decimal = self.gumball_vault.amount();
            return gumball_left;
        }
        // users can buy gumball with xrd
        pub fn buy_gumball(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // Check if the amount of XRD sent is equal or grater than the price
            // If the amount is equal to the price, transfer the amount
            // of XRD to the collected XRD vault
            // Transfer the gumball from the gumball vault to the user
            let amount: Decimal = payment.amount();
            assert!(amount >= self.price, "Not enough XRD");
            let max_amount_of_gumballs: Decimal = amount / self.price;
            let owner_share = payment.take(self.price * max_amount_of_gumballs);
            self.collected_xrd.put(owner_share);
            let gumball: Bucket = self.gumball_vault.take(max_amount_of_gumballs);
            return (gumball, payment);
        }
    }
}
