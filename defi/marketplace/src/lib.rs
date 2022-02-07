mod data;

use scrypto::prelude::*;

use data::*;

blueprint! {
    struct Market {
        order_count: i64,
        currency: Address,
        orders: Vec<Order>,
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
                .flags(MINTABLE | BURNABLE)
                .badge(
                    ticket_minter_badge.resource_def(),
                    MAY_MINT | MAY_BURN
                )
                .no_initial_supply();

            Self {
                order_count: 0,
                currency: currency,
                orders: vec![],
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
            let token = tokens.resource_def().address();
            let currency = self.currency;

            self.create_limit_order(token, tokens, |order_number, _tokens_| Market::make_sell_order(order_number, _tokens_, price, currency))
        }

        pub fn limit_buy(&mut self, token: Address, price: Decimal, payment: Bucket) -> Bucket {
            assert!(
                payment.resource_def().address() == self.currency,
                "Expecting payment in market currency!"
            );

            self.create_limit_order(token, payment, |order_number, _payment_| Market::make_buy_order(order_number, token, price, _payment_))
        }

        ///
        /// Creates a new limit order using the given factory method and inserts into the list of orders.
        ///
        /// @param token The address of the kind of token that is traded.
        /// @param bucket Used if the factory needs a bucket since Buckets cannot be captured in closures as far as I can tell.
        /// @param make_order Function creating the new order using the given order number.
        fn create_limit_order<F>(&mut self, token: Address, bucket: Bucket, make_order: F) -> Bucket where F: Fn(i64, Bucket) -> Order {
            let order_number = self.next_order_number();
            let ticket = self.order_ticket(order_number, token);

            let order = make_order(order_number, bucket);

            self.fill_order(&order);

            self.orders.push(order);

            ticket
        }

        fn make_buy_order(order_number: i64, token: Address, price: Decimal, payment: Bucket) -> Order {
            Order {
                number: order_number,
                buy: true,
                token: ResourceDef::from(token),
                price: price,
                purse: Vault::new(token),
                payment: Vault::with_bucket(payment)
            }
        }

        fn make_sell_order(order_number: i64, tokens: Bucket, price: Decimal, currency: Address) -> Order {
            Order {
                number: order_number,
                buy: false,
                token: tokens.resource_def(),
                price: price,
                purse: Vault::with_bucket(tokens),
                payment: Vault::new(currency)
            }
        }

        fn fill_order(&mut self, order: &Order) {
            let mut last_price: Option<Decimal> = None;

            for matched_order in self.matching_orders(order) {
                let price: Decimal = matched_order.price;

                self.fill_matched_order(order, matched_order, price);

                last_price = Some(price);

                if order.is_filled() {
                    break;
                }
            }

            for price in last_price {
                self.market_prices.update(order.token_symbol(), price);
            }
        }

        fn fill_matched_order(&self, order_a: &Order, order_b: &Order, price: Decimal) {
            assert!(order_a.is_buy_order() ^ order_b.is_buy_order(), "Expected a buy and a sell order.");

            if order_a.is_buy_order() && order_b.is_sell_order() {
                self.fill_sell_order(order_b, order_a, price);
            } else {
                self.fill_sell_order(order_a, order_b, price);
            }
        }

        fn fill_sell_order(&self, sell_order: &Order, buy_order: &Order, price: Decimal) {
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

        /// Given the right order ticket withdraws an order from the market.
        ///
        /// Can be called even when the order hasn't been fully filled yet. In that case the remaining payment
        /// tokens will be returned along with whatever tokens have been bought so far. The order will be removed
        /// from the market in any case.
        ///
        /// Always returns a 3-tuple of buckets `(purchased_tokens, payment_change, order_ticket)`.
        /// If no order was found, the first two will be empty while the 3rd one contains the unused ticket.
        /// If an order couldn't be found the 3rd bucket will be empty since the ticket will be burned.
        pub fn withdraw_order(&mut self, ticket_bucket: Bucket) -> (Bucket, Bucket, Bucket) {
            let tickets = ticket_bucket.get_nfts::<OrderTicket>();

            assert!(tickets.len() == 1, "Ticket required");

            let ticket = tickets.first().unwrap().data();

            let index = self.orders
                .iter()
                .position(|order| order.number == ticket.order_number);

            if index.is_some() {
                let i = index.unwrap();
                let order = self.orders.remove(i);

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

        fn matching_orders(&self, order: &Order) -> Vec<&Order> {
            let buy = order.is_buy_order();

            let mut orders = (&self.orders)
                .into_iter()
                .filter(|o| o.is_buy_order() != buy) // match buy to sell orders and vice versa
                .filter(|o| (buy && self.is_matching_sell_order(order, *o)) || self.is_matching_buy_order(order, *o))
                .collect::<Vec<&Order>>();

            orders.sort_by(|a, b| {
                if order.is_buy_order() {
                    a.price.cmp(&b.price) // sort by lowest price first
                } else {
                    b.price.cmp(&a.price) // sort by highest price first
                }
            });

            orders
        }

        fn is_matching_sell_order(&self, buy_order: &Order, sell_order: &Order) -> bool {
            if sell_order.is_filled() || sell_order.token != buy_order.token { return false }
            if buy_order.is_market_order() { return true }

            sell_order.price <= buy_order.price
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

        pub fn print_order_book(&self) {
            self.print_orders(true);

            info!("");

            self.print_orders(false);
        }

        fn print_orders(&self, buy: bool) {
            let title = if buy { "BUY" } else { "SELL" };
            let kind = if buy { "Bid" } else { "Ask" };
            let store = if buy { "Bought" } else { "For sale" };

            info!(" \\----------------------------------------------------/");
            info!("");
            info!(" /'''''''''''''''''' {:>5} {:>4} ORDERS '''''''''''''''''\\", self.base_currency(), title);
            info!(" +------------------------------------------------------+");
            info!(" | #    | Token | {:>7} | Filled | {:>8} | Payment |", kind, store);
            info!(" +------------------------------------------------------+");

            let orders = (&self.orders).into_iter().filter(|o| o.is_buy_order() == buy).collect::<Vec<&Order>>();

            for order in orders {
                let filled = if order.is_filled() {
                    "yes"
                } else {
                    if buy {
                        if order.purse.amount() == 0.into() { "no" } else { "part" }
                    } else {
                        if order.payment.amount() == 0.into() { "no" } else { "part" }
                    }
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
