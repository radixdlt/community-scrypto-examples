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
    name: String,
    exp: Decimal,
    class: Class,
    

blueprint! {
    struct Substradix {
         gold_vault: Vault,
    }

    impl Substradix {
        pub fn instantiate_nft() -> ComponentAddress {
            // Creates a set of NFTs
            let account_bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "Substradix Account NFT")
                .initial_supply([
                    (
                        NonFungibleId::from_u64(1u64),
                        Account {
                            name: Quwin,
                            class: Class::Paladin,
                            exp: 0,
                        },
                    ),
                ]);

         

