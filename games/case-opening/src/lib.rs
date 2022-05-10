use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Skin {
    name: String,
    weapon: String,
    wear: String,
    float: u128,
    pattern: u16,
    stat_track: bool,
    
}

blueprint! {
    struct Case {
        system_vault: Vault,
        skin_nft: ResourceAddress,
        collected_xrd: Vault,
        developer_vault: Vault,
        key: ResourceAddress,
        key_price: u8,
    }
    impl Case {
        pub fn new() -> (ComponentAddress, Bucket) {

 // Creates developer badge for methods. Necessary to control system_badge
            let mut developer_badge = ResourceBuilder::new_fungible()
                .metadata("name", "developer")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(10000);

            let developer_rule: AccessRule = rule!(require(developer_badge.resource_address()));


            let system_badge = ResourceBuilder::new_fungible()
                .metadata("name", "system")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(developer_rule.clone(), MUTABLE(developer_rule.clone()))
                .initial_supply(1000000);

            let system_rule: AccessRule = rule!(require(system_badge.resource_address()));

            let skin_nft = ResourceBuilder::new_non_fungible()
                .metadata("name", "Weapon Skin NFT")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .updateable_non_fungible_data(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply();

            let key = ResourceBuilder::new_fungible()
                .metadata("name", "Case Key")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply();

            let instantiate = Self {
                system_vault: Vault::with_bucket(system_badge),
                developer_vault: Vault::with_bucket(developer_badge.take(9990)),
                skin_nft,
                collected_xrd: Vault::new(RADIX_TOKEN),
                key,
                key_price: 25,

            }
            .instantiate()
            .globalize();
            (instantiate, developer_badge)

        }

        pub fn buy_keys(&mut self, mut payment: Bucket, amount: u32) -> (Bucket, Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);
            let cost = amount * 25;
            let xrd = payment.take(cost);
            self.collected_xrd.put(xrd);
            let new_keys = self.system_vault.authorize(||
                borrow_resource_manager!(self.key)
                    .mint(amount));
            self.system_vault.put(key_bucket);
            (new_keys, payment)
        }

        pub fn open_case(&mut self, mut key: Bucket) -> (Bucket, Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);
            let uuid: Runtime::generate_uuid();
            
            let xrd = payment.take(cost);
            self.collected_xrd.put(xrd);
            let new_keys = self.system_vault.authorize(||
                borrow_resource_manager!(self.key)
                    .mint(amount));
            self.system_vault.put(key_bucket);
            (new_keys, payment)
        }

            

}
        
