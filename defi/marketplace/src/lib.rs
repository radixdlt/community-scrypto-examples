mod data;

use scrypto::prelude::*;

use data::*;

blueprint! {
    struct Market {
        order_count: i64,
        currency: Address,
        buy_orders: Vec<BuyOrder>,
        sell_orders: Vec<SellOrder>
    }

    impl Market {
        pub fn open(currency: Address) -> Component {
            Self {
                order_count: 0,
                currency: currency,
                buy_orders: vec![],
                sell_orders: vec![]
            }
                .instantiate()
        }

        /// Yields a new token specifically for this order which can be used to withdraw
        /// from it once it's filled.
        /// 
        /// @TODO Do this properly. Right now anyone can pass any token with the right metadata
        ///       to withdraw.
        fn order_ticket(&self, order_number: i64) -> Bucket {
            ResourceBuilder::new()
                .metadata("name", format!("Order Ticket #{}", order_number))
                .metadata("symbol", format!("OT-{}", order_number))
                .metadata("url", format!("urn:market/order/{}", order_number))
                .metadata("order_number", format!("{}", order_number))
                .metadata("order_currency", self.base_currency())
                .new_token_fixed(1)
        }

        fn next_order_number(&mut self) -> i64 {
            let number = self.order_count + 1;

            self.order_count = number;

            number
        }

        pub fn sell(&mut self, tokens: Bucket, price: Decimal) -> Bucket {
            let order_number = self.next_order_number();
            let order = SellOrder {
                number: order_number,
                token: tokens.resource_def(),
                ask: price,
                sale: Vault::with_bucket(tokens),
                payment: Vault::new(self.currency)
            };

            self.sell_orders.push(order);

            self.order_ticket(order_number)
        }

        pub fn buy(&mut self, token: Address, price: Decimal, payment: Bucket) -> Bucket {
            assert!(
                payment.resource_def().address() == self.currency,
                "Expecting payment in market currency!"
            );

            let order_number = self.next_order_number();
            let order = BuyOrder {
                number: order_number,
                token: ResourceDef::from(token),
                bid: price,
                purchase: Vault::new(token),
                payment: Vault::with_bucket(payment)
            };

            self.buy_orders.push(order);

            self.order_ticket(order_number)
        }

        pub fn fill_orders(&self) {
            let buy_orders = &self.buy_orders;
            for buy_order in buy_orders {
                self.fill_buy_order(&buy_order);
            }
        }

        fn fill_buy_order(&self, buy_order: &BuyOrder) {
            for sell_order in self.matching_sell_orders(buy_order) {
                let full_payment_amount = sell_order.ask * sell_order.sale.amount();

                if full_payment_amount <= buy_order.payment.amount() {
                    info!(
                        "SO#{} filled fully. Bought {} {} for BO#{} filling it with {} {}, leaving {} {} to spend.",
                        sell_order.number,
                        sell_order.sale.amount(),
                        sell_order.token_symbol(),
                        buy_order.number,
                        buy_order.purchase.amount() + sell_order.sale.amount(),
                        buy_order.token_symbol(),
                        buy_order.payment.amount() - full_payment_amount,
                        self.base_currency()
                    );

                    sell_order.payment.put(buy_order.payment.take(full_payment_amount));
                    buy_order.purchase.put(sell_order.sale.take_all());
                } else {
                    let partial_token_amount = buy_order.payment.amount() / sell_order.ask;
                    let payment_amount = buy_order.payment.amount();

                    info!(
                        "SO#{} filled partially. Bought {} out of {} {} for {} {} to fully fill BO#{}.",
                        sell_order.number,
                        partial_token_amount,
                        sell_order.sale.amount(),
                        sell_order.token_symbol(),
                        payment_amount,
                        self.base_currency(),
                        buy_order.number
                    );

                    sell_order.payment.put(buy_order.payment.take_all());
                    buy_order.purchase.put(sell_order.sale.take(partial_token_amount));
                }
            }
        }

        pub fn withdraw_sale_payment(&self, ticket: Bucket) -> Bucket {
            for order in &self.sell_orders {
                let metadata = ticket.resource_def().metadata();

                if order.number.to_string() == metadata["order_number"] && order.currency() == metadata["order_currency"] {
                    return order.payment.take_all();
                }
            }

            panic!("No matching order found");
        }

        pub fn withdraw_purchase(&self, ticket: Bucket) -> (Bucket, Bucket) {
            for order in &self.buy_orders {
                if order.number.to_string() == ticket.resource_def().metadata()["order_number"] {
                    info!("FOUND IT!");
                    return (order.purchase.take_all(), order.payment.take_all());
                }
            }

            panic!("No matching order found");
        }

        fn matching_sell_orders(&self, buy_order: &BuyOrder) -> Vec<&SellOrder> {
            (&self.sell_orders)
                .into_iter()
                .filter(|so| self.is_matching_sell_order(buy_order, *so))
                .collect::<Vec<&SellOrder>>()
        }

        fn is_matching_sell_order(&self, buy_order: &BuyOrder, sell_order: &SellOrder) -> bool {
            !sell_order.is_filled() && sell_order.token == buy_order.token && buy_order.bid >= sell_order.ask
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
                    if order.purchase.amount() == 0.into() { "no" } else { "part" }
                };

                info!(
                    " | {:>4} | {:>5} | {:>7} | {:>6} | {:>6} | {:>7} |",
                    order.number,
                    order.token_symbol(),
                    order.bid.to_string(),
                    filled,
                    self.truncate(order.purchase.amount().to_string(), 6), // decimal fmt doesn't seem to work with info! macro so we do this
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
                    order.ask.to_string(),
                    filled,
                    self.truncate(order.sale.amount().to_string(), 8),
                    self.truncate(order.payment.amount().to_string(), 7)
                );
            }

            info!(" \\------------------------------------------------------/");
        }
    }
}
