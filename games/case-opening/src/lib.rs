use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Skin {
    name: String,
    weapon: String,
    float: u128,
    pattern: u16,
    
}

blueprint! {
    struct Case {
        system_badge: ResourceAddress,
        system_vault: Vault,
        skin_nft: ResourceAddress,
        xrd_vault: Vault,
    }
    impl Case {
        pub fn new() -> ComponentAddress {
             





            let skin_nft = ResourceBuilder::new_non_fungible()
                .metadata("type", "Substradix character NFT")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .restrict_withdraw(AccessRule::DenyAll, MUTABLE(developer_rule.clone()))
                .updateable_non_fungible_data(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply(); 

    }
        
