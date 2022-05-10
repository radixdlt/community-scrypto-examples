use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Skin {
    name: String,
    weapon: String,
    float: u128,
    pattern: u16,
    
}

blueprint! {
    struct Substradix {
