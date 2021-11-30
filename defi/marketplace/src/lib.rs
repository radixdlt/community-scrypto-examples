mod data;

use scrypto::prelude::*;

use data::*;

blueprint! {
    struct Market {
        order_count: i64,
        currency: Address,
        buy_orders: Vec<Order>,
        sell_orders: Vec<Order>,
        ticket_minter_badge: Vault
    }

    impl Market {
        pub fn open(currency: Address) -> Component {
            let ticket_minter_badge = ResourceBuilder::new()
                .metadata("name", "Order Ticket Minter Badge")
                .new_badge_fixed(1);

            Self {
                order_count: 0,
                currency: currency,
                buy_orders: vec![],
                sell_orders: vec![],
                ticket_minter_badge: Vault::with_bucket(ticket_minter_badge)
            }
                .instantiate()
        }

        /// Yields a new token specifically for this order which can be used to withdraw
        /// from it once it's filled.
        fn order_ticket(&self, order_number: i64, token_address: Address) -> (Bucket, Address) {
            let ticket_resource_def = ResourceBuilder::new()
                .metadata("name", format!("Order Ticket #{}", order_number))
                .metadata("symbol", format!("OT-{}", order_number))
                .metadata("order_number", format!("{}", order_number))
                .metadata("order_token_address", token_address.to_string())
                .metadata("order_currency", self.base_currency())
                .new_token_mutable(self.ticket_minter_badge.resource_def());

            self.ticket_minter_badge.authorize(|badge|{
                (ticket_resource_def.mint(1, badge), ticket_resource_def.address())
            })
        }

        fn next_order_number(&mut self) -> i64 {
            let number = self.order_count + 1;

            self.order_count = number;

            number
        }

        pub fn sell(&mut self, tokens: Bucket, price: Decimal) -> Bucket {
            let order_number = self.next_order_number();
            let (ticket, address) = self.order_ticket(order_number, tokens.resource_def().address());
            let order = Order {
                number: order_number,
                buy: false,
                token: tokens.resource_def(),
                price: price,
                purse: Vault::with_bucket(tokens),
                payment: Vault::new(self.currency),
                ticket_resource_address: address
            };

            self.sell_orders.push(order);

            ticket
        }

        pub fn buy(&mut self, token: Address, price: Decimal, payment: Bucket) -> Bucket {
            assert!(
                payment.resource_def().address() == self.currency,
                "Expecting payment in market currency!"
            );

            let order_number = self.next_order_number();
            let (ticket, address) = self.order_ticket(order_number, token);
            let order = Order {
                number: order_number,
                buy: true,
                token: ResourceDef::from(token),
                price: price,
                purse: Vault::new(token),
                payment: Vault::with_bucket(payment),
                ticket_resource_address: address
            };

            self.buy_orders.push(order);

            ticket
        }

        pub fn fill_orders(&self) {
            let buy_orders = &self.buy_orders;
            for buy_order in buy_orders {
                self.fill_buy_order(&buy_order);
            }
        }

        fn fill_buy_order(&self, buy_order: &Order) {
            for sell_order in self.matching_sell_orders(buy_order) {
                let full_payment_amount = sell_order.price * sell_order.purse.amount();

                if full_payment_amount <= buy_order.payment.amount() {
                    info!(
                        "SO#{} filled fully. Bought {} {} for BO#{} filling it with {} {}, leaving {} {} to spend.",
                        sell_order.number,
                        sell_order.purse.amount(),
                        sell_order.token_symbol(),
                        buy_order.number,
                        buy_order.purse.amount() + sell_order.purse.amount(),
                        buy_order.token_symbol(),
                        buy_order.payment.amount() - full_payment_amount,
                        self.base_currency()
                    );

                    sell_order.payment.put(buy_order.payment.take(full_payment_amount));
                    buy_order.purse.put(sell_order.purse.take_all());
                } else {
                    let partial_token_amount = buy_order.payment.amount() / sell_order.price;
                    let payment_amount = buy_order.payment.amount();

                    info!(
                        "SO#{} filled partially. Bought {} out of {} {} for {} {} to fully fill BO#{}.",
                        sell_order.number,
                        partial_token_amount,
                        sell_order.purse.amount(),
                        sell_order.token_symbol(),
                        payment_amount,
                        self.base_currency(),
                        buy_order.number
                    );

                    sell_order.payment.put(buy_order.payment.take_all());
                    buy_order.purse.put(sell_order.purse.take(partial_token_amount));
                }
            }
        }

        /// Given the right order ticket withdraws a sale (sell order) from the market.
        pub fn withdraw_sale(&mut self, ticket: Bucket) -> (Bucket, Bucket, Bucket) {
            assert!(ticket.amount() > 0.into(), "Ticket required");

            let index = self.sell_orders
                .iter()
                .position(|order| order.number.to_string() == ticket.resource_def().metadata()["order_number"]);

            if index.is_some() {
                let i = index.unwrap();

                assert!(self.sell_orders[i].ticket_resource_address == ticket.resource_def().address(), "Invalid ticket!");

                let order = self.sell_orders.remove(i);

                self.ticket_minter_badge.authorize(|badge| {
                    ticket.burn(badge);
                });

                (order.purse.take_all(), order.payment.take_all(), Bucket::new(order.ticket_resource_address))
            } else {
                warn!("No matching order found. Returning only ticket.");

                let token_resource_address = Address::from_str(&ticket.resource_def().metadata()["order_token_address"]).unwrap();

                (Bucket::new(token_resource_address), Bucket::new(self.currency), ticket)
            }
        }

        /// Given the right order ticket withdraws a purchase (buy order) from the market.
        ///
        /// Can be called even when the order hasn't been fully filled yet. In that case the remaining payment
        /// tokens will be returned along with what ever tokens have been bought so far. The order will be removed
        /// from the market in any case.
        ///
        /// Always returns a 3-tuple of buckets `(purchased_tokens, payment_change, order_ticket)`.
        /// If no order was found, the first two will be empty while the 3rd one contains the unused ticket.
        /// If an order couldn't be found the 3rd bucket will be empty since the ticket will be burned.
        pub fn withdraw_purchase(&mut self, ticket: Bucket) -> (Bucket, Bucket, Bucket) {
            assert!(ticket.amount() > 0.into(), "Ticket required");

            let index = self.buy_orders
                .iter()
                .position(|order| order.number.to_string() == ticket.resource_def().metadata()["order_number"]);

            if index.is_some() {
                let i = index.unwrap();

                assert!(self.buy_orders[i].ticket_resource_address == ticket.resource_def().address(), "Invalid ticket!");

                let order = self.buy_orders.remove(i);

                self.ticket_minter_badge.authorize(|badge| {
                    ticket.burn(badge);
                });

                (order.purse.take_all(), order.payment.take_all(), Bucket::new(order.ticket_resource_address))
            } else {
                warn!("No matching order found. Returning only ticket.");

                let token_resource_address = Address::from_str(&ticket.resource_def().metadata()["order_token_address"]).unwrap();

                (Bucket::new(token_resource_address), Bucket::new(self.currency), ticket)
            }
        }

        fn matching_sell_orders(&self, buy_order: &Order) -> Vec<&Order> {
            (&self.sell_orders)
                .into_iter()
                .filter(|so| self.is_matching_sell_order(buy_order, *so))
                .collect::<Vec<&Order>>()
        }

        fn is_matching_sell_order(&self, buy_order: &Order, sell_order: &Order) -> bool {
            !sell_order.is_filled() && sell_order.token == buy_order.token && buy_order.price >= sell_order.price
        }

        fn base_currency(&self) -> String {
            ResourceDef::from(self.currency).metadata()["symbol"].clone()
        }

        fn truncate(&self, str: String, length: usize) -> String {
            str.chars().take(length).collect::<String>()
        }

        pub fn print_order_book(&self) {
            let base_currency = self.base_currency();

            info!(" /''''''''''''''''' {:>5} BUY ORDERS '''''''''''''''''\\", base_currency);
            info!(" +----------------------------------------------------+");
            info!(" | #    | Token |   Bid   | Filled | Bought | Payment |");
            info!(" +----------------------------------------------------+");

            for order in &self.buy_orders {
                let filled = if order.is_filled() { "yes" } else {
                    if order.purse.amount() == 0.into() { "no" } else { "part" }
                };

                info!(
                    " | {:>4} | {:>5} | {:>7} | {:>6} | {:>6} | {:>7} |",
                    order.number,
                    order.token_symbol(),
                    order.price.to_string(),
                    filled,
                    self.truncate(order.purse.amount().to_string(), 6), // decimal fmt doesn't seem to work with info! macro so we do this
                    self.truncate(order.payment.amount().to_string(), 7)
                );
            }

            info!(" \\----------------------------------------------------/");
            info!("");
            info!(" /'''''''''''''''''' {:>5} SELL ORDERS '''''''''''''''''\\", base_currency);
            info!(" +------------------------------------------------------+");
            info!(" | #    | Token |   Ask   | Filled | For sale | Payment |");
            info!(" +------------------------------------------------------+");

            for order in &self.sell_orders {
                let filled = if order.is_filled() { "yes" } else {
                    if order.payment.amount() == 0.into() { "no" } else { "part" }
                };

                info!(
                    " | {:>4} | {:>5} | {:>7} | {:>6} | {:>8} | {:>7} |",
                    order.number,
                    order.token_symbol(),
                    order.price.to_string(),
                    filled,
                    self.truncate(order.purse.amount().to_string(), 8),
                    self.truncate(order.payment.amount().to_string(), 7)
                );
            }

            info!(" \\------------------------------------------------------/");
        }
    }
}
