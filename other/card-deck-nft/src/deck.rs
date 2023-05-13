use scrypto::prelude::*;
use std::fmt::{self, Display};

#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum Suit {
    D,
    H,
    C,
    S,
}
#[derive(NonFungibleData, ScryptoSbor, Copy, Clone)]
pub struct Card {
    suit: Suit,
    value: u8,
}
#[derive(NonFungibleData, ScryptoSbor)]
pub struct Deck {
    #[mutable]
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut item: Vec<Card> = Vec::<Card>::with_capacity(52);
        item.append(&mut Deck::build_suit(Suit::D));
        item.append(&mut Deck::build_suit(Suit::H));
        item.append(&mut Deck::build_suit(Suit::C));
        item.append(&mut Deck::build_suit(Suit::S));
        Deck { cards: item }
    }

    // Create an empty Struct for player
    pub fn player_hand() -> Self {
        Deck { cards: vec![] }
    }

    fn build_suit(suit: Suit) -> Vec<Card> {
        let mut item: Vec<Card> = Vec::<Card>::with_capacity(13);
        for n in 1..(item.capacity() + 1) {
            item.push(Card {
                suit,
                value: n as u8,
            });
        }
        item
    }

    // Shuffle
    pub fn shuffle(&mut self) {
        let n_cards = self.cards.len();
        for n in 0..self.cards.len() {
            let second_random_index = self.spin_wheel() as usize % n_cards;
            self.cards.swap(n, second_random_index);
        }
    }

    // Random number generator (this is not a true random method)
    fn spin_wheel(&mut self) -> i32 {
        let item = Runtime::generate_uuid() % self.cards.len() as u128;
        return i32::try_from(item).unwrap();
    }
}

// DISPLAY PURPOSED FOR PRETTIER PRINT OUTPUT
impl Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let emoji = match self {
            Self::D => String::from("♦️"),
            Self::H => String::from("♥️"),
            Self::C => String::from("♣️"),
            Self::S => String::from("♠️"),
        };
        write!(f, "{}", emoji)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self.value {
            1 => String::from("A"),
            11 => String::from("J"),
            12 => String::from("Q"),
            13 => String::from("K"),
            other_value => other_value.to_string(),
        };
        write!(f, "{}{}", value, self.suit)
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for single_card in &self.cards {
            write!(f, " {}", single_card)?;
        }
        Ok(())
    }
}
