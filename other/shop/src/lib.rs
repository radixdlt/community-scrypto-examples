use scrypto::prelude::*;

blueprint! {
    struct Shop {
        // Shops has fruits, groceries
        fruits_vault: Vault,
        groceries_vault: Vault,
    }

    impl Shop {
        // The function responsible for init, "opening" a localshop in this little example
        pub fn open_shop() -> ComponentAddress {
            // We will have one fruit bucket with a supply of 20
            let fruits_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "FruitBasket")
                .metadata("symbol", "FB")
                .initial_supply(20);

            // We will have one grocery bucket with a supply of 50
            let groceries_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "GroceryBasket")
                .metadata("symbol", "GB")
                .initial_supply(50);

            // Instantiate a Shop Component with all the stuff you can buy there
            Self {
                fruits_vault: Vault::with_bucket(fruits_bucket),
                groceries_vault: Vault::with_bucket(groceries_bucket),
            }
            .instantiate()
            .globalize()
        }

        // We can only buy one fruit from the shop
        pub fn buy_a_fruit(&mut self) -> Bucket {
            info!("Current supply in the shop: {} You bought one from us", self.fruits_vault.amount());
            self.fruits_vault.take(1)
        }

        // You can get 2 groceries from the shop
        pub fn take_two_groceries(&mut self) -> Bucket {
            info!("We have {} groceries left. Take two!", self.groceries_vault.amount());
            self.groceries_vault.take(2)
        }
    }
}
