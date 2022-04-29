use scrypto::prelude::*;
use sbor::*;

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Class {
    Gladiator,
    Rogue,
    Ranger,
    Mystic,
    Paladin,
    Enchanter,
}

#[derive(NonFungibleData)]
pub struct Account {
    #[scrypto(mutable)]
    exp: Decimal,
    class: Class,
    level: u8,

blueprint! {
    struct Substradix {
         gold_vault: Vault,
         

