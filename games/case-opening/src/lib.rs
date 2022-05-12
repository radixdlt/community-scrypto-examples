use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Skin {
    name: String,
    weapon: String,
    wear: String,
    rarity: String,
    float: Decimal,
    pattern: u16,
    stat_trak: bool,
    
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
                developer_vault: Vault::with_bucket(developer_badge.take(9999)),
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

        pub fn open_case(&mut self, key: Bucket) -> Bucket {
            
            let key_bucket: Bucket = self.system_vault.take(1);
            let random = self.random_data();
            let rarity = match random.1 {
                0..=7992326 => String::from("Mil-Spec"),
                7992327..=9590791 => String::from("Restricted"),
                9590792..=9910484 => String::from("Classified"),
                9910485..=9974423 => String::from("Covert"),
                9974424..=9999999 => String::from("Rare Special Item"),
                _ => String::from("Error!"),
            };
            if rarity == String::from("Mil-Spec") {
                let name = match random.5 {
                    1..=4 => String::from("Armor Core"),
                    5..=8 => String::from("Elite Build"),
                    9..=12 => String::from("Bronze Deco"),
                    13..=16 => String::from("Man o` war"),
                    17..=20 => String::from("Origami"),
                    21..=24 => String::from("Valence"),
                    _ => String::from("Error"),
                };
                let weapon = match random.5 {
                    1..=4 => String::from("Mp7"),
                    5..=8 => String::from("Ak47"),
                    9..=12 => String::from("Desert Eagle"),
                    13..=16 => String::from("Negev"),
                    17..=20 => String::from("Sawed Off"),
                    21..=24 => String::from("P250"),
                    _ => String::from("Error"),
                };
                let skin_data = Skin {
                    name: name,
                    weapon: weapon,
                    wear: random.3,
                    rarity: rarity,
                    float: random.2,
                    pattern: random.0,
                    stat_trak: random.4,
                };
                let new_skin = self.system_vault.authorize(||
                    borrow_resource_manager!(self.skin_nft)
                        .mint_non_fungible(&NonFungibleId::random(), skin_data));

                self.system_vault.authorize(||
                    key.burn());
                self.system_vault.put(key_bucket);
                return new_skin
            }
            else if rarity  == String::from("Restricted") {
                let name = match random.5 {
                    1..=6 => String::from("Pole Position"),
                    7..=12 => String::from("Grand Prix"),
                    13..=18 => String::from("Heat"),
                    19..=24 => String::from("Worm God"),
                    _ => String::from("Error"),
                };
                let weapon = match random.5 {
                    1..=6 => String::from("CZ75 Auto"),
                    7..=12 => String::from("UMP 45"),
                    13..=18 => String::from("MAG 7"),
                    19..=24 => String::from("AWP"),
                    _ => String::from("Error"),
                };
                let skin_data = Skin {
                    name: name,
                    weapon: weapon,
                    wear: random.3,
                    rarity: rarity,
                    float: random.2,
                    pattern: random.0,
                    stat_trak: random.4,
                };
                let new_skin = self.system_vault.authorize(||
                    borrow_resource_manager!(self.skin_nft)
                        .mint_non_fungible(&NonFungibleId::random(), skin_data));

                self.system_vault.authorize(||
                    key.burn());
                self.system_vault.put(key_bucket);
                return new_skin
            }
            else if rarity  == String::from("Classified") {
                let name = match random.5 {
                    1..=8 => String::from("Eco"),
                    9..=16 => String::from("Djinn"),
                    17..=24 => String::from("Monkey Business"),
                    _ => String::from("Error"),
                };
                let weapon = match random.5 {
                    1..=8 => String::from("Galil AR"),
                    9..=16 => String::from("FAMAS"),
                    17..=24 => String::from("Five SeveN"),
                    _ => String::from("Error"),
                };
                let skin_data = Skin {
                    name: name,
                    weapon: weapon,
                    wear: random.3,
                    rarity: rarity,
                    float: random.2,
                    pattern: random.0,
                    stat_trak: random.4,
                };
                let new_skin = self.system_vault.authorize(||
                    borrow_resource_manager!(self.skin_nft)
                        .mint_non_fungible(&NonFungibleId::random(), skin_data));

                self.system_vault.authorize(||
                    key.burn());
                self.system_vault.put(key_bucket);
                return new_skin
            }
            else if rarity == String::from("Covert") {
                let name = match random.5 {
                    1..=16 => String::from("Neon Rider"),
                    17..=24 => String::from("Hyper Beast"),
                    _ => String::from("Error"),
                };
                let weapon = match random.5 {
                    1..=16 => String::from("MAC 10"),
                    17..=24 => String::from("M4A1 S"),
                    _ => String::from("Error"),
                };
                let skin_data = Skin {
                    name: name,
                    weapon: weapon,
                    wear: random.3,
                    rarity: rarity,
                    float: random.2,
                    pattern: random.0,
                    stat_trak: random.4,
                };
                let new_skin = self.system_vault.authorize(||
                    borrow_resource_manager!(self.skin_nft)
                        .mint_non_fungible(&NonFungibleId::random(), skin_data));

                self.system_vault.authorize(||
                    key.burn());
                self.system_vault.put(key_bucket);
                return new_skin
            }
            else {
                let name = match random.5 {
                    1..=6 => String::from("Pole Position"),
                    7..=12 => String::from("Grand Prix"),
                    13..=18 => String::from("Heat"),
                    19..=24 => String::from("Worm God"),
                    _ => String::from("Error"),
                };
                let weapon = match random.5 {
                    1..=6 => String::from("CZ75 Auto"),
                    7..=12 => String::from("UMP 45"),
                    13..=18 => String::from("MAG 7"),
                    19..=24 => String::from("AWP"),
                    _ => String::from("Error"),
                };
                let skin_data = Skin {
                    name: name,
                    weapon: weapon,
                    wear: random.3,
                    rarity: rarity,
                    float: random.2,
                    pattern: random.0,
                    stat_trak: random.4,
                };
                let new_skin = self.system_vault.authorize(||
                    borrow_resource_manager!(self.skin_nft)
                        .mint_non_fungible(&NonFungibleId::random(), skin_data));

                self.system_vault.authorize(||
                    key.burn());
                self.system_vault.put(key_bucket);
                return new_skin
            };
        }
 
        pub fn random_data(&self) -> (u16,u32,Decimal,String,bool,u8) {
            let mut digits = Vec::new();
            let mut seed = Runtime::generate_uuid();
            while seed > 9 {
                digits.push(seed % 10);
                seed = seed / 10
            }
            digits.push(seed);
            digits.reverse();
            let pattern: u16 = format!("{}{}{}", digits[0], digits[1], digits[2]).parse().unwrap();

            let rarity: u32 = format!("{}{}{}{}{}{}{}", digits[3], digits[4], digits[5], 
                digits[6], digits[7], digits[8], digits[9]).parse().unwrap();

            let float_format: i128 = format!("{}{}{}{}{}{}{}{}{}{}", digits[10], digits[11], digits[12], 
                digits[13], digits[14], digits[15], digits[16], digits[17], digits[18], digits[19]).parse().unwrap();
            let float_dec: Decimal = float_format.into();
            let big_i: i128 = 9999999999;
            let big_decimal: Decimal = big_i.into();
            let float = float_dec / big_decimal;
            let wear = match float_format {
                0..=0699999999 => String::from("Factory New"),
                0700000000..=1499999999 => String::from("Minimal Wear"),
                1500000000..=3799999999 => String::from("Field Tested"),
                3800000000..=4499999999 => String::from("Well Worn"),
                4500000000..=9999999999 => String::from("Battle Scared"),
                _ => String::from("Error"),
            };
            let stattrak = match digits[22] {
                0..=4 => true,
                _ => false,
            };
            let mut counter: usize = 23;
            let mut counter2: usize = 24;
            loop {
                let point23 = digits[counter];
                let point24 = digits[counter2];
                counter += 2;
                counter2 += 2;
                let random_24: u128 = format!("{}{}", point23, point24).parse().unwrap();
                let choice = match random_24 {
                    00..=03 => 1,
                    04..=07 => 2,
                    08..=11 => 3,
                    12..=15 => 4,
                    16..=19 => 5,
                    20..=23 => 6,
                    24..=27 => 7,
                    28..=31 => 8,
                    32..=35 => 9,
                    36..=39 => 10,
                    40..=43 => 11,
                    44..=47 => 12,
                    48..=51 => 13,
                    52..=55 => 14,
                    56..=59 => 15,
                    60..=63 => 16,
                    64..=67 => 17,
                    68..=71 => 18,
                    72..=75 => 19,
                    76..=79 => 20,
                    80..=83 => 21,
                    84..=87 => 22,
                    88..=91 => 23,
                    92..=95 => 24,
                    96..=99 => 25,
                    _ => 25,
                };
                if choice != 25 {
                    return (pattern,rarity,float,wear,stattrak,choice)
                }
                else {
                    continue
                }
            };
        }
    }
}
