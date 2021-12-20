use scrypto::prelude::*;
// use rand::Rng;

static SLOT_ITEMS: [&str; 6] = ["CHERRY", "LEMON", "ORANGE", "PLUM", "BELL", "BAR"];

blueprint! {
    struct SlotMachine {
        casino_bank: Vault,
    }

    impl SlotMachine {

        pub fn new(supply: Decimal) -> Component {
            let bank_bucket: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Vegas")
                .metadata("symbol", "VEG")
                .initial_supply_fungible(supply);

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                casino_bank: Vault::with_bucket(bank_bucket),
            }
            .instantiate()
        }

        pub fn free_token(&mut self) -> Bucket {
            info!(
                "My balance is: {} VegasToken. Now giving away a token!",
                self.casino_bank.amount()
            );
            self.casino_bank.take(1)
        }

        pub fn play(&mut self, bet: Bucket) -> Bucket {

            let bet_amount = bet.amount();

            info!("You are betting {} VegasToken.", bet_amount);

            self.casino_bank.put(bet);

            let first_wheel: String = Self::spin_wheel();
            let second_wheel: String = Self::spin_wheel();
            let third_wheel: String = Self::spin_wheel();
            let score = Self::get_score(&first_wheel, &second_wheel, &third_wheel);

            info!(
                "You rolled {}, {}, {} and your score is {}",
                first_wheel, second_wheel, third_wheel, score
            );

            if score == 0 {
                info!("You've lost! Good luck next time.");
                let empty_bucket: Bucket = Bucket::new(RADIX_TOKEN);
                return empty_bucket
            } else {
                let win = bet_amount * score;
                info!("You've won {} VegasToken! Congratulation!", win);
                return self.casino_bank.take(win)
            }
        }

        fn spin_wheel() -> String {
            // rand does not work
            // let mut rng = rand::thread_rng();
            // let rand_num = rng.gen_range(0, 5);
            SLOT_ITEMS[4].to_string()
        }

        fn get_score(first_wheel: &String, second_wheel: &String, third_wheel: &String) -> i32 {

            let score: i32;

            if (first_wheel == "CHERRY") && (second_wheel != "CHERRY") {
                score = 2;
            }
            else if (first_wheel == "CHERRY") && (second_wheel == "CHERRY") && (third_wheel != "CHERRY") {
                score = 5;
            }
            else if (first_wheel == "CHERRY") && (second_wheel == "CHERRY") && (third_wheel == "CHERRY") {
                score = 7;
            }
            else if (first_wheel == "ORANGE") && (second_wheel == "ORANGE") && ((third_wheel == "ORANGE") || (third_wheel == "BAR")) {
                score = 10;
            }
            else if (first_wheel == "PLUM") && (second_wheel == "PLUM") && ((third_wheel == "PLUM") || (third_wheel == "BAR")) {
                score = 14;
            }
            else if (first_wheel == "BELL") && (second_wheel == "BELL") && ((third_wheel == "BELL") || (third_wheel == "BAR")) {
                score = 20;
            }
            else if (first_wheel == "BAR") && (second_wheel == "BAR") && (third_wheel == "BAR") {
                score = 250;
            }
            else {
                score = 0;
            }
            return score;
        }

    }
}
