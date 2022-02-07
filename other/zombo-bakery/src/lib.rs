use scrypto::prelude::*;

blueprint! {
    struct Hello {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault
    }

    impl Hello {
        // Implement the functions and methods which will manage those resources and data
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new() -> Component {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let my_bucket: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .initial_supply_fungible(1000);

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                sample_vault: Vault::with_bucket(my_bucket)
            }
            .instantiate()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn free_token(&mut self) -> Bucket {
            info!("My balance is: {} HelloToken. Now giving away a token!", self.sample_vault.amount());
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            self.sample_vault.take(1)
        }
    }
}
use scrypto::prelude::*;

#[derive(NftData)]
pub struct Baker {
    
    #[scrypto(mutable)]
    weight:u8,
}

blueprint! {
    struct HelloNft {
        baker_nft_minter:Vault,
        baker_nft_def:ResourceDef,
        baker_nft_price:Decimal,
        baker_card_id_counter:u128,
        bread:ResourceDef,
        butter:ResourceDef,
        cake_pan:ResourceDef,
        collected_xrd:Vault,
        flour:ResourceDef,
        material_price:Decimal,
        material_minter:Vault,
        water:ResourceDef,
        yeast:ResourceDef,
        zombo:Vault,
        
    }

    impl HelloNft {
        pub fn new() -> Component {

            let nft_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Baker NFT Minter")
                .initial_supply_fungible(1);

            let baker_nft_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Baker NFT")
                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    nft_minter.resource_def(),
                    MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
                )
                .no_initial_supply();

            let material_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Material Minter")
                .initial_supply_fungible(1);

            let bread_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Bread")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let butter_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Butter")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let cake_pan_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Cake Pan")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let flour_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Flour")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            
            let water_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Water")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let yeast_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Yeast")
                .flags(MINTABLE | BURNABLE)
                .badge(material_minter.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            let zombo_def: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Zombo")
                .metadata("symbol", "ZBO")
                .initial_supply_fungible(10000);

            Self{
                baker_nft_minter:Vault::with_bucket(nft_minter),
                baker_nft_def,
                baker_nft_price:50.into(),
                baker_card_id_counter:0,
                bread:bread_def,
                butter:butter_def,
                cake_pan:cake_pan_def,
                collected_xrd:Vault::new(RADIX_TOKEN),
                flour:flour_def,
                material_price:100.into(),
                material_minter:Vault::with_bucket(material_minter),
                water:water_def,
                yeast:yeast_def,
                zombo:Vault::with_bucket(zombo_def),
                
            }

            .instantiate()      

        }

        pub fn mint_nft(&mut self, payment: Bucket) -> (Bucket, Bucket, Bucket) {
            // Take NFT price out of the payment bucket and place in XRD vault 
            self.collected_xrd.put(payment.take(self.baker_nft_price));

            // Mint Baker NFT
            let new_card = Baker {
                weight: 1.into(),
            };

            let nft_bucket = self.baker_nft_minter.authorize(|auth| {
                self.baker_nft_def
                    .mint_nft(self.baker_card_id_counter, new_card, auth)
            });
            self.baker_card_id_counter += 1;

            //Free zombo with purchase of NFT
            let zombo_bucket = self.zombo.take(2000);

            // Return the NFT, change, and zombo
            (nft_bucket, payment, zombo_bucket)
        }
        
        //Need NFT to purchase flour
        #[auth(baker_nft_def)]
        //Buy some organic whole wheat flour
        pub fn buy_flour(&self, payment:Bucket) -> (Bucket,Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("{}", "Enjoy the flour!");
            //return flour and remaining zombo
            (self.material_minter.authorize(|auth| self.flour.mint(1, auth)),payment)
        }

        //Need NFT to purchase water
        #[auth(baker_nft_def)]
        //Buy some distilled water
        pub fn buy_water(&self, payment:Bucket) -> (Bucket,Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("{}", "Enjoy the water!");
            //return water and remaining zombo
            (self.material_minter.authorize(|auth| self.water.mint(1, auth)),payment)
        }

        //Need NFT to purchase yeast
        #[auth(baker_nft_def)]
        //Buy some Fleischmann's bakers yeast
        pub fn buy_yeast(&self, payment:Bucket) -> (Bucket, Bucket) {
            //take payment in zombo
            self.zombo.put(payment.take(self.material_price));
            info!("{}", "Enjoy the yeast!");
            //return water and remaining zombo
            (self.material_minter.authorize(|auth| self.yeast.mint(1, auth)),payment)
        }

        //Combine all ingredients in bread maker
        //Need flour water and yest to make bread
        pub fn make_bread(&self, flour:Bucket, water:Bucket, yeast:Bucket) -> Bucket {
            assert!(water.resource_def() == self.water, "That aint flour!");
            assert!(flour.resource_def() == self.flour, "That aint water!");
            assert!(yeast.resource_def() == self.yeast, "That aint yeast!");

            //burn flour water and yeast and mint bread
            self.material_minter.authorize(|auth| self.water.burn_with_auth(water, auth));
            self.material_minter.authorize(|auth| self.flour.burn_with_auth(flour, auth));
            self.material_minter.authorize(|auth| self.yeast.burn_with_auth(yeast, auth));
            info!("{}", "Enjoy your bread, careful it's hot!!!");
            self.material_minter.authorize(|auth| self.bread.mint(1, auth))
        }

        //Breads always better with butter
        //Need bread to purchase butter
        #[auth(bread)]
        pub fn buy_butter(&self,payment:Bucket)->(Bucket,Bucket){
            info!("{}", "Bread and butter is the best!");
            (self.material_minter.authorize(|auth| self.butter.mint(1, auth)),payment)
        }

        //Enjoy your bread
        //Need bread butter and NFT to eat bread
        pub fn eat_bread(&self, bread:Bucket, butter:Bucket, nft_bucket:Bucket)->Bucket{
            assert!(bread.resource_def() == self.bread, "That aint bread!");
            assert!(butter.resource_def() == self.butter, "That aint bread!");
            assert!(
                nft_bucket.amount() == 1.into(),
                "You can only feed Qty. 1 NFT at a time!"
            );
            let nft_id = nft_bucket.get_nft_ids()[0];

            let mut nft_data: Baker = nft_bucket.get_nft_data(nft_id);
            nft_data.weight += 1;

            self.baker_nft_minter
                .authorize(|auth| nft_bucket.update_nft_data(nft_id, nft_data, auth));

            self.material_minter.authorize(|auth| self.bread.burn_with_auth(bread, auth));
            self.material_minter.authorize(|auth| self.butter.burn_with_auth(butter, auth));
            info!("{}", "Damn, that's some good bread!");

            nft_bucket

        }
        //Now that you are sick of bread, lets make somthing else
        //Need NFT weight >= 3 
        pub fn next_level(&self, nft_bucket:Bucket) ->  (Bucket, Bucket) {
            let nft_id = nft_bucket.get_nft_ids()[0];
            let nft_data: Baker = nft_bucket.get_nft_data(nft_id);
            //check NFT weight
            assert!(nft_data.weight >= 3, "Eat more bread, your too skinny");
            info!("{}", "We will now be making cake, here is your free cake pan!");
            //mint cake pan 
            (self.material_minter.authorize(|auth| self.cake_pan.mint(1, auth)),nft_bucket)

        }
        
    }
}
