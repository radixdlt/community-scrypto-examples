use scrypto::prelude::*;

// Classes are: 
    // #1 = Gladiator: High attack damage, medium defense, medium speed. Can be built towards Lifesteal identity (Heal from damage, stronger activates <50% HP), or Perfect Warrior identity (attacks can combo, stronger >50% HP)
    // #2 = Rogue: Hybrid attack/magic damage, High Speed. All rounder, able to be built towards both max attack (crit burst identity) or max magic (shadow magic dps identity)
    // #3 = Ranger: Ranged Attack Damage, medium attack medium speed. Single-target ranged dps identity. 
    // #4 = Mystic: Ranged Magic Damage, High magic low defense, low speed. AOE ranged burst identity.
    // #5 = Vanguard: High Health, High Defense. Can be built towards risky Counter Identity(attack burst damage) or Suffocation Identity(magic dps damage + max tanky)
    // More Classes can be added later
#[derive(NonFungibleData)]
pub struct Account {
    #[scrypto(mutable)]
    name: String,
    #[scrypto(mutable)]
    class: u8,
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
    class: u8,
    #[scrypto(mutable)]
    name: String,
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
        char_1_stats: HashMap<u8, Vec<u64>>,
        char_2_stats: HashMap<u8, Vec<u64>>,
        char_3_stats: HashMap<u8, Vec<u64>>,
        char_4_stats: HashMap<u8, Vec<u64>>,
        char_5_stats: HashMap<u8, Vec<u64>>,
        stage_data: HashMap<u64, Vec<u64>>
         
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
                //Each line = ten levels. .first of each vector is 0 to represent lvl 0 lol
                char_1_stats: HashMap::new(),
                char_2_stats: HashMap::new(),
                char_3_stats: HashMap::new(),
                char_4_stats: HashMap::new(),
                char_5_stats: HashMap::new(),
                stage_data: HashMap::new(),

            }
            .instantiate()
            .globalize();
            (instantiate, developer_badge)
        }
        // For easier testing, not to keep 
        pub fn upload_data(&mut self, class: u8, data: u64) {
            let mut vec_hp_1 = vec![11,13,14,14,15,16,18,18,20,22,24,24,25,28,30,35,35,36,41,45];
            let mut vec_atk_1 = vec![11,12,13,14,15,17,20,22,23,25,25,27,31,34,36,38,39,41,44,47];
            let mut vec_mag_1 = vec![8,8,8,9,9,9,9,9,10,10,11,11,11,12,12,13,13,13,13,13];
            let mut vec_def_1 = vec![10,10,10,11,12,13,13,14,15,15,16,19,20,20,23,25,28,30,30,33];			
            let mut vec_spd_1 = vec![10,10,11,11,12,13,13,15,15,17,20,22,23,23,24,25,28,30,33,34];			                       			
            self.change_data(true, 1, 0, vec_hp_1);
            self.change_data(true, 1, 0, vec_atk_1);
            self.change_data(true, 1, 0, vec_mag_1);
            self.change_data(true, 1, 0, vec_def_1);
            self.change_data(true, 1, 0, vec_spd_1);
        }
        pub fn change_data(&mut self, choice: bool, class: u8, stat: u8, data: Vec<u64>) {
            self.developer_vault.authorize(||
            if choice == true {
                if class == 1 {
                    if class == 1 { self.char_1_stats.insert(stat, data); }
                    else if class == 2 { self.char_2_stats.insert(stat, data); }
                    else if class == 3 { self.char_3_stats.insert(stat, data); }
                    else if class == 4 { self.char_4_stats.insert(stat, data); }
                    else if class == 5 { self.char_5_stats.insert(stat, data); }
                }
            }
            else {
                for class in 1..=5{
                    if class == 1 { self.char_1_stats.remove(&stat); }
                    else if class == 2 { self.char_2_stats.remove(&stat); }
                    else if class == 3 { self.char_3_stats.remove(&stat); }
                    else if class == 4 { self.char_4_stats.remove(&stat); }
                    else if class == 5 { self.char_5_stats.remove(&stat); }
                }
            });
        }
        // Creates character
        pub fn create_character(&mut self, mut payment: Bucket, class: u8, name: String) -> (Bucket, Bucket, Bucket, Bucket, Bucket, Bucket, Bucket, Bucket, Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);
            let hp = match class {
                1 => self.char_1_stats.get(&0).unwrap(),  
                2 => self.char_2_stats.get(&0).unwrap(), 
                3 => self.char_3_stats.get(&0).unwrap(),  
                4 => self.char_4_stats.get(&0).unwrap(),  
                5 => self.char_5_stats.get(&0).unwrap(), 
                _ => self.char_1_stats.get(&0).unwrap(),   
            };
            let atk = match class {
                1 => self.char_1_stats.get(&1).unwrap(),  
                2 => self.char_2_stats.get(&1).unwrap(),  
                3 => self.char_3_stats.get(&1).unwrap(),  
                4 => self.char_4_stats.get(&1).unwrap(),  
                5 => self.char_5_stats.get(&1).unwrap(), 
                _ => self.char_1_stats.get(&1).unwrap(),   
            };
            let mag = match class {
                1 => self.char_1_stats.get(&2).unwrap(),  
                2 => self.char_2_stats.get(&2).unwrap(),  
                3 => self.char_3_stats.get(&2).unwrap(),  
                4 => self.char_4_stats.get(&2).unwrap(),  
                5 => self.char_5_stats.get(&2).unwrap(), 
                _ => self.char_1_stats.get(&2).unwrap(),   
            };
            let def = match class {
                1 => self.char_1_stats.get(&3).unwrap(),  
                2 => self.char_2_stats.get(&3).unwrap(),  
                3 => self.char_3_stats.get(&3).unwrap(),  
                4 => self.char_4_stats.get(&3).unwrap(),  
                5 => self.char_5_stats.get(&3).unwrap(), 
                _ => self.char_1_stats.get(&3).unwrap(),   
            };
            let spd = match class {
                1 => self.char_1_stats.get(&4).unwrap(),  
                2 => self.char_2_stats.get(&4).unwrap(),  
                3 => self.char_3_stats.get(&4).unwrap(),  
                4 => self.char_4_stats.get(&4).unwrap(),  
                5 => self.char_5_stats.get(&4).unwrap(), 
                _ => self.char_1_stats.get(&4).unwrap(),   
            };

            let character_data = Account { name: name, class: class, level: dec!(1), exp: 0, stage: dec!(1),
                health: hp[0].into(), attack: atk[0].into(), magic: mag[0].into(), 
                defense: def[0].into(), speed: spd[0].into(), version: self.version, 
            };
            let weapon_data = Weapon { class: class, name: String::from("Stick"), level: dec!(1), physical_base: dec!(1),
                physical_scaling: dec!(100), spell_base: dec!(1), spell_scaling: dec!(100), ability: dec!(0),
                ability_scaling: dec!(0), range: dec!(1), version: self.version,
            };
            let helmet_data = Armor { id: dec!(0), part: String::from("Helmet"), level: dec!(1), health: dec!(0),
                defense: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let chest_data = Armor { id: dec!(0), part: String::from("Chest"), level: dec!(1), health: dec!(0),
            defense: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let pants_data = Armor { id: dec!(0), part: String::from("Pants"), level: dec!(1), health: dec!(0),
            defense: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let gloves_data = Accessory { id: dec!(0), part: String::from("Gloves"), level: dec!(1), attack: dec!(0),
                magic: dec!(0), speed: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let belt_data = Accessory { id: dec!(0), part: String::from("Belt"), level: dec!(1), attack: dec!(0),
            magic: dec!(0), speed: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let shoes_data = Accessory { id: dec!(0), part: String::from("Shoes"), level: dec!(1), attack: dec!(0),
            magic: dec!(0), speed: dec!(0), weight: dec!(0), ability: dec!(0), ability_scaling: dec!(0), version: self.version,
            };
            let new_helmet = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), helmet_data));
            let new_chest = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), chest_data));
            let new_pants = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), pants_data));
            let new_gloves = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), gloves_data));
            let new_belt = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), belt_data));
            let new_shoes = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), shoes_data));
            let new_weapon = self.system_vault.authorize(|| borrow_resource_manager!(self.item_nft)
                .mint_non_fungible(&NonFungibleId::random(), weapon_data));
            let new_character = self.system_vault.authorize(|| borrow_resource_manager!(self.character_nft)
                .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data));
                    
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            self.system_vault.put(key_bucket);
            return (new_character, new_weapon, new_helmet, new_chest, new_pants, new_gloves, new_belt, new_shoes, payment,)
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
            let point = digits[6];
            let point2 = digits[9];
            let random: u128 = format!("{}{}", point, point2).parse().unwrap();
            match random {
                00 => dec!(75) / dec!(100),
                01 => dec!(125) / dec!(100),
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
                _ => dec!(1) / dec!(100),
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
            let point = digits[6];
            match point {
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
        pub fn fuse_items(&mut self, which: u8, item_bucket: Bucket) -> Bucket {
            assert!(item_bucket.amount() == dec!("2"));
            assert!(item_bucket.resource_address() == self.item_nft);
            if which == 1 {
                let mut item: Weapon = item_bucket.non_fungibles()[0].data();
                let item2: Weapon = item_bucket.non_fungibles()[1].data();
                assert!(item.level == item2.level);
                assert!(item.name == item2.name);
                item.level += dec!(1);
                item.physical_base *= dec!(5)/dec!(4);
                item.physical_scaling *= dec!(5)/dec!(4);
                item.spell_base *= dec!(5)/dec!(4);
                item.spell_scaling *= dec!(5)/dec!(4); 
                item.ability_scaling *= dec!(5)/dec!(4);          
                let new = Weapon { class: item.class, name: item.name, level: item.level, physical_base: item.physical_base,
                    physical_scaling: item.physical_scaling, spell_base: item.spell_base, spell_scaling: item.spell_scaling, ability: item.ability,
                    ability_scaling: item.ability_scaling, range: item.range, version: self.version,
                };
                self.system_vault.authorize(||item_bucket.burn());
                let new_bucket = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), new));
                return new_bucket
            }
            else if which == 2 {
                let mut item: Armor = item_bucket.non_fungibles()[0].data();
                let item2: Armor = item_bucket.non_fungibles()[1].data();   
                assert!(item.level == item2.level);            
                assert!(item.id == item2.id);
                assert!(item.part == item2.part);
                item.level += dec!(1); 
                item.health *= dec!(5)/dec!(4);
                item.defense *= dec!(5)/dec!(4);
                item.ability_scaling *= dec!(5)/dec!(4);  
                let new = Armor { id: item.id, part: item.part, level: item.level, health: item.health,
                    defense: item.defense, weight: item.weight, ability: item.ability, ability_scaling: item.ability_scaling, version: self.version,
                    };
                self.system_vault.authorize(||item_bucket.burn());
                let new_bucket = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), new));
                return new_bucket
            }
            else {
                let mut item: Accessory = item_bucket.non_fungibles()[0].data();
                let item2: Accessory = item_bucket.non_fungibles()[1].data();  
                assert!(item.level == item2.level);        
                assert!(item.id == item2.id);
                assert!(item.part == item2.part);
                item.level += dec!(1); 
                item.attack *= dec!(4)/dec!(5);
                item.magic *= dec!(4)/dec!(5);
                item.speed *= dec!(4)/dec!(5);
                item.ability_scaling *= dec!(4)/dec!(5);  
                let new = Accessory { id: item.id, part: item.part, level: item.level, attack: item.attack,
                    magic: item.magic, speed: item.speed, weight: item.weight, ability: item.ability, ability_scaling: item.ability_scaling, version: self.version,
                    };
                self.system_vault.authorize(||item_bucket.burn());
                let new_bucket = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), new));
                return new_bucket
            }              
        }
        // Creates weapons
        pub fn create_weapon_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket, class: u8) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let class_chosen = if class == 1 { String::from("Sword") }
                else if class == 2 { String::from("Dagger") }
                else if class == 3 { String::from("Bow") }
                else if class == 4 { String::from("Staff") }
                else { String::from("Shield")
            };
            let p1 = if class == 1 { dec!(25) }
                else if class == 2 { dec!(15) }
                else if class == 3 { dec!(15) }
                else if class == 4 { dec!(0) }
                else { dec!(20)
            };
            let p2 = if class == 1 { dec!(110) }
                else if class == 2 { dec!(50) }
                else if class == 3 { dec!(40) }
                else if class == 4 { dec!(0) }
                else { dec!(20)
            };
            let m1 = if class == 1 { dec!(0) }
                else if class == 2 { dec!(15) }
                else if class == 3 { dec!(15) }
                else if class == 4 { dec!(20) }
                else { dec!(20)
            };
            let m2 = if class == 1 { dec!(0) }
                else if class == 2 { dec!(50) }
                else if class == 3 { dec!(40) }
                else if class == 4 { dec!(120) }
                else { dec!(20)
            };
            let phys_base = p1 * self.seed_50();
            let phys_scale = p2 * self.seed_50();
            let mag_base = m1 * self.seed_50();
            let mag_scale = m2 * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            let weapon_data = Weapon {  
                class: 1,
                name: class_chosen,
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
        pub fn create_armor_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket, part_chosen: u8) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let part = if part_chosen == 1 {
                String::from("Helmet")
            }
            else if part_chosen == 2 {
                String::from("Chest")
            }
            else {
                String::from("Pants")
            };
            let hp = dec!(5) * self.seed_50();
            let def = dec!(5) * self.seed_50();
            let hp_bonus = dec!(10) * self.seed_50();
            let def_bonus = dec!(10) * self.seed_50();
            let load = dec!(5) * self.seed_50();
            let ability_scale = dec!(0) * self.seed_50();
            if self.seed_10() < dec!(5) {
                let armor_data = Armor {  
                    id: dec!(1),
                    part: part,
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
                    part: part,
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
        pub fn create_accessory_1(&mut self, gold: Bucket, resource1: Bucket, resource2: Bucket, part_chosen: u8,) -> Bucket {
            assert!(gold.resource_address() == self.token_gold);
            assert!(resource1.resource_address() == self.token_greavite);
            assert!(resource2.resource_address() == self.token_wood);
            let key_bucket: Bucket = self.system_vault.take(1);
            let part = if part_chosen == 1 {
                String::from("Gloves")
            }
            else if part_chosen == 2 {
                String::from("Belt")
            }
            else {
                String::from("Shoes")
            };
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
                let accessory_data = Accessory {  
                    id: dec!(1),
                    part: part,
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk_bonus),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_accessory = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), accessory_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_accessory
            }
            else if rng > dec!(24) && rng < dec!(50) {
                let accessory_data = Accessory {  
                    id: dec!(1),
                    part: part,
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag_bonus),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_accessory = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), accessory_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_accessory
            }
            else if rng > dec!(49) && rng < dec!(75) {
                let accessory_data = Accessory {  
                    id: dec!(1),
                    part: part,
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd_bonus),
                    weight: Decimal::floor(&load),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_accessory = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), accessory_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_accessory
            }
            else {
                let accessory_data = Accessory {  
                    id: dec!(1),
                    part: part,
                    level: dec!(1), 
                    attack: Decimal::ceiling(&atk),
                    magic: Decimal::ceiling(&mag),
                    speed: Decimal::ceiling(&spd),
                    weight: Decimal::floor(&load_bonus),
                    ability: dec!(0),
                    ability_scaling: Decimal::ceiling(&ability_scale),
                    version: dec!(1),
                };
                let new_accessory = self.system_vault.authorize(||
                    borrow_resource_manager!(self.item_nft)
                        .mint_non_fungible(&NonFungibleId::random(), accessory_data));
                
                self.system_vault.authorize(|| 
                    gold.burn());
                self.system_vault.authorize(||
                    resource1.burn());
                self.system_vault.authorize(||
                    resource2.burn());
                self.system_vault.put(key_bucket);
                new_accessory
            }
        }

        // Calculates the combat between user and enemies. Enemies Data + rewards set by caller. Boolean, Boolean
        // TODO: make it so damage cannot go under 1, Shorten method using Proof copies.
        pub fn combat(
            &mut self, 
            mut health: Decimal, 
            mut damage: Decimal,
            mut defense: Decimal,  
            mut speed: Decimal, 
            mut enemy_health: Decimal, 
            mut enemy_damage: Decimal, 
            mut enemy_defense: Decimal, 
            mut enemy_speed: Decimal,
            ) -> Decimal {
            let speed_ratio = speed / enemy_speed;
            loop {
                let mut damage_given = damage - (enemy_defense / dec!(2));
                if damage_given == dec!(1) || damage_given <= dec!(1) {
                    damage_given = dec!(1)
                }
                let mut damage_taken = enemy_damage  - (defense / dec!(2));
                if damage_taken == dec!(1) || damage_given <= dec!(1) {
                    damage_taken = dec!(1)
                }
                let mut priority = speed_ratio * self.seed_50();
                if health < dec!(0) || health == dec!(0) {
                    return health
                }
                if enemy_health < dec!(0) || enemy_health == dec!(0) {
                    return health
                }
                if priority > dec!(1) {
                    enemy_health -= damage_given * self.seed_50();
                    if enemy_health < dec!(0) || enemy_health == dec!(0) {
                        return health
                    }
                    priority *= dec!(65)/dec!(100);
                    if priority < dec!(1) {
                        health -= damage_taken * self.seed_50();
                        if health < dec!(0) || health == dec!(0) {
                            return health
                        }
                        continue;
                    }
                    else {
                        enemy_health -= damage_given * self.seed_50();
                        if enemy_health < dec!(0) || enemy_health == dec!(0) {
                            return health
                        }
                        priority *= dec!(65)/dec!(100);
                        if priority < dec!(1) {
                            health -= damage_taken * self.seed_50();
                            if health < dec!(0) || health == dec!(0) {
                                return health
                            }
                            continue;
                        }
                        else {
                            enemy_health -= damage_given * self.seed_50();
                            if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                return health
                            }
                            priority *= dec!(65)/dec!(100);
                            if priority < dec!(1) {
                                health -= damage_taken * self.seed_50();
                                if health < dec!(0) || health == dec!(0) {
                                    return health
                                }
                                continue;
                            }
                            else {
                                enemy_health -= damage_given * self.seed_50();
                                if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                    return health
                                }
                                priority *= dec!(65)/dec!(100);
                                if priority < dec!(1) {
                                    health -= damage_taken * self.seed_50();
                                    if health < dec!(0) || health == dec!(0) {
                                        return health
                                    }
                                    continue;
                                }
                                else {
                                    enemy_health -= damage_given * self.seed_50();
                                    if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                        return health
                                    }
                                    priority *= dec!(65)/dec!(100);
                                    if priority < dec!(1) {
                                        health -= damage_taken * self.seed_50();
                                        if health < dec!(0) || health == dec!(0) {
                                            return health
                                        }
                                        continue;
                                    }
                                    else {
                                        enemy_health -= damage_given * self.seed_50();
                                        if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                            return health
                                        }
                                        priority *= dec!(65)/dec!(100);
                                        if priority < dec!(1) {
                                            health -= damage_taken * self.seed_50();
                                            if health < dec!(0) || health == dec!(0) {
                                                return health
                                            }
                                            continue;
                                        }
                                        else {
                                            health -= damage_taken * self.seed_50();
                                            if health < dec!(0) || health == dec!(0) {
                                                return health
                                            }
                                            enemy_health -= damage_given * self.seed_50();
                                            if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                                return health
                                            }
                                            continue;
                                        }
                                        // This is enough for 99% of cases
                                    }
                                }
                            }
                        }
                    }
                }
                if priority < dec!(1) {
                    health -= damage_taken * self.seed_50();
                    if health < dec!(0) || health == dec!(0) {
                        return health
                    }
                    priority *= dec!(135)/dec!(100);
                    if priority > dec!(1) {
                        enemy_health -= damage_given * self.seed_50();
                        if enemy_health < dec!(0) || enemy_health == dec!(0) {
                            return health
                        }
                        continue;
                    }
                    else {
                        health -= damage_taken * self.seed_50();
                        if health < dec!(0) || health == dec!(0) {
                            return health
                        }
                        priority *= dec!(135)/dec!(100);
                        if priority > dec!(1) {
                            enemy_health -= damage_given * self.seed_50();
                            if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                return health
                            }
                            continue;
                        }
                        else {
                            health -= damage_taken * self.seed_50();
                            if health < dec!(0) || health == dec!(0) {
                                return health
                            }
                            priority *= dec!(135)/dec!(100);
                            if priority > dec!(1) {
                                enemy_health -= damage_given * self.seed_50();
                                if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                    return health
                                }
                                continue;
                            }
                            else {
                                health -= damage_taken * self.seed_50();
                                if health < dec!(0) || health == dec!(0) {
                                    return health
                                }
                                priority *= dec!(135)/dec!(100);
                                if priority > dec!(1) {
                                    enemy_health -= damage_given * self.seed_50();
                                    if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                        return health
                                    }
                                    continue;
                                }
                                else {
                                    health -= damage_taken * self.seed_50();
                                    if health < dec!(0) || health == dec!(0) {
                                        return health
                                    }
                                    priority *= dec!(135)/dec!(100);
                                    if priority > dec!(1) {
                                        enemy_health -= damage_given * self.seed_50();
                                        if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                            return health
                                        }
                                        continue;
                                    }
                                    else {
                                        health -= damage_taken * self.seed_50();
                                        if health < dec!(0) || health == dec!(0) {
                                            return health
                                        }
                                        priority *= dec!(135)/dec!(100);
                                        if priority > dec!(1) {
                                            enemy_health -= damage_given * self.seed_50();
                                            if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                                return health
                                            }
                                            continue;
                                        }
                                        else {
                                            health -= damage_taken * self.seed_50();
                                            enemy_health -= damage_given * self.seed_50();
                                            if health < dec!(0) || health == dec!(0) {
                                                return health
                                            }
                                            if enemy_health < dec!(0) || enemy_health == dec!(0) {
                                                return health
                                            }
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                else {
                    enemy_health -= damage_given * self.seed_50();
                    if enemy_health < dec!(0) || enemy_health == dec!(0) {
                        return health
                    }
                    health -= damage_taken * self.seed_50();
                    if health < dec!(0) || health == dec!(0) {
                        return health
                    }
                    continue;
                }
            };
        }

        // Place character,weapon,armor, and accessory data + stage # to fight. 
        // Method calculates whether you win and grants rewards based on win or loss
        pub fn stage(&mut self, 
            nft_proof: Proof, 
            weapon: Proof, 
            helmet: Proof, 
            chest: Proof, 
            pants: Proof, 
            gloves: Proof, 
            belt: Proof, 
            shoes: Proof,
            stage: u64,
            ) -> (Bucket, Bucket, Bucket) {
            // Takes system badge for authorization
            let key_bucket: Bucket = self.system_vault.take(1);
            // Data from Proofs
            let mut nft_data: Account = nft_proof.non_fungible().data();
            let mut weapon_data: Weapon = weapon.non_fungible().data();
            let helmet_data: Armor = helmet.non_fungible().data();
            let chest_data: Armor = chest.non_fungible().data();
            let pants_data: Armor = pants.non_fungible().data();
            let gloves_data: Accessory = gloves.non_fungible().data();
            let belt_data: Accessory = belt.non_fungible().data();
            let shoes_data: Accessory = shoes.non_fungible().data();
            // Getting data of selected stage:
            let data = self.stage_data.get(&stage).unwrap();
            let enemy_1_hp: Decimal = data[1].into();
            let enemy_1_dmg: Decimal = data[2].into();
            let enemy_1_def: Decimal = data[3].into();
            let enemy_1_spd: Decimal = data[4].into();
            let enemy_2_hp: Decimal = data[5].into();
            let enemy_2_dmg: Decimal = data[6].into();
            let enemy_2_def: Decimal = data[7].into();
            let enemy_2_spd: Decimal = data[8].into();
            let enemy_3_hp: Decimal = data[9].into();
            let enemy_3_dmg: Decimal = data[10].into();
            let enemy_3_def: Decimal = data[11].into();
            let enemy_3_spd: Decimal = data[12].into();
            let reward1_d_1 = data[13];
            let reward2_d_1 = data[14];
            let reward3_d_1 = data[15];
            let reward1_d_2 = data[16];
            let reward2_d_2 = data[17];
            let reward3_d_2 = data[18];
            let reward1_d_3 = data[19];
            let reward2_d_3 = data[20];
            let reward3_d_3 = data[21];
            let reward1_w = data[22];
            let reward2_w = data[23];
            let reward3_w = data[24];
            // Assertions so no cheating
            assert!(nft_proof.amount() == dec!("1"));
            assert!(nft_proof.resource_address() == self.character_nft,);
            assert!(nft_data.stage >= data[0].into() || nft_data.stage == data[0].into());

            assert!(weapon.resource_address() == self.item_nft,);

            assert!(helmet.resource_address() == self.item_nft,);
            assert!(helmet_data.part == String::from("Helmet"));

            assert!(chest.resource_address() == self.item_nft,);
            assert!(chest_data.part == String::from("Chest"));

            assert!(pants.resource_address() == self.item_nft,);
            assert!(pants_data.part == String::from("Pants"));

            assert!(gloves.resource_address() == self.item_nft,);
            assert!(gloves_data.part == String::from("Gloves"));

            assert!(belt.resource_address() == self.item_nft,);
            assert!(belt_data.part == String::from("Belt"));

            assert!(shoes.resource_address() == self.item_nft,);
            assert!(shoes_data.part == String::from("Shoes"));
            // Calculations
            let mut speed = 
                (nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) -  
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * helmet_data.weight / dec!(100)) - 
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * chest_data.weight / dec!(100)) - 
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * pants_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * gloves_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * belt_data.weight / dec!(100)) -
                ((nft_data.speed + gloves_data.speed + belt_data.speed + shoes_data.speed) * shoes_data.weight / dec!(100));
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
            let mut health = nft_data.health + helmet_data.health + chest_data.health + pants_data.health;
            let mut damage: Decimal = 
                (weapon_data.physical_base + (weapon_data.physical_scaling / dec!(100) * attack)) +
                (weapon_data.spell_base + (weapon_data.spell_scaling / dec!(100) * magic));  
            // To modify combat, simply change numbers for Enemy Data, EXP rewards, and Stage Number.
            let fight = self.combat(health, damage, defense, speed, enemy_1_hp, enemy_1_dmg, enemy_1_def, enemy_1_spd);
            let fight2 = self.combat(fight, damage, defense, speed, enemy_2_hp, enemy_2_dmg, enemy_2_def, enemy_2_spd);
            let fight3 = self.combat(fight2, damage, defense, speed, enemy_3_hp, enemy_3_dmg, enemy_3_def, enemy_3_spd);
            // To modify stage rewards, simply change numbers + minted rewards below
            // Numbers which drop can be randomized as well, simply add self.seed_
            let class = nft_data.class;
            if fight == dec!(0) || fight <= dec!(0) {
                nft_data.exp += 1;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(reward1_d_1));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(reward2_d_1));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(reward3_d_1));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                self.levelup(nft_proof);
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
                        .mint(reward1_d_2));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(reward2_d_2));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(reward3_d_2));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                self.levelup(nft_proof);
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
                        .mint(reward1_d_3));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(reward2_d_3));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(reward3_d_3));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data));
                self.levelup(nft_proof);
                self.system_vault.put(key_bucket);
                return (reward1, reward2, reward3)
            }
            else {
                nft_data.exp += 5;
                let reward1 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_gold)
                        .mint(reward1_w));
                let reward2 = self.system_vault.authorize(||
                        borrow_resource_manager!(self.token_wood)
                        .mint(reward2_w));
                let reward3 = self.system_vault.authorize(||
                    borrow_resource_manager!(self.token_greavite)
                        .mint(reward3_w));
                self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data)); 
                self.levelup(nft_proof);
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

        //Levelup method
        pub fn levelup(&mut self, nft_proof: Proof) {
            let mut nft_data: Account = nft_proof.non_fungible().data();
            assert!(
                nft_proof.amount() == dec!("1"),
                "You can only level up one character at once!"
            );
        
            assert!(
                nft_proof.resource_address() == self.character_nft,
                "Wrong resource address!"
            );
            let total_exp = nft_data.exp;
            let class = nft_data.class;
            let hp = match class {
                1 => self.char_1_stats.get(&0).unwrap(),  
                2 => self.char_2_stats.get(&0).unwrap(), 
                3 => self.char_3_stats.get(&0).unwrap(),  
                4 => self.char_4_stats.get(&0).unwrap(),  
                5 => self.char_5_stats.get(&0).unwrap(), 
                _ => self.char_1_stats.get(&0).unwrap(),   
            };
            let atk = match class {
                1 => self.char_1_stats.get(&1).unwrap(),  
                2 => self.char_2_stats.get(&1).unwrap(),  
                3 => self.char_3_stats.get(&1).unwrap(),  
                4 => self.char_4_stats.get(&1).unwrap(),  
                5 => self.char_5_stats.get(&1).unwrap(), 
                _ => self.char_1_stats.get(&1).unwrap(),   
            };
            let mag = match class {
                1 => self.char_1_stats.get(&2).unwrap(),  
                2 => self.char_2_stats.get(&2).unwrap(),  
                3 => self.char_3_stats.get(&2).unwrap(),  
                4 => self.char_4_stats.get(&2).unwrap(),  
                5 => self.char_5_stats.get(&2).unwrap(), 
                _ => self.char_1_stats.get(&2).unwrap(),   
            };
            let def = match class {
                1 => self.char_1_stats.get(&3).unwrap(),  
                2 => self.char_2_stats.get(&3).unwrap(),  
                3 => self.char_3_stats.get(&3).unwrap(),  
                4 => self.char_4_stats.get(&3).unwrap(),  
                5 => self.char_5_stats.get(&3).unwrap(), 
                _ => self.char_1_stats.get(&3).unwrap(),   
            };
            let spd = match class {
                1 => self.char_1_stats.get(&4).unwrap(),  
                2 => self.char_2_stats.get(&4).unwrap(),  
                3 => self.char_3_stats.get(&4).unwrap(),  
                4 => self.char_4_stats.get(&4).unwrap(),  
                5 => self.char_5_stats.get(&4).unwrap(), 
                _ => self.char_1_stats.get(&4).unwrap(),   
            };

            // Idk if this is good, the level scaling seems wack af lmao
            match total_exp {
                0..=14 =>  { return },
                15..=48 => { 
                    if nft_data.level < dec!(2) {
                        nft_data.level = dec!(2);
                        nft_data.health = hp[1].into();
                        nft_data.attack = atk[1].into();
                        nft_data.magic = mag[1].into();
                        nft_data.defense = def[1].into();
                        nft_data.speed = spd[1].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                49..=105 => { 
                    if nft_data.level < dec!(3) {
                        nft_data.level = dec!(3);
                        nft_data.health = hp[2].into();
                        nft_data.attack = atk[2].into();
                        nft_data.magic = mag[2].into();
                        nft_data.defense = def[2].into();
                        nft_data.speed = spd[2].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                106..=197 => { 
                    if nft_data.level < dec!(4) {
                        nft_data.level = dec!(4);
                        nft_data.health = hp[3].into();
                        nft_data.attack = atk[3].into();
                        nft_data.magic = mag[3].into();
                        nft_data.defense = def[3].into();
                        nft_data.speed = spd[3].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                198..=332 => { 
                    if nft_data.level < dec!(5) {
                        nft_data.level = dec!(5);
                        nft_data.health = hp[4].into();
                        nft_data.attack = atk[4].into();
                        nft_data.magic = mag[4].into();
                        nft_data.defense = def[4].into();
                        nft_data.speed = spd[4].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                333..=704 => { 
                    if nft_data.level < dec!(6) {
                        nft_data.level = dec!(6);
                        nft_data.health = hp[5].into();
                        nft_data.attack = atk[5].into();
                        nft_data.magic = mag[5].into();
                        nft_data.defense = def[5].into();
                        nft_data.speed = spd[5].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                705..=1264 => { 
                    if nft_data.level < dec!(7) {
                        nft_data.level = dec!(7);
                        nft_data.health = hp[6].into();
                        nft_data.attack = atk[6].into();
                        nft_data.magic = mag[6].into();
                        nft_data.defense = def[6].into();
                        nft_data.speed = spd[6].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                1265..=2104 => { 
                    if nft_data.level < dec!(8) {
                        nft_data.level = dec!(8);
                        nft_data.health = hp[7].into();
                        nft_data.attack = atk[7].into();
                        nft_data.magic = mag[7].into();
                        nft_data.defense = def[7].into();
                        nft_data.speed = spd[7].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                2105..=3346 => { 
                    if nft_data.level < dec!(9) {
                        nft_data.level = dec!(9);
                        nft_data.health = hp[8].into();
                        nft_data.attack = atk[8].into();
                        nft_data.magic = mag[8].into();
                        nft_data.defense = def[8].into();
                        nft_data.speed = spd[8].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                3347..=4588 => { 
                    if nft_data.level < dec!(10) {
                        nft_data.level = dec!(10);
                        nft_data.health = hp[9].into();
                        nft_data.attack = atk[9].into();
                        nft_data.magic = mag[9].into();
                        nft_data.defense = def[9].into();
                        nft_data.speed = spd[9].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                4589..=5830 => { 
                    if nft_data.level < dec!(11) {
                        nft_data.level = dec!(11);
                        nft_data.health = hp[10].into();
                        nft_data.attack = atk[10].into();
                        nft_data.magic = mag[10].into();
                        nft_data.defense = def[10].into();
                        nft_data.speed = spd[10].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                5831..=7072 => { 
                    if nft_data.level < dec!(12) {
                        nft_data.level = dec!(12);
                        nft_data.health = hp[11].into();
                        nft_data.attack = atk[11].into();
                        nft_data.magic = mag[11].into();
                        nft_data.defense = def[11].into();
                        nft_data.speed = spd[11].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                7073..=8314 => { 
                    if nft_data.level < dec!(13) {
                        nft_data.level = dec!(13);
                        nft_data.health = hp[12].into();
                        nft_data.attack = atk[12].into();
                        nft_data.magic = mag[12].into();
                        nft_data.defense = def[12].into();
                        nft_data.speed = spd[12].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                8315..=9556 => { 
                    if nft_data.level < dec!(14) {
                        nft_data.level = dec!(14);
                        nft_data.health = hp[13].into();
                        nft_data.attack = atk[13].into();
                        nft_data.magic = mag[13].into();
                        nft_data.defense = def[13].into();
                        nft_data.speed = spd[13].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                9557..=11046 => { 
                    if nft_data.level < dec!(15) {
                        nft_data.level = dec!(15);
                        nft_data.health = hp[14].into();
                        nft_data.attack = atk[14].into();
                        nft_data.magic = mag[14].into();
                        nft_data.defense = def[14].into();
                        nft_data.speed = spd[14].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                11047..=12834 => { 
                    if nft_data.level < dec!(16) {
                        nft_data.level = dec!(16);
                        nft_data.health = hp[15].into();
                        nft_data.attack = atk[15].into();
                        nft_data.magic = mag[15].into();
                        nft_data.defense = def[15].into();
                        nft_data.speed = spd[15].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                12835..=14979 => { 
                    if nft_data.level < dec!(17) {
                        nft_data.level = dec!(17);
                        nft_data.health = hp[16].into();
                        nft_data.attack = atk[16].into();
                        nft_data.magic = mag[16].into();
                        nft_data.defense = def[16].into();
                        nft_data.speed = spd[16].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                14980..=17553 => { 
                    if nft_data.level < dec!(18) {
                        nft_data.level = dec!(18);
                        nft_data.health = hp[17].into();
                        nft_data.attack = atk[17].into();
                        nft_data.magic = mag[17].into();
                        nft_data.defense = def[17].into();
                        nft_data.speed = spd[17].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                17554..=20641 => { 
                    if nft_data.level < dec!(19) {
                        nft_data.level = dec!(19);
                        nft_data.health = hp[18].into();
                        nft_data.attack = atk[18].into();
                        nft_data.magic = mag[18].into();
                        nft_data.defense = def[18].into();
                        nft_data.speed = spd[18].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                20642..=1000000 => { 
                    if nft_data.level < dec!(20) {
                        nft_data.level = dec!(20);
                        nft_data.health = hp[19].into();
                        nft_data.attack = atk[19].into();
                        nft_data.magic = mag[19].into();
                        nft_data.defense = def[19].into();
                        nft_data.speed = spd[19].into();
                        self.system_vault.authorize(|| nft_proof.non_fungible().update_data(nft_data))
                    }
                    else {
                        return
                    }
                },
                _ =>  { return }
            }
        }
    }
}
