use scrypto::prelude::*;

/* This a non-trivial example that shows off a blueprint that operates with
    with an adjustable number of vaults. My main discover was that you can
    return Vec<Bucket> and the resim methd call can handle it and update
    the account correctly.

   If someone wants to do so there are many possible extensions:
        - add purchase method(s) via XRD
            - have the purchase method(s) also return change
            - add badges to control who can get the collected XRD
        - allow for different prices for each type of candy
        - allow candy restocking by a badged account (hint: mutable supply)
        - tag the candy_vaults with the rri instead of the token symbol
        - many more...
*/

blueprint! {
    struct CandyShop {
        // The different kinds of candies are kept here with each in a tuple
        // with a unique tag string that doubles as the candies' token symbol.
        candy_vaults: Vec<(String,Vault)>
    }

    impl CandyShop {
        
        pub fn new() -> Component {
            // This constructor sets up an empty candy shop
            let tagged_vaults: Vec<(String,Vault)> = Vec::new();
            Self {
                candy_vaults: tagged_vaults
            }
            .instantiate()
        }

        pub fn initial_supply( supply_size: u32 ) -> Component {
            // This constructor sets up a variety of candies with the specified amount.
            let mut tagged_vaults: Vec<(String,Vault)> = Vec::new();
            // Now define the meta data for each type of candy.
            let mut metas = vec![("Gumball", "GUM", "The best gumball in the world.")];
            metas.push(("Jawbreaker", "JAWB", "Jawbreakers teach patience."));
            metas.push(("Lollipop","LPOP","You can't lick Lollipops!"));
            metas.push(("Candy Cane", "CANE", "Striped candy rules!"));
            metas.push(("Jelly Bean", "JELLY", "Jelly Beans are best!"));
            metas.push(("Mint Candy", "MINT", "Mints are wonderful!"));
            metas.push(("Gummy Bear", "BEAR", "Gummy Bears rules!"));
            // Create a supply 
            for tup in metas {
                let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                    .metadata("name", tup.0.to_string())
                    .metadata("symbol", tup.1.to_string())
                    .metadata("description", tup.2.to_string())
                    .initial_supply_fungible(supply_size);
                tagged_vaults.push((tup.1.to_string(), Vault::with_bucket(bucket)));
            }
            Self {
                candy_vaults: tagged_vaults
            }
            .instantiate()
        }

        fn take_from_vault(&self, symbol: String, quantity: Decimal) -> Bucket {
            // private function returns a bucket with the specified number and type of candy (or an empty bucket)
            for c in &self.candy_vaults[..] {
                if c.0 == symbol {
                    let v = &c.1;
                    if v.amount() >= quantity {
                        return v.take(quantity)
                    } else {
                        break;
                    }
                }
            }
            let empty_bucket: Bucket = Bucket::new(RADIX_TOKEN); // canonical way to make an empty_bucket
            return empty_bucket
        }

        pub fn free_gum(&self) -> Bucket {
            // Return a gumball if we have at least one available.
            // If there is no GUM vault or the GUM vaut is empty, this method will fail.
            self.take_from_vault("GUM".to_string(), 1.into())
        }

        pub fn free_samples(&mut self) -> Vec<Bucket> {
            let mut buckets = Vec::new();
            for c in &self.candy_vaults[..] {
                if c.1.amount() > 0.into() {
                    buckets.push(c.1.take(1));
                }
            }
            return buckets
        }

        fn contains(&self, symbol: &str) -> bool {
            // return True if the symbol is found in the candy_vaults based on the token symbol
            let mut found: bool = false;
            let symbol_string = symbol.to_string();
            for c in &self.candy_vaults[..] {
                if c.0 == symbol_string {
                    found = true;
                    break;
                }
            }
            return found
        }

        pub fn add_candy(&mut self, name: String, symbol: String, description: String, supply_size: Decimal) {
            assert!(supply_size >= 1.into(), "Not enough initial candy");
            assert!(self.contains(&symbol) == false, "That type of candy is already available.");
            // Add a new kind of candy to the CandyShop
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", name)
                .metadata("symbol", symbol.to_string())
                .metadata("description", description)
                .initial_supply_fungible(supply_size);
            self.candy_vaults.push((symbol, Vault::with_bucket(bucket)));
        }
    }
}
