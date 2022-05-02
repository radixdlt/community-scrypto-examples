use scrypto::prelude::*;

pub mod combat;

#[derive(NonFungibleData)]
pub struct Account {
    // Classes are: 
    // #1 = Gladiator: High attack damage, medium defense, medium speed. Can be built towards Lifesteal identity (Heal from damage, stronger activates <50% HP), or Perfect Warrior identity (attacks can combo, stronger >50% HP)
    // #2 = Rogue: Hybrid attack/magic damage, High Speed. All rounder, able to be built towards both max attack (crit burst identity) or max magic (shadow magic dps identity)
    // #3 = Ranger: Ranged Attack Damage, medium attack medium speed. Single-target ranged dps identity. 
    // #4 = Mystic: Ranged Magic Damage, High magic low defense, low speed. AOE ranged burst identity.
    // #5 = Vanguard: High Health, High Defense. Can be built towards risky Counter Identity(attack burst damage) or Suffocation Identity(magic dps damage + max tanky)
    // More Classes can be added later
    #[scrypto(mutable)]
    class: u8, 
    #[scrypto(mutable)]
    exp: u128,
    #[scrypto(mutable)]
    health: i32,
    #[scrypto(mutable)]
    attack: i32,
    #[scrypto(mutable)]
    magic: i32,
    #[scrypto(mutable)]
    defense: i32,
    #[scrypto(mutable)]
    speed: i32,
}


blueprint! {
    struct Substradix {
         // Vault for holding ingame Gold
         gold_vault: Vault,
         // Vault for holding Xrd from sales
         collected_xrd: Vault,
         // Game price (can be changed)
         game_price: Decimal,
         // Account/Character ID
         character_number: u64,
         // NFT for character
         character_nft: ResourceAddress,
         // Vault to hold with badge for system actions
         system_vault: Vault,
         // Version ID to allow updates post instantiation, older NFTs can't be used in updated versions without updating the NFT. Prevents exploits.
         version:  u8,
         developer_vault: Vault,

    }

    impl Substradix {
        pub fn new(game_price: Decimal) -> (ComponentAddress, Bucket) {

            // Creates developer badge for methods. Necessary to control system_badge
            let mut developer_badge = ResourceBuilder::new_fungible()
                .metadata("name", "developer")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(10000);

            let developer_rule: AccessRule = rule!(require(developer_badge.resource_address()));

            // Creates system badge changing NFT Data. Necessary for game expansions.
            let system_badge = ResourceBuilder::new_fungible()
                .metadata("name", "system")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(developer_rule.clone(), MUTABLE(developer_rule.clone()))
                .initial_supply(1000000);

            let system_rule: AccessRule = rule!(require(system_badge.resource_address()));

            // NFT for character data
            let character_nft = ResourceBuilder::new_non_fungible()
                .metadata("type", "Substradix character NFT")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .restrict_withdraw(AccessRule::DenyAll, MUTABLE(developer_rule.clone()))
                .updateable_non_fungible_data(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply();

            // Gold for ingame currency
            let token_gold = ResourceBuilder::new_fungible()
                .metadata("name", "Gold Coin")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply();
                    

            // Creates materials for upgrading  
            let token_greavite = ResourceBuilder::new_fungible()
                .metadata("name", "Greavite Ore")
                .mintable(rule!(require(system_badge.resource_address())), MUTABLE(developer_rule.clone()))
                .burnable(rule!(require(system_badge.resource_address())), MUTABLE(developer_rule.clone()))
                .no_initial_supply();


            let token_wood = ResourceBuilder::new_fungible()
                .metadata("name", "Ageis Wood")
                .mintable(rule!(require(system_badge.resource_address())), MUTABLE(developer_rule.clone()))
                .burnable(rule!(require(system_badge.resource_address())), MUTABLE(developer_rule.clone()))
                .no_initial_supply();

            // Sets values for instantiation
            let instantiate = Self {
                system_vault: Vault::with_bucket(system_badge),
                developer_vault: Vault::with_bucket(developer_badge.take(9990)),
                gold_vault: Vault::new(token_gold),
                collected_xrd: Vault::new(RADIX_TOKEN),
                game_price,
                character_number: 0,
                character_nft,
                version: 1,

            }
            .instantiate()
            .globalize();
            (instantiate, developer_badge)
        }

        // Creates Gladiator
        pub fn create_character_class1(&mut self,
            mut payment: Bucket,
            ) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 1,
                exp: 0,
                health: 11,
                attack: 11,
                magic: 8,
                defense: 10,
                speed: 10,
            };

            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
            
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            (new_character, payment,)

        }
        // Creates Rogue
        pub fn create_character_class2(&mut self,
            mut payment: Bucket,
            ) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 2,
                exp: 0,
                health: 9,
                attack: 11,
                magic: 11,
                defense: 8,
                speed: 11,
            };

            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
            
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            (new_character, payment,)

        }

        // Owner only, collects all XRD from sold Personal Tokens
        pub fn withdraw_xrd(&mut self) -> Bucket {
            let withdraw = self.developer_vault.authorize(||self.collected_xrd.take_all());
            withdraw
        }

        // Changes price of Substradix
         pub fn change_price(&mut self, new_price: Decimal) {
            self.developer_vault.authorize(||self.game_price = new_price);
        }

        pub fn level_1(&mut self, nft_proof: Proof) -> Proof {
            assert!(
                nft_proof.amount() == dec!("1"),
                "You can only fight with one character at once!"
            );

            assert!(
                nft_proof.resource_address() == self.character_nft,
                "Wrong resource address!"
            );

            //let key_bucket: Bucket = self.system_vault.take(1);

            let _seed = Runtime::generate_uuid();
            let mut nft_data: Account = nft_proof.non_fungible().data();
            let mut health: i32 = nft_data.health;
            let mut attack: i32 = nft_data.attack;
            //let mut magic: i32 = nft_data.magic;
            let mut defense: i32 = nft_data.defense;
            let mut speed: i32 = nft_data.speed;
            let mut enemy_health: i32 = 10;
            let mut enemy_attack: i32 = 11;
            //let mut enemy_magic: i32 = 0;
            let mut enemy_defense: i32 = 3;
            let mut enemy_speed: i32 = 8;
            // Fight simulator
            // Enemy 1:
            if enemy_speed < speed {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=0 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 0 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            else if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                };
            }
            else {
                loop{
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            // Enemy 2:
            if enemy_speed < speed {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=0 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 0 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            else if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                };
            }
            else {
                loop{
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            // Enemy 3:
            if enemy_speed < speed {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=0 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 0 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            else if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                };
            }
            else {
                loop{
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <=1 {
                        self.system_vault.authorize(||nft_data.exp += 15);
                        break
        
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        self.system_vault.authorize(||nft_data.exp += 1);
                        return nft_proof
                    }
                };
            }
            return nft_proof
        }
    }
}
