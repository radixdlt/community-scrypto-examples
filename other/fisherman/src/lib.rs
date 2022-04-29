use scrypto::prelude::*;

blueprint! {
    struct Fisherman {
        admin_badge: ResourceAddress,
        fee: Decimal,
        collected_fees: Vault,
        prize_pool: Vault,
        // game id
        game_id: u64, 
        // flag o track game is ready for placing bets
        is_ready: bool, 
        // price per bet
        price: Decimal, 
        // collect history of games played
        history: LazyMap<u64, Vault>,
        players: HashMap<ComponentAddress, (Decimal, usize)>
    }

    impl Fisherman {
        
        pub fn new(fee_percent: Decimal) -> (ComponentAddress, Bucket) {
            let admin_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Fisherman Badge Mint Auth")
                .initial_supply(1);

            let access_rules = AccessRules::new()
                .method("withdraw", rule!(require(admin_bucket.resource_address())))
                .method("new_game", rule!(require(admin_bucket.resource_address())))
                .method("finish", rule!(require(admin_bucket.resource_address())))
                .default(rule!(allow_all));

            let component = Self {
                admin_badge: admin_bucket.resource_address(),
                collected_fees: Vault::new(RADIX_TOKEN),
                fee: fee_percent,
                prize_pool: Vault::new(RADIX_TOKEN),
                game_id: 0,
                is_ready: false,
                price: Decimal::zero(),
                history: LazyMap::new(),
                players: HashMap::new()
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (component, admin_bucket)
        }

        pub fn new_game(&mut self, price: Decimal) {
            assert!(price > Decimal::zero(), "Price per game cannot be zero");
            assert!(!self.is_ready, "Current game is ready. Please finish it before making a new one");

            // make a new game
            // increment an id
            self.game_id += 1;
            self.is_ready = true;
            self.price = price;
        }

        pub fn capture(&mut self, player: ComponentAddress, depth: Decimal, mut payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == RADIX_TOKEN.into(), "You must use Radix (XRD).");
            assert!(self.is_ready, "Current game is not ready yet");
            assert!(!self.players.contains_key(&player), "You are already in the game");
            assert!(depth > Decimal::zero(), "Depth cannot be zero");
            assert!(depth <= dec!("10"), "Depth more than 10 is not allowed");
            assert!(payment.amount() >= self.price, "Not enough amount to play");

            // we use special target possibility. Bigger depth increases fish size but reduces target (chances to capture)
            let portion = dec!("100") / depth;
            let target = (portion / 100) * 10000 * dec!("99")/dec!("100");
            info!("Target: {}", target);

            // take hash
            let hash = Runtime::transaction_hash().to_string();

            //take first 10 bits of result hash
            let seed = &hash[0..10];

            //convert 10 hex bits to decimal
            let mut result = i128::from_str_radix(&seed, 16).unwrap();

            //take decimal mod 10,001
            result = result % 10001;
            info!("Result: {}", result);

            let mut weight = Decimal::zero();
            if Decimal::from(result) < target {
                // calculate fish weight (let's say in grams)
                // the lower result - the heavier fish
                result = 10000-result;

                weight = Decimal::from(result) * depth;
                info!("You captured the fish: {}g weight",weight );
            }else{
                info!("You didn't capture anything");
            }

            // save player result
            let player_num = self.players.len() + 1;
            self.players.insert(player, (weight, player_num));

            // take payment
            let bucket = payment.take(self.price);
            self.prize_pool.put(bucket);

            // return the rest
            payment
        }

        pub fn finish(&mut self) {
            assert!(self.is_ready, "Current game is not ready");

            // find player with heavier fish
            let mut weight = Decimal::zero();
            let mut winner = self.players.keys().next().unwrap();
            let mut num: usize = 0;

            for (key, value) in &self.players {

                let pl_weight = value.0;
                let pl_num = value.1;

                // here we check who has bigger weight value
                // if we found that the weight is equal, then we check who was first
                if (pl_weight > weight) || (pl_weight > Decimal::zero() && pl_weight == weight && pl_num < num){
                    winner = &key;
                    weight = pl_weight;
                    num = pl_num;
                } 
            }

            if weight > Decimal::zero() {
                // send money to winner minus fee
                let fee = self.prize_pool.amount() * self.fee / dec!("100");
                self.collected_fees.put(self.prize_pool.take(fee));
                borrow_component!(*winner).call::<()>("deposit", vec![scrypto_encode(&self.prize_pool.take_all())]);
            }else{
                info!("Ooops, we have no winner. The prize pool will be used in the next game");
            }

            // end game
            self.is_ready = false;

            // save history
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Fisherman Game Badge")
                .metadata("symbol", "FGB")
                .metadata("epoch", Runtime::current_epoch().to_string())
                .metadata("winner", winner.to_string())
                .metadata("winner_num", num.to_string())
                .metadata("fish_weight", weight.to_string())
                .initial_supply(1);

            let vault = Vault::with_bucket(bucket);
            self.history.insert(self.game_id, vault);

            // clear players
            self.players.clear();
        }

        pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
            assert!(self.collected_fees.amount() >= amount, "Withdraw amount is bigger than available assets");

            self.collected_fees.take(amount)
        }
        
    }
}
