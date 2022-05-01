use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Baker {
    
    #[scrypto(mutable)]
    weight:u8,
}

blueprint! {
    struct ZomboBakery {
        baker_nft_minter:Vault,
        baker_nft_resource_address:ResourceAddress,
        baker_nft_price:Decimal,
        baker_card_id_counter:u64,
        bread:ResourceAddress,
        butter:ResourceAddress,
        cake_pan:ResourceAddress,
        collected_xrd:Vault,
        flour:ResourceAddress,
        material_price:Decimal,
        material_minter:Vault,
        water:ResourceAddress,
        yeast:ResourceAddress,
        zombo:Vault,
    }

    impl ZomboBakery {
        pub fn new(baker_nft_price:Decimal, material_price:Decimal) -> ComponentAddress {

            let nft_minter = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Baker NFT Minter")
                .initial_supply(1);

            let baker_nft_resource_address = ResourceBuilder::new_non_fungible()
                .metadata("name", "Baker NFT")
                .mintable(rule!(require(nft_minter.resource_address())), LOCKED)
                .burnable(rule!(require(nft_minter.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(nft_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let material_minter = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Material Minter")
                .initial_supply(1);

            let bread_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Bread")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let butter_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Butter")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let cake_pan_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Cake Pan")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let flour_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Flour")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();
            
            let water_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Water")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let yeast_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Yeast")
                .mintable(rule!(require(material_minter.resource_address())), LOCKED)
                .burnable(rule!(require(material_minter.resource_address())), LOCKED)
                .no_initial_supply();

            let zombo_resource_address: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Zombo")
                .metadata("symbol", "ZBO")
                .initial_supply(10000);

            let access_rules = AccessRules::new()
                .method("buy_flour", rule!(require(baker_nft_resource_address)))
                .method("buy_water", rule!(require(baker_nft_resource_address)))
                .method("buy_yeast", rule!(require(baker_nft_resource_address)))
                .method("buy_butter", rule!(require(bread_resource_address)))
                .default(rule!(allow_all));

            Self{
                baker_nft_minter:Vault::with_bucket(nft_minter),
                baker_nft_resource_address,
                baker_nft_price:baker_nft_price,
                baker_card_id_counter:0,
                bread:bread_resource_address,
                butter:butter_resource_address,
                cake_pan:cake_pan_resource_address,
                collected_xrd:Vault::new(RADIX_TOKEN),
                flour:flour_resource_address,
                material_price:material_price,
                material_minter:Vault::with_bucket(material_minter),
                water:water_resource_address,
                yeast:yeast_resource_address,
                zombo:Vault::with_bucket(zombo_resource_address),
            }
            .instantiate()   
            .globalize()   
        }


        pub fn mint_nft(&mut self, mut payment: Bucket) -> (Bucket, Bucket, Bucket) {
            // Take NFT price out of the payment bucket and place in XRD vault 
            self.collected_xrd.put(payment.take(self.baker_nft_price));

            // Mint Baker NFT
            let new_card = Baker {
                weight: 1.into(),
            };

            let nft_bucket = self.baker_nft_minter.authorize(|| {
                borrow_resource_manager!(self.baker_nft_resource_address)
                    .mint_non_fungible(&NonFungibleId::from_u64(self.baker_card_id_counter), new_card)
            });
            self.baker_card_id_counter += 1;

            //Free zombo with purchase of NFT
            let zombo_bucket = self.zombo.take(2000);

            // Return the NFT, change, and zombo
            (nft_bucket, payment, zombo_bucket)
        }
        
        //Need NFT to purchase flour
        //Buy some organic whole wheat flour
        pub fn buy_flour(&mut self, mut payment:Bucket) -> (Bucket,Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("Enjoy the flour!");
            //return flour and remaining zombo
            (self.material_minter.authorize(|| borrow_resource_manager!(self.flour).mint(1)),payment)
        }

        //Need NFT to purchase water
        //Buy some distilled water
        pub fn buy_water(&mut self, mut payment:Bucket) -> (Bucket,Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("Enjoy the water!");
            //return water and remaining zombo
            (self.material_minter.authorize(|| borrow_resource_manager!(self.water).mint(1)),payment)
        }

        //Need NFT to purchase yeast
        //Buy some Fleischmann's bakers yeast
        pub fn buy_yeast(&mut self, mut payment:Bucket) -> (Bucket, Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("Enjoy the yeast!");
            //return water and remaining zombo
            (self.material_minter.authorize(|| borrow_resource_manager!(self.yeast).mint(1)),payment)
        }

        //Combine all ingredients in bread maker
        //Need flour water and yest to make bread
        pub fn make_bread(&mut self, flour:Bucket, water:Bucket, yeast:Bucket) -> Bucket {
            assert!(water.resource_address() == self.water, "That aint water!");
            assert!(water.amount() > Decimal::zero(), "No water provided!");
            assert!(flour.resource_address() == self.flour, "That aint flour!");
            assert!(flour.amount() > Decimal::zero(), "No flour provided!");
            assert!(yeast.resource_address() == self.yeast, "That aint yeast!");
            assert!(yeast.amount() > Decimal::zero(), "No yeast provided!");

            //burn flour water and yeast and mint bread
            self.material_minter.authorize(|| water.burn());
            self.material_minter.authorize(|| flour.burn());
            self.material_minter.authorize(|| yeast.burn());
            info!("Enjoy your bread, careful it's hot!!!");
            self.material_minter.authorize(|| borrow_resource_manager!(self.bread).mint(1))
        }

        //Breads always better with butter
        //Need bread to purchase butter
        pub fn buy_butter(&mut self,payment:Bucket)->(Bucket,Bucket){
            info!("Bread and butter is the best!");
            (self.material_minter.authorize(|| borrow_resource_manager!(self.butter).mint(1)),payment)
        }

        //Enjoy your bread
        //Need bread butter and NFT to eat bread
        pub fn eat_bread(&mut self, bread:Bucket, butter:Bucket, baker_nft:Proof) {
            assert!(bread.resource_address() == self.bread, "That aint bread!");
            assert!(bread.amount() > Decimal::zero(), "No bread provided!");
            assert!(butter.resource_address() == self.butter, "That aint butter!");
            assert!(butter.amount() > Decimal::zero(), "No butter provided!");
            assert!(
                baker_nft.amount() == Decimal::one(),
                "You can only feed Qty. 1 NFT at a time!"
            );

            let mut nft_data: Baker = baker_nft.non_fungible::<Baker>().data();
            nft_data.weight += 1;

            self.baker_nft_minter.authorize(|| baker_nft.non_fungible().update_data(nft_data));
            self.material_minter.authorize(|| bread.burn());
            self.material_minter.authorize(|| butter.burn());
            info!("Damn, that's some good bread!");

            baker_nft.drop()

        }
        //Now that you are sick of bread, lets make somthing else
        //Need NFT weight >= 3 
        pub fn next_level(&mut self, baker_nft:Proof) ->  Bucket {
            let nft_data: Baker = baker_nft.non_fungible().data();
             //check NFT weight
            assert!(nft_data.weight >= 3, "Eat more bread, your too skinny");
            info!("We will now be making cake, here is your free cake pan!");
             //mint cake pan 
            baker_nft.drop();
            self.material_minter.authorize(|| borrow_resource_manager!(self.cake_pan).mint(1))

        }
        
    }
}
