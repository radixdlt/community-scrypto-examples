use scrypto::prelude::*;

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
    level: u32, 
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
    #[scrypto(mutable)]
    version: u8,
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
        pub fn create_character_class1(&mut self, mut payment: Bucket) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 1,
                level: 1,
                exp: 0,
                health: 11,
                attack: 11,
                magic: 8,
                defense: 10,
                speed: 10,
                version: 1,
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
        pub fn create_character_class2(&mut self, mut payment: Bucket) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 2,
                level: 1,
                exp: 0,
                health: 9,
                attack: 10,
                magic: 10,
                defense: 9,
                speed: 12,
                version: 1,
            };

            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
            
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            (new_character, payment,)
        }
        // Creates Ranger
        pub fn create_character_class3(&mut self, mut payment: Bucket) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 3,
                level: 1,
                exp: 0,
                health: 9,
                attack: 12,
                magic: 8,
                defense: 9,
                speed: 12,
                version: 1,
            };

            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
            
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            (new_character, payment,)
        }
        // Creates Mystic
        pub fn create_character_class4(&mut self, mut payment: Bucket) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 4,
                level: 1,
                exp: 0,
                health: 9,
                attack: 8,
                magic: 15,
                defense: 9,
                speed: 9,
                version: 1,
            };

            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
            
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            (new_character, payment,)
        }
        // Creates Vanguard
        pub fn create_character_class5(&mut self, mut payment: Bucket) -> (Bucket, Bucket, ) {
            let key_bucket: Bucket = self.system_vault.take(1);

            let character_data = Account {  
                class: 5,
                level: 1,
                exp: 0,
                health: 12,
                attack: 9,
                magic: 9,
                defense: 12,
                speed: 8,
                version: 1,
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

        // Level up EXP is straight up copied from MapleStory lmao
        pub fn level_up_class1(&mut self, nft_proof: Proof) {
            assert!(
                nft_proof.amount() == dec!("1"),
                "You can only level up one character at once!"
            );
        
            assert!(
                nft_proof.resource_address() == self.character_nft,
                "Wrong resource address!"
            );
            let mut nft_data: Account = nft_proof.non_fungible().data();
            assert!(
                nft_data.class == 1,
                "Wrong class!"
            );
            let total_exp = nft_data.exp;
            // Idk if this is good, the level scaling seems wack af lmao
            match total_exp {
                0..=14 =>  { return },
                15..=48 => { 
                    if nft_data.level < 2 {
                    nft_data.level = 2;
                    // + 3 stats
                    nft_data.health = 13;
                    nft_data.attack = 12;
                    nft_data.magic = 8;
                    nft_data.defense = 10;
                    nft_data.speed = 10;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                    }
                    else {
                        return
                    }
                },
                49..=105 => { 
                    if nft_data.level < 3 {
                    nft_data.level = 3;
                    // + 3 stats
                    nft_data.health = 14;
                    nft_data.attack = 13;
                    nft_data.magic = 8;
                    nft_data.defense = 10;
                    nft_data.speed = 11;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                106..=197 => { 
                    if nft_data.level < 4 {
                    nft_data.level = 4;
                    // + 3 stats
                    nft_data.health = 14;
                    nft_data.attack = 14;
                    nft_data.magic = 8;
                    nft_data.defense = 11;
                    nft_data.speed = 12;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                198..=332 => { 
                    if nft_data.level < 5 {
                    nft_data.level = 5;
                    // + 4 stats
                    nft_data.health = 15;
                    nft_data.attack = 16;
                    nft_data.magic = 8;
                    nft_data.defense = 12;
                    nft_data.speed = 12;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                333..=704 => { 
                    if nft_data.level < 6 {
                    nft_data.level = 6;
                    // + 5 stats
                    nft_data.health = 16;
                    nft_data.attack = 17;
                    nft_data.magic = 9;
                    nft_data.defense = 13;
                    nft_data.speed = 13;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                705..=1264 => { 
                    if nft_data.level < 7 {
                    nft_data.level = 7;
                    // + 5 stats
                    nft_data.health = 18;
                    nft_data.attack = 20;
                    nft_data.magic = 9;
                    nft_data.defense = 13;
                    nft_data.speed = 13;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                1265..=2104 => { 
                    if nft_data.level < 8 {
                    nft_data.level = 8;
                    // + 5 stats
                    nft_data.health = 18;
                    nft_data.attack = 22;
                    nft_data.magic = 9;
                    nft_data.defense = 14;
                    nft_data.speed = 15;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                2105..=3346 => { 
                    if nft_data.level < 9 {
                    nft_data.level = 9;
                    // + 5 stats
                    nft_data.health = 20;
                    nft_data.attack = 23;
                    nft_data.magic = 10;
                    nft_data.defense = 15;
                    nft_data.speed = 15;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                3347..=4588 => { 
                    if nft_data.level < 10 {
                    nft_data.level = 10;
                    // + 6 stats
                    nft_data.health = 22;
                    nft_data.attack = 25;
                    nft_data.magic = 9;
                    nft_data.defense = 14;
                    nft_data.speed = 17;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                4589..=5830 => { 
                    if nft_data.level < 11 {
                    nft_data.level = 11;
                    // + 7 stats
                    nft_data.health = 24;
                    nft_data.attack = 25;
                    nft_data.magic = 9;
                    nft_data.defense = 16;
                    nft_data.speed = 20;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                5831..=7072 => { 
                    if nft_data.level < 12 {
                    nft_data.level = 12;
                    // + 7 stats
                    nft_data.health = 24;
                    nft_data.attack = 27;
                    nft_data.magic = 11;
                    nft_data.defense = 18;
                    nft_data.speed = 21;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                7073..=8314 => { 
                    if nft_data.level < 13 {
                    nft_data.level = 13;
                    // + 7 stats
                    nft_data.health = 25;
                    nft_data.attack = 30;
                    nft_data.magic = 11;
                    nft_data.defense = 20;
                    nft_data.speed = 22;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                8315..=9556 => { 
                    if nft_data.level < 14 {
                    nft_data.level = 14;
                    // + 7 stats
                    nft_data.health = 28;
                    nft_data.attack = 34;
                    nft_data.magic = 11;
                    nft_data.defense = 20;
                    nft_data.speed = 22;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                9557..=11046 => { 
                    if nft_data.level < 15 {
                    nft_data.level = 15;
                    // + 8 stats
                    nft_data.health = 32;
                    nft_data.attack = 36;
                    nft_data.magic = 11;
                    nft_data.defense = 21;
                    nft_data.speed = 23;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                11047..=12834 => { 
                    if nft_data.level < 16 {
                    nft_data.level = 16;
                    // + 9 stats
                    nft_data.health = 35;
                    nft_data.attack = 38;
                    nft_data.magic = 13;
                    nft_data.defense = 23;
                    nft_data.speed = 23;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                12835..=14979 => { 
                    if nft_data.level < 17 {
                    nft_data.level = 17;
                    // + 9 stats
                    nft_data.health = 35;
                    nft_data.attack = 38;
                    nft_data.magic = 15;
                    nft_data.defense = 26;
                    nft_data.speed = 27;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                14980..=17553 => { 
                    if nft_data.level < 18 {
                    nft_data.level = 18;
                    // + 9 stats
                    nft_data.health = 36;
                    nft_data.attack = 41;
                    nft_data.magic = 15;
                    nft_data.defense = 29;
                    nft_data.speed = 29;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                17554..=20641 => { 
                    if nft_data.level < 19 {
                    nft_data.level = 19;
                    // + 9 stats
                    nft_data.health = 39;
                    nft_data.attack = 44;
                    nft_data.magic = 15;
                    nft_data.defense = 30;
                    nft_data.speed = 31;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                20642..=1000000 => { 
                    if nft_data.level < 20 {
                    nft_data.level = 20;
                    // + 10 stats
                    nft_data.health = 41;
                    nft_data.attack = 47;
                    nft_data.magic = 15;
                    nft_data.defense = 32;
                    nft_data.speed = 34;
                    self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                _ =>  { return }
            };
        }
        pub fn level_1(&mut self, nft_proof: Proof) {
            assert!(
                nft_proof.amount() == dec!("1"),
                "You can only fight with one character at once!"
            );
        
            assert!(
                nft_proof.resource_address() == self.character_nft,
                "Wrong resource address!"
            );
        
            let key_bucket: Bucket = self.system_vault.take(1);
        
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
            if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
        
                    }
                };
            }
            else {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                };
            }
            // Enemy 2:
            if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
        
                    }
                };
            }
            else {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                };
            }
            // Enemy 3:
            if speed < enemy_speed {
                loop{
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
        
                    }
                };
            }
            else {
                loop {
                    enemy_health -= attack - enemy_defense;
                    if enemy_health <= 1 {
                        nft_data.exp += 5;
                        break
                    }
                    health -= enemy_attack - defense;
                    if health <= 1 {
                        nft_data.exp += 1;
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                        self.level_up_class1(nft_proof);
                        self.system_vault.put(key_bucket);
                        return
                    }
                };
            }
            self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
            self.level_up_class1(nft_proof);
            self.system_vault.put(key_bucket);
            return
        }
    }
}
