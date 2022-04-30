use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Account {
    #[scrypto(mutable)]
    class: u8,
    exp: u128,
    health: u32,
    attack: u32,
    magic: u32,
    defense: u32,
    speed: u32,
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
         version: Decimal,
    }

    impl Substradix {
        pub fn new(game_price: Decimal, version: Decimal) -> ComponentAddress {
            // Creates system badge for minting + verifying level outcomes.
            let system_badge = ResourceBuilder::new_fungible()
                .metadata("name", "system")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // NFT for character data
            let character_nft = ResourceBuilder::new_non_fungible()
                .metadata("type", "Substradix character NFT")
                .mintable(AccessRule::AllowAll, LOCKED)
                .burnable(AccessRule::AllowAll, LOCKED)
                .updateable_non_fungible_data(rule!(require(system_badge.resource_address())), LOCKED)
                .restrict_deposit(AccessRule::DenyAll, LOCKED)
                .restrict_withdraw(AccessRule::DenyAll, LOCKED)
                .no_initial_supply();

            // Gold for ingame currency
            let token_gold: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Gold Coin")
                .mintable(rule!(require(system_badge.resource_address())), LOCKED)
                .burnable(rule!(require(system_badge.resource_address())), LOCKED)
                .initial_supply(1);
                    

            // Creates materials for upgrading  
            let _token_greavite = ResourceBuilder::new_fungible()
                .metadata("name", "Greavite Ore")
                .mintable(rule!(require(system_badge.resource_address())), LOCKED)
                .burnable(rule!(require(system_badge.resource_address())), LOCKED)
                .no_initial_supply();


            let _token_wood = ResourceBuilder::new_fungible()
                .metadata("name", "Ageis Wood")
                .mintable(rule!(require(system_badge.resource_address())), LOCKED)
                .burnable(rule!(require(system_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // Sets values for instantiation
            Self {
                system_vault: Vault::with_bucket(system_badge),
                gold_vault: Vault::with_bucket(token_gold),
                collected_xrd: Vault::new(RADIX_TOKEN),
                game_price,
                character_number: 0,
                character_nft,
                version: 1,

            }
            .instantiate()
            .globalize()
        }

        // Creates character
        pub fn create_character(&mut self,
            class: u8,
            health: u32,
            attack: u32,
            magic: u32,
            defense: u32,
            speed: u32,
            new_character: Bucket,
            mut payment: Bucket,
            ) -> (Bucket, Bucket) {

            // Prevents making alt accounts on one wallet.
            assert!(new_character.amount() == dec!(0),
                "You cannot have more than one character per account!");

            let character_data = Account {
                class: class,
                exp: 0,
                health: health,
                attack: attack,
                magic: magic,
                defense: defense,
                speed: speed,
            };

            let new_character = self.authorize(|| {
                borrow_resource_manager!(self.character_nft)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.character_number), character_data)
            });
            self.character_number += 1;
            self.collected_xrd.put(payment.take(self.game_price));
            (new_character, payment)
        }

        // Owner only, collects all XRD from sold Personal Tokens
        pub fn withdraw_xrd(&mut self) -> Bucket {
            self.collected_xrd.take_all()
        }

        // Changes price of Substradix
         pub fn change_price(&mut self, new_price: Decimal) -> Decimal {
            self.game_price = new_price;
            self.game_price
        }

        pub fn game_level(&mut self,  nft_character: Bucket, exp_gain: u128) -> Bucket {
            let seed = Runtime::generate_uuid();

            let _nft_key = nft_character.non_fungible::<Account>().id();

            let mut nft_data: Account = nft_character.non_fungible().data();
            nft_data.exp += exp_gain;

            self.system_vault.authorize(|| nft_character.non_fungible().update_data(nft_data));

            println!("Your seed is {}", seed);

            nft_character

        }

        pub fun level_1(&mut self) -> Bucket {
            let seed = Runtime::generate_uuid();
            let damage_rng = 
            let mut nft_data: Account = nft_character.non_fungible().data();
            let health: u32 = nft_data.health;
            let attack: u32 = nft_data.attack;
            let magic: u32 = nft_data.magic;
            let defense: u32 = nft_data.defense;
            let speed: u32 = nft_data.speed;
            let enemy_data = bruh;
    
            loop {
                let health = health - ((enemy_attack - defense) *  );
                let enemy_health = enemy_health - ((attack - enemy_defense) *  );

                if health <=1 {
                    println!("You Died");
                    break;
                }
  
                if enemy_health <=1 {
                    println!("Victory!");
                    nft_data.exp += 15;
                    continue;
                    

                }
            }
        }
    }
}
