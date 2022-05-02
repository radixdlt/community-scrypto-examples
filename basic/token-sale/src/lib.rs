use std::cmp::min;

use scrypto::prelude::*;

blueprint! {
    struct TokenSale {
        admin_badge: ResourceAddress,
        tokens_for_sale: Vault,
        payment_vault: Vault,
        sale_ticket_minter: Vault,
        sale_tickets: ResourceAddress,
        price_per_token: Decimal,
        max_personal_allocation: Decimal,
        sale_started: bool,
    }

    impl TokenSale {
        pub fn new(
            tokens_for_sale: Bucket,
            payment_token: ResourceAddress,
            price_per_token: Decimal,
            max_personal_allocation: Decimal,
        ) -> (ComponentAddress, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "admin_badge")
                .initial_supply(1);

            let sale_ticket_minter = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "sale_ticket_minter")
                .initial_supply(1);

            let sale_tickets = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Sale Ticket Token")
                .metadata("symbol", "STT")
                .mintable(rule!(require(sale_ticket_minter.resource_address())), LOCKED)
                .burnable(rule!(require(sale_ticket_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let component = Self {
                admin_badge: admin_badge.resource_address(),
                tokens_for_sale: Vault::with_bucket(tokens_for_sale),
                payment_vault: Vault::new(payment_token),
                sale_ticket_minter: Vault::with_bucket(sale_ticket_minter),
                sale_tickets,
                price_per_token,
                max_personal_allocation,
                sale_started: false,
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("create_tickets", rule!(require(admin_badge.resource_address())))
                .method("start_sale", rule!(require(admin_badge.resource_address())))
                .method("withdraw_payments", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(), admin_badge)
        }

        pub fn create_tickets(&mut self, amount: i32) -> Bucket {
            self.sale_ticket_minter
                .authorize(|| borrow_resource_manager!(self.sale_tickets).mint(amount))
        }

        pub fn start_sale(&mut self) {
            self.sale_started = true
        }

        pub fn withdraw_payments(&mut self) -> Bucket {
            self.payment_vault.take_all()
        }

        fn has_tokens_left(&self) -> bool {
            self.tokens_for_sale.amount().is_positive()
        }

        pub fn buy_tokens(&mut self, mut payment: Bucket, ticket: Bucket) -> (Bucket, Bucket) {
            // Check the sale has already started and is not over yet
            assert!(self.sale_started, "The sale has not started yet");
            assert!(self.has_tokens_left(), "The sale has ended already");

            // Check the user's ticket and burn it
            assert!(
                ticket.amount() == Decimal::one(),
                "You need to send exactly one ticket in order to participate in the sale"
            );
            self.sale_ticket_minter
                .authorize(|| ticket.burn());

            // Calculate the actual amount of tokens that the user can buy
            let payment_amount = min(payment.amount(), self.max_personal_allocation);
            let buy_ammount = payment_amount / self.price_per_token;
            let actual_buy_amount = min(self.tokens_for_sale.amount(), buy_ammount);
            let actual_payment_amount = actual_buy_amount * self.price_per_token;

            // Perform the token buy operation
            self.payment_vault.put(payment.take(actual_payment_amount));
            let bought_tokens = self.tokens_for_sale.take(actual_buy_amount);

            // Return the bought tokens and the amount the user might have overpaid
            (bought_tokens, payment)
        }
    }
}
