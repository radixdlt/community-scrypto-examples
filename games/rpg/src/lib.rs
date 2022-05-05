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
    class: Decimal,
    #[scrypto(mutable)]
    level: Decimal,
    #[scrypto(mutable)]
    exp: u128,
    #[scrypto(mutable)]
    stage: Decimal, 
    #[scrypto(mutable)]
    health: Decimal,
    #[scrypto(mutable)]
    attack: Decimal,
    #[scrypto(mutable)]
    magic: Decimal,
    #[scrypto(mutable)]
    defense: Decimal,
    #[scrypto(mutable)]
    speed: Decimal,
    #[scrypto(mutable)]
    version: Decimal,
}

#[derive(NonFungibleData)]
pub struct Weapon {
    #[scrypto(mutable)]
    class: Decimal,
    #[scrypto(mutable)]
    id: Decimal,
    #[scrypto(mutable)]
    level: Decimal, 
    #[scrypto(mutable)]
    physical_base: Decimal,
    #[scrypto(mutable)]
    physical_scaling: Decimal,
    #[scrypto(mutable)]
    spell_base: Decimal,
    #[scrypto(mutable)]
    spell_scaling: Decimal,
    #[scrypto(mutable)]
    ability: Decimal,
    #[scrypto(mutable)]
    ability_scaling: Decimal,
    #[scrypto(mutable)]
    range: Decimal,
    #[scrypto(mutable)]
    version: Decimal,
}

#[derive(NonFungibleData)]
pub struct Armor {
    #[scrypto(mutable)]
    id: Decimal,
    #[scrypto(mutable)]
    part: String,
    #[scrypto(mutable)]
    level: Decimal, 
    #[scrypto(mutable)]
    health: Decimal,
    #[scrypto(mutable)]
    defense: Decimal,
    #[scrypto(mutable)]
    weight: Decimal,
    #[scrypto(mutable)]
    ability: Decimal,
    #[scrypto(mutable)]
    ability_scaling: Decimal,
    #[scrypto(mutable)]
    version: Decimal,
}

#[derive(NonFungibleData)]
pub struct Accessory {
    #[scrypto(mutable)]
    id: Decimal,
    #[scrypto(mutable)]
    part: String,
    #[scrypto(mutable)]
    level: Decimal, 
    #[scrypto(mutable)]
    attack: Decimal,
    #[scrypto(mutable)]
    magic: Decimal,
    #[scrypto(mutable)]
    speed: Decimal,
    #[scrypto(mutable)]
    weight: Decimal,
    #[scrypto(mutable)]
    ability: Decimal,
    #[scrypto(mutable)]
    ability_scaling: Decimal,
    #[scrypto(mutable)]
    version: Decimal,
}

blueprint! {
    struct Substradix {
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
         version:  Decimal,
         developer_vault: Vault,
         // Weapon/Armor NFT
         item_nft: ResourceAddress,
         token_greavite: ResourceAddress,
         token_wood: ResourceAddress,
         token_gold: ResourceAddress,

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
            // Item NFT
            let item_nft = ResourceBuilder::new_non_fungible()
                .metadata("type", "Substradix weapon NFT")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .restrict_withdraw(AccessRule::AllowAll, MUTABLE(developer_rule.clone()))
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
                collected_xrd: Vault::new(RADIX_TOKEN),
                game_price,
                character_number: 0,
                character_nft,
                item_nft,
                token_greavite,
                token_wood,
                token_gold,
                version: dec!(1),

            }
            .instantiate()
            .globalize();
            (instantiate, developer_badge)
        }
        pub fn mint_mats(&mut self) -> (Bucket, Bucket, Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);
            let gold = self.system_vault.authorize(||
                borrow_resource_manager!(self.token_gold)
                    .mint(1000));
            let wood = self.system_vault.authorize(||
                borrow_resource_manager!(self.token_wood)
                    .mint(1000));
            let greavite = self.system_vault.authorize(||
                borrow_resource_manager!(self.token_greavite)
                    .mint(1000));
            self.system_vault.put(key_bucket);
            (gold, wood, greavite)
        }
        // Creates character
        pub fn create_character(&mut self, mut payment: Bucket, _chosen_class: u8) -> (Bucket, Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);

            for chosen_class in 1..=5 {
                if chosen_class == 1 {
                    let character_data = Account {  
                        class: dec!(1),
                        level: dec!(1),
                        exp: 0,
                        stage: dec!(1),
                        health: dec!(11),
                        attack: dec!(11),
                        magic: dec!(8),
                        defense: dec!(10),
                        speed: dec!(10),
                        version: dec!(1),
                    };
                    let new_character = self.system_vault.authorize(||
                        borrow_resource_manager!(self.character_nft)
                            .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                            
                    self.character_number += 1;
                    self.collected_xrd.put(payment.take(self.game_price));
                    self.system_vault.put(key_bucket);
                    return (new_character, payment,)
                }
            
                else if chosen_class == 2 {
                    let character_data = Account {  
                        class: dec!(2),
                        level: dec!(1),
                        exp: 0,
                        stage: dec!(1),
                        health: dec!(9),
                        attack: dec!(10),
                        magic: dec!(10),
                        defense: dec!(9),
                        speed: dec!(12),
                        version: dec!(1),
                    };
                    let new_character = self.system_vault.authorize(||
                        borrow_resource_manager!(self.character_nft)
                            .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                            
                    self.character_number += 1;
                    self.collected_xrd.put(payment.take(self.game_price));
                    self.system_vault.put(key_bucket);
                    return (new_character, payment,)
                }
                else if chosen_class == 3 {
                    let character_data = Account {  
                        class: dec!(3),
                        level: dec!(1),
                        exp: 0,
                        stage: dec!(1),
                        health: dec!(10),
                        attack: dec!(10),
                        magic: dec!(10),
                        defense: dec!(10),
                        speed: dec!(10),
                        version: dec!(1),
                    };
                    let new_character = self.system_vault.authorize(||
                        borrow_resource_manager!(self.character_nft)
                            .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                            
                    self.character_number += 1;
                    self.collected_xrd.put(payment.take(self.game_price));
                    self.system_vault.put(key_bucket);
                    return (new_character, payment,)
                }
                else if chosen_class == 4 {
                    let character_data = Account {  
                        class: dec!(4),
                        level: dec!(1),
                        exp: 0,
                        stage: dec!(1),
                        health: dec!(9),
                        attack: dec!(8),
                        magic: dec!(15),
                        defense: dec!(9),
                        speed: dec!(9),
                        version: dec!(1),
                    };
                    let new_character = self.system_vault.authorize(||
                        borrow_resource_manager!(self.character_nft)
                            .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                            
                    self.character_number += 1;
                    self.collected_xrd.put(payment.take(self.game_price));
                    self.system_vault.put(key_bucket);
                    return (new_character, payment,)
                }
                else {
                    let character_data = Account {  
                        class: dec!(5),
                        level: dec!(1),
                        exp: 0,
                        stage: dec!(1),
                        health: dec!(12),
                        attack: dec!(9),
                        magic: dec!(9),
                        defense: dec!(12),
                        speed: dec!(8),
                        version: dec!(1),
                    };
                    let new_character = self.system_vault.authorize(||
                        borrow_resource_manager!(self.character_nft)
                            .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                            
                    self.character_number += 1;
                    self.collected_xrd.put(payment.take(self.game_price));
                    self.system_vault.put(key_bucket);
                    return (new_character, payment,)
                };
            };
            let character_data = Account {  
                class: dec!(1),
                level: dec!(1),
                exp: 0,
                stage: dec!(1),
                health: dec!(11),
                attack: dec!(11),
                magic: dec!(8),
                defense: dec!(10),
                speed: dec!(10),
                version: dec!(1),
            };
            let new_character = self.system_vault.authorize(||
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                    
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            return (new_character, payment,)
        }
        pub fn seed_50(&mut self) -> Decimal {
            let mut digits = Vec::new();
            let mut seed = Runtime::generate_uuid();
            while seed > 9 {
                digits.push(seed % 10);
                seed = seed / 10
            }
            digits.push(seed);
            digits.reverse();
            let point = digits.get(1);
            let anti_option: u128 = match point {
                Some(0) => 0,
                Some(1) => 1,
                Some(2) => 2,
                Some(3) => 3,
                Some(4) => 4,
                Some(5) => 5,
                Some(6) => 6,
                Some(7) => 7,
                Some(8) => 8,
                Some(9) => 9,
                _ => 0
            };
            let point2 = digits.get(2);
            let anti_option2: u128 = match point2 {
                Some(0) => 0,
                Some(1) => 1,
                Some(2) => 2,
                Some(3) => 3,
                Some(4) => 4,
                Some(5) => 5,
                Some(6) => 6,
                Some(7) => 7,
                Some(8) => 8,
                Some(9) => 9,
                _ => 0
            };
            let random: u128 = format!("{}{}", anti_option, anti_option2).parse().unwrap();
            match random {
                00..=01  => dec!(75) / dec!(100),
                02..=03  => dec!(76) / dec!(100),
                04..=05  => dec!(77) / dec!(100),
                06..=07  => dec!(78) / dec!(100),
                08..=09  => dec!(79) / dec!(100),
                10..=11  => dec!(80) / dec!(100),
                12..=13  => dec!(81) / dec!(100),
                14..=15  => dec!(82) / dec!(100),
                16..=17  => dec!(83) / dec!(100),
                18..=19  => dec!(84) / dec!(100),
                20..=21  => dec!(85) / dec!(100),
                22..=23  => dec!(86) / dec!(100),
                24..=25  => dec!(87) / dec!(100),
                26..=27  => dec!(88) / dec!(100),
                28..=29  => dec!(89) / dec!(100),
                30..=31  => dec!(90) / dec!(100),
                32..=33  => dec!(91) / dec!(100),
                34..=35  => dec!(92) / dec!(100),
                36..=37  => dec!(93) / dec!(100),
                38..=39  => dec!(94) / dec!(100),
                40..=41  => dec!(95) / dec!(100),
                42..=43  => dec!(96) / dec!(100),
                44..=45  => dec!(97) / dec!(100),
                46..=47  => dec!(98) / dec!(100),
                48..=49  => dec!(99) / dec!(100),
                50..=51  => dec!(1),
                52..=53  => dec!(101) / dec!(100),
                54..=55  => dec!(102) / dec!(100),
                56..=57  => dec!(103) / dec!(100),
                58..=59  => dec!(104) / dec!(100),
                60..=61  => dec!(105) / dec!(100),
                62..=63  => dec!(106) / dec!(100),
                64..=65  => dec!(107) / dec!(100),
                66..=67  => dec!(108) / dec!(100),
                68..=69  => dec!(109) / dec!(100),
                70..=71  => dec!(110) / dec!(100),
                72..=73  => dec!(111) / dec!(100),
                74..=75  => dec!(112) / dec!(100),
                76..=77  => dec!(113) / dec!(100),
                78..=79  => dec!(114) / dec!(100),
                80..=81  => dec!(115) / dec!(100),
                82..=83  => dec!(116) / dec!(100),
                84..=85  => dec!(117) / dec!(100),
                86..=87  => dec!(118) / dec!(100),
                88..=89  => dec!(119) / dec!(100),
                90..=91  => dec!(120) / dec!(100),
                92..=93  => dec!(121) / dec!(100),
                94..=95  => dec!(122) / dec!(100),
                96..=97  => dec!(123) / dec!(100),
                98..=99  => dec!(124) / dec!(100),
                _ => dec!(99) / dec!(100),
            }
        }
        // Generates a Uuid, turns it into a Vector, and takes the value of a cell of that vector. Returns as u128 
        pub fn seed_10(&mut self) -> Decimal {
            let mut digits = Vec::new();
            let mut seed = Runtime::generate_uuid();
            while seed > 9 {
                digits.push(seed % 10);
                seed = seed / 10
            }
            digits.push(seed);
            digits.reverse();
            let point = digits.get(1);
            let anti_option: u128 = match point {
                Some(0) => 0,
                Some(1) => 1,
                Some(2) => 2,
                Some(3) => 3,
                Some(4) => 4,
                Some(5) => 5,
                Some(6) => 6,
                Some(7) => 7,
                Some(8) => 8,
                Some(9) => 9,
                _ => 0
            };
            match anti_option {
                0 => dec!(0),
                1 => dec!(1),
                2 => dec!(2),
                3 => dec!(3),
                4 => dec!(4),
                5 => dec!(5),
                6 => dec!(6),
                7 => dec!(7),
                8 => dec!(8),
                9 => dec!(9),
                _ => dec!(1),
            }
        }
        // Creates weapons
        pub fn create_sword_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let phys_base = dec!(25) * self.seed_50();
            let phys_scale = dec!(110) * self.seed_50();
            let mag_base = dec!(0) * self.seed_50();
            let mag_scale = dec!(0) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: dec!(1),
                id: dec!(1),
                level: dec!(1), 
                physical_base: Decimal::ceiling(&phys_base),
                physical_scaling: Decimal::ceiling(&phys_scale),
                spell_base: Decimal::ceiling(&mag_base),
                spell_scaling: Decimal::ceiling(&mag_scale),
                ability: dec!(0),
                ability_scaling: Decimal::ceiling(&ability_scale),
                range: dec!(1),
                version: dec!(1),
            };
            let new_weapon = self.system_vault.authorize(||
                borrow_resource_manager!(self.item_nft)
                    .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            
            self.system_vault.authorize(|| 
                gold.burn());
            self.system_vault.authorize(||
                resource1.burn());
            self.system_vault.authorize(||
                resource2.burn());
            self.system_vault.put(key_bucket);
            new_weapon
        }
        pub fn create_dagger_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let phys_base = dec!(15) * self.seed_50();
            let phys_scale = dec!(50) * self.seed_50();
            let mag_base = dec!(15) * self.seed_50();
            let mag_scale = dec!(50) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: dec!(2),
                id: dec!(1),
                level: dec!(1), 
                physical_base: Decimal::ceiling(&phys_base),
                physical_scaling: Decimal::ceiling(&phys_scale),
                spell_base: Decimal::ceiling(&mag_base),
                spell_scaling: Decimal::ceiling(&mag_scale),
                ability: dec!(0),
                ability_scaling: Decimal::ceiling(&ability_scale),
                range: dec!(1),
                version: dec!(1),
            };
            let new_weapon = self.system_vault.authorize(||
                borrow_resource_manager!(self.item_nft)
                    .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            
            self.system_vault.authorize(|| 
                gold.burn());
            self.system_vault.authorize(||
                resource1.burn());
            self.system_vault.authorize(||
                resource2.burn());
            self.system_vault.put(key_bucket);
            new_weapon
        }
        pub fn create_bow_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let phys_base = dec!(15) * self.seed_50();
            let phys_scale = dec!(40) * self.seed_50();
            let mag_base = dec!(15) * self.seed_50();
            let mag_scale = dec!(40) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: dec!(3),
                id: dec!(1),
                level: dec!(1), 
                physical_base: Decimal::ceiling(&phys_base),
                physical_scaling: Decimal::ceiling(&phys_scale),
                spell_base: Decimal::ceiling(&mag_base),
                spell_scaling: Decimal::ceiling(&mag_scale),
                ability: dec!(0),
                ability_scaling: Decimal::ceiling(&ability_scale),
                range: dec!(2),
                version: dec!(1),
            };
            let new_weapon = self.system_vault.authorize(||
                borrow_resource_manager!(self.item_nft)
                    .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            
            self.system_vault.authorize(|| 
                gold.burn());
            self.system_vault.authorize(||
                resource1.burn());
            self.system_vault.authorize(||
                resource2.burn());
            self.system_vault.put(key_bucket);
            new_weapon
        }
        pub fn create_staff_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let phys_base = dec!(0) * self.seed_50();
            let phys_scale = dec!(0) * self.seed_50();
            let mag_base = dec!(20) * self.seed_50();
            let mag_scale = dec!(120) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: dec!(4),
                id: dec!(1),
                level: dec!(1), 
                physical_base: Decimal::ceiling(&phys_base),
                physical_scaling: Decimal::ceiling(&phys_scale),
                spell_base: Decimal::ceiling(&mag_base),
                spell_scaling: Decimal::ceiling(&mag_scale),
                ability: dec!(0),
                ability_scaling: Decimal::ceiling(&ability_scale),
                range: dec!(2),
                version: dec!(1),
            };
            let new_weapon = self.system_vault.authorize(||
                borrow_resource_manager!(self.item_nft)
                    .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            
            self.system_vault.authorize(|| 
                gold.burn());
            self.system_vault.authorize(||
                resource1.burn());
            self.system_vault.authorize(||
                resource2.burn());
            self.system_vault.put(key_bucket);
            new_weapon
        }
        pub fn create_shield_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let phys_base = dec!(20) * self.seed_50();
            let phys_scale = dec!(20) * self.seed_50();
            let mag_base = dec!(20) * self.seed_50();
            let mag_scale = dec!(20) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: dec!(5),
                id: dec!(1),
                level: dec!(1), 
                physical_base: Decimal::ceiling(&phys_base),
                physical_scaling: Decimal::ceiling(&phys_scale),
                spell_base: Decimal::ceiling(&mag_base),
                spell_scaling: Decimal::ceiling(&mag_scale),
                ability: dec!(0),
                ability_scaling: Decimal::ceiling(&ability_scale),
                range: dec!(1),
                version: dec!(1),
            };
            let new_weapon = self.system_vault.authorize(||
                borrow_resource_manager!(self.item_nft)
                    .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            
            self.system_vault.authorize(|| 
                gold.burn());
            self.system_vault.authorize(||
                resource1.burn());
            self.system_vault.authorize(||
                resource2.burn());
            self.system_vault.put(key_bucket);
            new_weapon
        }
        pub fn create_chest_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let hp = dec!(5) * self.seed_50();
            let def = dec!(5) * self.seed_50();
            let hp_bonus = dec!(10) * self.seed_50();
            let def_bonus = dec!(10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            if self.seed_10() < dec!(5) {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Chest"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp_bonus),
                    defense: Decimal::ceiling(&def),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Chest"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp),
                    defense: Decimal::ceiling(&def_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
        }

        pub fn create_pants_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let hp = dec!(5) * self.seed_50();
            let def = dec!(5) * self.seed_50();
            let hp_bonus = dec!(10) * self.seed_50();
            let def_bonus = dec!(10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            if self.seed_10() < dec!(5) {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Pants"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp_bonus),
                    defense: Decimal::ceiling(&def),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Pants"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp),
                    defense: Decimal::ceiling(&def_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
        }
        pub fn create_helmet_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let hp = dec!(5) * self.seed_50();
            let def = dec!(5) * self.seed_50();
            let hp_bonus = dec!(10) * self.seed_50();
            let def_bonus = dec!(10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            if self.seed_10() < dec!(5) {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Helmet"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp_bonus),
                    defense: Decimal::ceiling(&def),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: String::from("Helmet"),
                    level: dec!(1), 
                    health: Decimal::ceiling(&hp),
                    defense: Decimal::ceiling(&def_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            
        }
        pub fn create_gloves_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let atk = dec!(5) * self.seed_50();
            let mag = dec!(5) * self.seed_50();
            let spd = dec!(5) * self.seed_50();
            let atk_bonus = dec!(10) * self.seed_50();
            let mag_bonus = dec!(10) * self.seed_50();
            let spd_bonus = dec!(10) * self.seed_50();
            let load_bonus = dec!(-10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let rng = self.seed_50();
            if rng < dec!(25) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Gloves"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk_bonus),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(24) && rng < dec!(50) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Gloves"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag_bonus),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(49) && rng < dec!(75) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Gloves"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Gloves"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load_bonus),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
        }
        pub fn create_belt_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let atk = dec!(5) * self.seed_50();
            let mag = dec!(5) * self.seed_50();
            let spd = dec!(5) * self.seed_50();
            let atk_bonus = dec!(10) * self.seed_50();
            let mag_bonus = dec!(10) * self.seed_50();
            let spd_bonus = dec!(10) * self.seed_50();
            let load_bonus = dec!(-10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let rng = self.seed_50();
            if rng < dec!(25) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Belt"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk_bonus),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(24) && rng < dec!(50) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Belt"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag_bonus),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(49) && rng < dec!(75) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Belt"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Belt"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load_bonus),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
        }
        pub fn create_shoes_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let atk = dec!(5) * self.seed_50();
            let mag = dec!(5) * self.seed_50();
            let spd = dec!(5) * self.seed_50();
            let atk_bonus = dec!(10) * self.seed_50();
            let mag_bonus = dec!(10) * self.seed_50();
            let spd_bonus = dec!(10) * self.seed_50();
            let load_bonus = dec!(-10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let rng = self.seed_50();
            if rng < dec!(25) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Shoes"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk_bonus),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(24) && rng < dec!(50) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Shoes"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag_bonus),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else if rng > dec!(49) && rng < dec!(75) {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Shoes"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
            else {
                let armor_data = Accessory {  
                    id: dec!(1),
                    part: String::from("Shoes"),
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load_bonus),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_armor = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), armor_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_armor
            }
        }
        // Calculates the combat between user and enemies. Enemies Data + rewards set by caller. Boolean, Boolean
        // TODO: make it so damage cannot go under 1, Shorten method using Proof copies.
        pub fn combat(
            &mut self, 
            nft_proof: Proof, 
            weapon: Proof, 
            helmet: Proof, 
            chest: Proof, 
            pants: Proof, 
            gloves: Proof, 
            belt: Proof, 
            shoes: Proof, 
            mut enemy_health: Decimal, 
            mut enemy_damage: Decimal, 
            mut enemy_defense: Decimal, 
            mut enemy_speed: Decimal,
            mut health: Decimal,
            ) -> Decimal {

            let mut nft_data: Account = nft_proof.non_fungible().data();
            let weapon_data: Weapon = weapon.non_fungible().data();
            let helmet_data: Armor = helmet.non_fungible().data();
            let chest_data: Armor = chest.non_fungible().data();
            let pants_data: Armor = pants.non_fungible().data();
            let gloves_data: Accessory = gloves.non_fungible().data();
            let belt_data: Accessory = belt.non_fungible().data();
            let shoes_data: Accessory = shoes.non_fungible().data();
            let mut speed = 
                (nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) -  
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * helmet_data.weight / dec!(100)) - 
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * chest_data.weight / dec!(100)) - 
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * pants_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * gloves_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * belt_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * shoes_data.weight / dec!(100));
            let mut speed_ratio = speed / enemy_speed;
            let mut defense = nft_data.defense + helmet_data.defense + chest_data.defense + pants_data.defense;
            let attack = {
                nft_data.attack + 
                (nft_data.attack * gloves_data.attack / dec!(100)) -
                (nft_data.attack * belt_data.attack / dec!(100)) -
                (nft_data.attack * shoes_data.attack / dec!(100))
            };
            let magic = {
                nft_data.magic + 
                (nft_data.magic * gloves_data.magic / dec!(100)) -
                (nft_data.magic * belt_data.magic/ dec!(100)) -
                (nft_data.magic * shoes_data.magic / dec!(100))
            };
            let mut damage: Decimal = 
                (weapon_data.physical_base + (weapon_data.physical_scaling / dec!(100) * attack)) +
                (weapon_data.spell_base + (weapon_data.spell_scaling / dec!(100) * magic));

            loop {
                let mut priority = speed_ratio * dec!(5)/(self.seed_10() + dec!(1));
                if health < dec!(0) || health == dec!(0) {
                    return health
                }
                if enemy_health < dec!(0) || enemy_health == dec!(0) {
                    return health
                }
                if priority > dec!(1) {
                    enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                    priority *= dec!(50)/dec!(100);
                    if priority < dec!(1) {
                        health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                        continue;
                    }
                    else {
                        enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                        priority *= dec!(50)/dec!(100);
                        if priority < dec!(1) {
                            health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                            continue;
                        }
                        else {
                            enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                            priority *= dec!(50)/dec!(100);
                            if priority < dec!(1) {
                                health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                continue;
                            }
                            else {
                                enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                priority *= dec!(50)/dec!(100);
                                if priority < dec!(1) {
                                    health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                    continue;
                                }
                                else {
                                    enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                    priority *= dec!(50)/dec!(100);
                                    if priority < dec!(1) {
                                        health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                        continue;
                                    }
                                    else {
                                        enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                        priority *= dec!(50)/dec!(100);
                                        if priority < dec!(1) {
                                            health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                            continue;
                                        }
                                        else {
                                            health -= enemy_damage * self.seed_50();
                                            enemy_health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                            continue;
                                        }
                                        // If u have more than 32* speed of your opponent and they're not dead yet fuck you
                                    }
                                }
                            }
                        }
                    }
                }
                if priority < dec!(1) {
                    health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                    priority *= dec!(2);
                    if priority > dec!(1) {
                        enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                        continue;
                    }
                    else {
                        health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                        priority *= dec!(2);
                        if priority > dec!(1) {
                            enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                            continue;
                        }
                        else {
                            health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                            priority *= dec!(2);
                            if priority > dec!(1) {
                                enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                continue;
                            }
                            else {
                                health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                priority *= dec!(2);
                                if priority > dec!(1) {
                                    enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                    continue;
                                }
                                else {
                                    health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                    priority *= dec!(2);
                                    if priority > dec!(1) {
                                        enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                        continue;
                                    }
                                    else {
                                        health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                        priority *= dec!(2);
                                        if priority > dec!(1) {
                                            enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                            continue;
                                        }
                                        else {
                                            health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                                            enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                                            continue;
                                        }
                                        // If u have less than 1/32 speed of your opponent you're not dead lucky you
                                    }
                                }
                            }
                        }
                    }
                }
                else {
                    enemy_health -= (damage * self.seed_50()) - (enemy_defense / dec!(2));
                    health -= (enemy_damage * self.seed_50()) - (defense / dec!(2));
                    continue;
                }
            };
        }

        // TODO: Add levelup check.
        pub fn stage_1(&mut self, 
            nft_proof: Proof, 
            weapon: Proof, 
            helmet: Proof, 
            chest: Proof, 
            pants: Proof, 
            gloves: Proof, 
            belt: Proof, 
            shoes: Proof,) -> (Bucket, Bucket, Bucket) {
            // Takes system badge for authorization
            let key_bucket: Bucket = self.system_vault.take(1);
            // Data from Proofs
            let mut nft_data: Account = nft_proof.non_fungible().data();
            let helmet_data: Armor = helmet.non_fungible().data();
            let chest_data: Armor = chest.non_fungible().data();
            let pants_data: Armor = pants.non_fungible().data();
            let gloves_data: Accessory = gloves.non_fungible().data();
            let belt_data: Accessory = belt.non_fungible().data();
            let shoes_data: Accessory = shoes.non_fungible().data();
            let mut health = nft_data.health + helmet_data.health + chest_data.health + pants_data.health;
            // Assertions so no cheating
            assert!(nft_proof.amount() == dec!("1"));
            assert!(nft_proof.resource_address() == self.character_nft,);
            assert!(nft_data.stage >= dec!(1) || nft_data.stage == dec!(1));

            assert!(weapon.amount() == dec!("1"));
            assert!(weapon.resource_address() == self.item_nft,);

            assert!(helmet.amount() == dec!("1"));
            assert!(helmet.resource_address() == self.item_nft,);
            assert!(helmet_data.part == String::from("Helmet"));

            assert!(chest.amount() == dec!("1"));
            assert!(chest.resource_address() == self.item_nft,);
            assert!(chest_data.part == String::from("Chest"));

            assert!(pants.amount() == dec!("1"));
            assert!(pants.resource_address() == self.item_nft,);
            assert!(pants_data.part == String::from("Pants"));

            assert!(gloves.amount() == dec!("1"));
            assert!(gloves.resource_address() == self.item_nft,);
            assert!(gloves_data.part == String::from("Gloves"));

            assert!(belt.amount() == dec!("1"));
            assert!(belt.resource_address() == self.item_nft,);
            assert!(belt_data.part == String::from("Belt"));

            assert!(shoes.amount() == dec!("1"));
            assert!(shoes.resource_address() == self.item_nft,);
            assert!(shoes_data.part == String::from("Shoes"));
            // To modify combat, simply change numbers for Enemy Data, EXP rewards, and Stage Number.
            let fight = self.combat(
                nft_proof.clone(), weapon.clone(), 
                helmet.clone(), chest.clone(), pants.clone(), 
                gloves.clone(), belt.clone(), shoes.clone(), 
                dec!(20), dec!(50), dec!(15), dec!(10), nft_data.health);
            let fight2 = self.combat(
                nft_proof.clone(), weapon.clone(), 
                helmet.clone(), chest.clone(), pants.clone(), 
                gloves.clone(), belt.clone(), shoes.clone(), 
                dec!(20), dec!(50), dec!(15), dec!(10), fight);
            let fight3 = self.combat(
                nft_proof.clone(), weapon.clone(), 
                helmet.clone(), chest.clone(), pants.clone(), 
                gloves.clone(), belt.clone(), shoes.clone(), 
                dec!(20), dec!(50), dec!(15), dec!(10), fight2);
            // To modify stage rewards, simply change numbers + minted rewards below
            // Numbers which drop can be randomized as well, simply add self.seed_
            if fight == dec!(0) || fight <= dec!(0) {
                nft_data.exp += 1;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(0));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(0));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(0));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));   
                self.system_vault.put(key_bucket);
                return (reward1, reward2, reward3)
            }
            else {
                nft_data.exp += 5;
            }
            if fight2 == dec!(0) || fight2 <= dec!(0) {
                nft_data.exp += 1;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(0));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(0));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(0));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));   
                self.system_vault.put(key_bucket);
                return (reward1, reward2, reward3)
            }
            else {
                nft_data.exp += 5;
            }
            if fight3 == dec!(0) || fight3 <= dec!(0) {
                nft_data.exp += 1;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(0));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(1));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(1));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));   
                self.system_vault.put(key_bucket);
                return (reward1, reward2, reward3)
            }
            else {
                nft_data.exp += 5;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(1));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(2));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(2));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));   
                self.system_vault.put(key_bucket);
                return (reward1, reward2, reward3)
            }
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
    }
}
