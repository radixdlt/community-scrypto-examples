use scrypto::prelude::*;
use scrypto::core::Uuid;

const CHERRY: i32 = 0;
const LEMON:  i32 = 1;
const ORANGE: i32 = 2;
const PLUM:   i32 = 3;
const BELL:   i32 = 4;
const BAR:    i32 = 5;

const NUMBER_OF_ITEMS: u128 = 6;

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

            Self {
                casino_bank: Vault::with_bucket(bank_bucket),
            }
            .instantiate()
        }

        pub fn free_token(&mut self) -> Bucket {
            let free_amount: i32 = 10;
            info!(
                "My balance is: {} VegasToken. Now giving away {} token!",
                self.casino_bank.amount(), free_amount
            );
            self.casino_bank.take(Decimal::from(free_amount))
        }

        pub fn play(&mut self, bet: Bucket) -> Bucket {

            let bet_amount = bet.amount();

            info!("You are betting {} VegasToken.", bet_amount);

            self.casino_bank.put(bet);

            let first_wheel: i32 = Self::spin_wheel();
            let second_wheel: i32 = Self::spin_wheel();
            let third_wheel: i32 = Self::spin_wheel();

            let score = Self::get_score(&first_wheel, &second_wheel, &third_wheel);

            info!(
                "You rolled {}, {}, {} and your score is {}.",
                Self::get_fruit_symbol(&first_wheel),
                Self::get_fruit_symbol(&second_wheel),
                Self::get_fruit_symbol(&third_wheel),
                score
            );

            if score == 0 {
                info!("You've lost! Good luck next time.");
                let empty_bucket: Bucket = Bucket::new(RADIX_TOKEN);
                return empty_bucket
            } else {
                let win = bet_amount * score;
                info!("You've won {} VegasToken! Congratulations!", win);
                return self.casino_bank.take(win)
            }
        }

        fn spin_wheel() -> i32 {
            let item = Uuid::generate() % NUMBER_OF_ITEMS;
            assert!(item < NUMBER_OF_ITEMS);
            dbg!("Generated number: {}", item);
            return i32::try_from(item).unwrap();
        }

        fn get_score(first_wheel: &i32, second_wheel: &i32, third_wheel: &i32) -> i32 {

            let score: i32;
            let columns = (*first_wheel, *second_wheel, *third_wheel);

            match columns {
                (BAR,    BAR,     BAR)     => score = 250,
                (BELL,   BELL,    BELL)    => score = 20,
                (PLUM,   PLUM,    PLUM)    => score = 14,
                (LEMON,  LEMON,   LEMON)   => score = 12,
                (ORANGE, ORANGE,  ORANGE)  => score = 10,
                (CHERRY, CHERRY,  CHERRY)  => score = 7,
                (CHERRY, CHERRY,  _)       => score = 5,
                (CHERRY, _,       _)       => score = 2,
                (_,      _,       _)       => score = 0,
            }
            return score;
        }

        fn get_fruit_symbol(number: &i32) -> String {
            let symbol: &str;
            match *number {
                CHERRY  => symbol = "CHERRY",
                LEMON   => symbol = "LEMON",
                ORANGE  => symbol = "ORANGE",
                PLUM    => symbol = "PLUM",
                BELL    => symbol = "BELL",
                BAR     => symbol = "BAR",
                _ => panic! ("Invalid item number: {}!", number)
            }
            return String::from(symbol)
        }

    }

}
