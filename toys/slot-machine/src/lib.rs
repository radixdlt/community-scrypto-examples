use scrypto::prelude::*;

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

        pub fn new(supply: Decimal) -> ComponentAddress {
            let bank_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Vegas")
                .metadata("symbol", "VEG")
                .initial_supply(supply);

            Self {
                casino_bank: Vault::with_bucket(bank_bucket),
            }
            .instantiate().globalize()
        }

        pub fn free_token(&mut self) -> Bucket {
            let free_amount: i32 = 10;
            info!(
                "My balance is: {} VegasToken. Now giving away {} token!",
                self.casino_bank.amount(), free_amount
            );
            self.casino_bank.take(Decimal::from(free_amount))
        }

        pub fn play(&mut self, bet: Bucket) -> Option<Bucket> {

            let bet_amount = bet.amount();

            info!("You are betting {} VegasToken.", bet_amount);

            self.casino_bank.put(bet);

            let first_wheel: i32 = spin_wheel();
            let second_wheel: i32 = spin_wheel();
            let third_wheel: i32 = spin_wheel();

            let score = get_score(&first_wheel, &second_wheel, &third_wheel);

            info!(
                "You rolled {}, {}, {} and your score is {}.",
                get_fruit_symbol(&first_wheel),
                get_fruit_symbol(&second_wheel),
                get_fruit_symbol(&third_wheel),
                score
            );

            if score == 0 {
                info!("You've lost! Good luck next time.");
                return None
            } else {
                let win = bet_amount * score;
                info!("You've won {} VegasToken! Congratulations!", win);
                return Some(self.casino_bank.take(win))
            }

        }

    }

}

fn spin_wheel() -> i32 {
    let item = Runtime::generate_uuid() % NUMBER_OF_ITEMS;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_bars() {
        let score = get_score(&BAR, &BAR, &BAR);
        assert!(score == 250,
            "Exprected score for three BARS is 250. Actual score: {}", score)
    }

    #[test]
    fn test_one_cherry() {
        let score = get_score(&CHERRY, &BAR, &BAR);
        assert_eq!( score, 2,
            "Exprected score for three CHERRIES is 2. Actual score: {}", score)
    }

    #[test]
    fn test_two_cherries() {
        let score = get_score(&CHERRY, &CHERRY, &BAR);
        assert_eq!( score, 5,
            "Exprected score for three CHERRIES is 5. Actual score: {}", score)
    }

    #[test]
    fn test_cherry_symbol() {
        let symbol = get_fruit_symbol(&CHERRY);
        assert_eq!( symbol, String::from("CHERRY"),
            "Expected symbol CHERRY. Actual symbol: {}", symbol)
    }

    #[test]
    #[ignore] // test panicks with 'not yet implemented'
    fn test_spin_wheel() {
        let result = spin_wheel();
        let max = i32::try_from(NUMBER_OF_ITEMS).unwrap();
        let range = 0..max;
        assert!(range.contains(&result));
    }
}
