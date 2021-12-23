mod data;

use scrypto::prelude::*;

use data::*;

blueprint! {
    struct Market {
        order_count: i64,
        currency: Address,
        buy_orders: Vec<Order>,
        sell_orders: Vec<Order>,
        ticket_minter_badge: Vault,
        ticket_nft_def: ResourceDef,
        market_prices: MarketPrices
    }

    impl Market {
        pub fn open(currency: Address) -> Component {
            let ticket_minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Order Ticket Minter Badge")
                .initial_supply_fungible(1);

            let ticket_nft_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Order Ticket")
                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    ticket_minter_badge.resource_def(),
                    MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA
                )
                .no_initial_supply();

            Self {
                order_count: 0,
                currency: currency,
                buy_orders: vec![],
                sell_orders: vec![],
                ticket_minter_badge: Vault::with_bucket(ticket_minter_badge),
                ticket_nft_def: ticket_nft_def,
                market_prices: MarketPrices::new()
            }
                .instantiate()
        }

        /// Yields a ticket (NFT) specifically for this order which can be used to withdraw
        /// from it once it's filled.
        fn order_ticket(&self, order_number: i64, token_address: Address) -> Bucket {
            let ticket = OrderTicket {
                order_number: order_number,
                order_token_address: token_address.to_string(),
                order_currency: self.base_currency()
            };

            self.ticket_minter_badge.authorize(|badge|{
                self.ticket_nft_def.mint_nft(order_number as u128, ticket, badge)
            })
        }

        fn next_order_number(&mut self) -> i64 {
            let number = self.order_count + 1;

            self.order_count = number;

            number
        }

        pub fn market_buy(&mut self, token: Address, payment: Bucket) -> Bucket {
            self.limit_buy(token, 0.into(), payment)
        }

        pub fn market_sell(&mut self, tokens: Bucket) -> Bucket {
            self.limit_sell(tokens, 0.into())
        }

        pub fn limit_sell(&mut self, tokens: Bucket, price: Decimal) -> Bucket {
            let order_number = self.next_order_number();
            let ticket = self.order_ticket(order_number, tokens.resource_def().address());
            let order = Order {
                number: order_number,
                buy: false,
                token: tokens.resource_def(),
                price: price,
                purse: Vault::with_bucket(tokens),
                payment: Vault::new(self.currency)
            };

            self.fill_sell_order(&order);

            self.sell_orders.push(order);

            ticket
        }

        pub fn limit_buy(&mut self, token: Address, price: Decimal, payment: Bucket) -> Bucket {
            assert!(
                payment.resource_def().address() == self.currency,
                "Expecting payment in market currency!"
            );

            let order_number = self.next_order_number();
            let ticket = self.order_ticket(order_number, token);
            let order = Order {
                number: order_number,
                buy: true,
                token: ResourceDef::from(token),
                price: price,
                purse: Vault::new(token),
                payment: Vault::with_bucket(payment)
            };

            self.fill_buy_order(&order);

            self.buy_orders.push(order);

            ticket
        }

        fn fill_buy_order(&mut self, buy_order: &Order) {
            let mut last_price: Option<Decimal> = None;

            for sell_order in self.matching_sell_orders(buy_order) {
                let price: Decimal = sell_order.price;

                self.fill_order(sell_order, buy_order, price);

                last_price = Some(price);

                if buy_order.is_filled() {
                    break;
                }
            }

            for price in last_price {
                self.market_prices.update(buy_order.token_symbol(), price);
            }
        }

        fn fill_sell_order(&mut self, sell_order: &Order) {
            let mut last_price: Option<Decimal> = None;

            for buy_order in self.matching_buy_orders(sell_order) {
                let price: Decimal = buy_order.price;

                self.fill_order(sell_order, buy_order, price);

                last_price = Some(price);

                if sell_order.is_filled() {
                    break;
                }
            }

            for price in last_price {
                self.market_prices.update(sell_order.token_symbol(), price);
            }
        }

        fn fill_order(&self, sell_order: &Order, buy_order: &Order, price: Decimal) {
            let full_payment_amount = price * sell_order.purse.amount();

            if full_payment_amount <= buy_order.payment.amount() {
                self.log_fully_filled_sell_order(full_payment_amount, sell_order, buy_order);

                sell_order.payment.put(buy_order.payment.take(full_payment_amount));
                buy_order.purse.put(sell_order.purse.take_all());
            } else {
                let partial_token_amount = buy_order.payment.amount() / price;
                let payment_amount = buy_order.payment.amount();

                self.log_partially_filled_sell_order(partial_token_amount, payment_amount, sell_order, buy_order);

                sell_order.payment.put(buy_order.payment.take_all());
                buy_order.purse.put(sell_order.purse.take(partial_token_amount));
            }
        }

        fn log_fully_filled_sell_order(&self, payment_amount: Decimal, sell_order: &Order, buy_order: &Order) {
            info!(
                "SO#{} filled fully. Bought {} {} for BO#{} filling it with {} {}, leaving {} {} to spend.",
                sell_order.number,
                sell_order.purse.amount(),
                sell_order.token_symbol(),
                buy_order.number,
                buy_order.purse.amount() + sell_order.purse.amount(),
                buy_order.token_symbol(),
                buy_order.payment.amount() - payment_amount,
                self.base_currency()
            );
        }

        fn log_partially_filled_sell_order(&self, token_amount: Decimal, payment_amount: Decimal, sell_order: &Order, buy_order: &Order) {
            info!(
                "SO#{} filled partially. Bought {} out of {} {} for {} {} to fully fill BO#{}.",
                sell_order.number,
                token_amount,
                sell_order.purse.amount(),
                sell_order.token_symbol(),
                payment_amount,
                self.base_currency(),
                buy_order.number
            );
        }

        /// Given the right order ticket withdraws a sale (sell order) from the market.
        pub fn withdraw_sale(&mut self, ticket_bucket: Bucket) -> (Bucket, Bucket, Bucket) {
            let tickets = ticket_bucket.get_nfts::<OrderTicket>();

            assert!(tickets.len() == 1, "Ticket required");

            let ticket = tickets.first().unwrap().data();

            let index = self.sell_orders
                .iter()
                .position(|order| order.number == ticket.order_number);

            if index.is_some() {
                let i = index.unwrap();
                let order = self.sell_orders.remove(i);

                self.ticket_minter_badge.authorize(|badge| {
                    ticket_bucket.burn_with_auth(badge);
                });

                (order.purse.take_all(), order.payment.take_all(), Bucket::new(self.ticket_nft_address()))
            } else {
                warn!("No matching order found. Returning only ticket.");

                let token_resource_address = Address::from_str(&ticket.order_token_address).unwrap();

                (Bucket::new(token_resource_address), Bucket::new(self.currency), ticket_bucket)
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
        pub fn withdraw_purchase(&mut self, ticket_bucket: Bucket) -> (Bucket, Bucket, Bucket) {
            let tickets = ticket_bucket.get_nfts::<OrderTicket>();

            assert!(tickets.len() == 1, "Ticket required");

            let ticket = tickets.first().unwrap().data();

            let index = self.buy_orders
                .iter()
                .position(|order| order.number == ticket.order_number);

            if index.is_some() {
                let i = index.unwrap();
                let order = self.buy_orders.remove(i);

                self.ticket_minter_badge.authorize(|badge| {
                    ticket_bucket.burn_with_auth(badge);
                });

                (order.purse.take_all(), order.payment.take_all(), Bucket::new(self.ticket_nft_address()))
            } else {
                warn!("No matching order found. Returning only ticket.");

                let token_resource_address = Address::from_str(&ticket.order_token_address).unwrap();

                (Bucket::new(token_resource_address), Bucket::new(self.currency), ticket_bucket)
            }
        }

        fn matching_sell_orders(&self, buy_order: &Order) -> Vec<&Order> {
            let mut orders = (&self.sell_orders)
                .into_iter()
                .filter(|so| self.is_matching_sell_order(buy_order, *so))
                .collect::<Vec<&Order>>();

            orders.sort_by(|a, b| a.price.cmp(&b.price)); // sort by lowest price first

            orders
        }

        fn is_matching_sell_order(&self, buy_order: &Order, sell_order: &Order) -> bool {
            if sell_order.is_filled() || sell_order.token != buy_order.token { return false }
            if buy_order.is_market_order() { return true }

            sell_order.price <= buy_order.price
        }

        fn matching_buy_orders(&self, sell_order: &Order) -> Vec<&Order> {
            let mut orders = (&self.buy_orders)
                .into_iter()
                .filter(|bo| self.is_matching_buy_order(sell_order, *bo))
                .collect::<Vec<&Order>>();

            orders.sort_by(|a, b| b.price.cmp(&a.price)); // sort by highest price first

            orders
        }

        fn is_matching_buy_order(&self, sell_order: &Order, buy_order: &Order) -> bool {
            if buy_order.is_filled() || buy_order.token != sell_order.token { return false }
            if sell_order.is_market_order() { return true }

            buy_order.price >= sell_order.price
        }

        fn base_currency(&self) -> String {
            ResourceDef::from(self.currency).metadata()["symbol"].clone()
        }

        fn ticket_nft_address(&self) -> Address {
            self.ticket_nft_def.address()
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
                    if order.is_market_order() { String::from("market") } else { order.price.to_string() },
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
                    if order.is_market_order() { String::from("market") } else { order.price.to_string() },
                    filled,
                    self.truncate(order.purse.amount().to_string(), 8),
                    self.truncate(order.payment.amount().to_string(), 7)
                );
            }

            info!(" \\------------------------------------------------------/");
        }

        pub fn print_market_prices(&self) {
            info!(" /' MARKET PRICES '\\");
            info!(" +-----------------+");
            info!(" | Asset |  Price  |");
            info!(" +-----------------+");

            for asset in self.market_prices.assets() {
                let name = asset.clone();
                let price = self.market_prices.get(asset).unwrap();

                info!(" | {:>5} | {:>7} |", name, self.truncate(price.to_string(), 7));
            }

            info!(" \\-----------------/");
        }
    }
}
