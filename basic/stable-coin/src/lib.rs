use scrypto::prelude::*;

// credit: thanks to Scrypto-Example/regulated-token
// admin, version, withdraw, mint, burn,
blueprint! {
    struct StableCoinVault {
        token_vault: Vault,
        auth: Vault,
        total_supply: Decimal,
        version: u16,
        admin_addr: ResourceAddress,
    }

    impl StableCoinVault {
        pub fn new(total_supply: Decimal, keys: Vec<String>, values: Vec<String>) -> (ComponentAddress, Bucket, Bucket) {
          info!("StableCoin new(): total_supply = {}", total_supply);
            // top admin
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "admin_badge")
                .burnable(rule!(allow_all), LOCKED)
                .initial_supply(3);
            // to withdraw coins
            let wd_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "withdraw badge")
                .burnable(rule!(allow_all), LOCKED)
                .initial_supply(2);

            // for minting & withdraw authority
            let auth_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "auth_badge")
                .burnable(rule!(allow_all), LOCKED)
                .initial_supply(1);
            info!("check1");

            let token_rule: AccessRule = rule!(
                require(admin_badge.resource_address())
                    || require(auth_badge.resource_address())
            );
            info!("check2");
            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(keys[0].as_str(), values[0].as_str())
                .metadata(keys[1].as_str(), values[1].as_str())
                .metadata(keys[2].as_str(), values[2].as_str())
                .metadata(keys[3].as_str(), values[3].as_str())
                .metadata(keys[4].as_str(), values[4].as_str())
                .metadata(keys[5].as_str(),values[5].as_str())
                .updateable_metadata(token_rule.clone(), token_rule.clone())
                //.updateable_non_fungible_data(...)
                //.restrict_withdraw(token_rule.clone(), token_rule.clone())
                //.restrict_deposit(token_rule.clone(), token_rule.clone())
                .mintable(token_rule.clone(), token_rule.clone())
                .burnable(token_rule.clone(), token_rule.clone())
                .initial_supply(total_supply);
            info!("check3");
            // Next we need to setup the access rules for the methods of the component
            let method_rule: AccessRules = AccessRules::new()
                .method(
                    "get_data",
                    rule!(allow_all), AccessRule::DenyAll
                )
                .default(token_rule, AccessRule::DenyAll);
            info!("check4");
            let mut component = Self {
                token_vault: Vault::with_bucket(my_bucket),
                auth: Vault::with_bucket(auth_badge),
                total_supply,
                version: 1,
                admin_addr: admin_badge.resource_address(),
            }
            .instantiate();
            component.add_access_check(method_rule);
            info!("check5");
            (component.globalize(), admin_badge, wd_badge)
        }// new()

        pub fn mint_to_bucket(&mut self, amount: Decimal) -> Bucket {
          info!("mint_to_bucket");
          assert!(amount > dec!(0), "invalid amount");
          info!("self.total_supply:{}", self.total_supply);
          self.total_supply += amount;
          info!("self.total_supply:{}", self.total_supply);
          self.auth.authorize(|| {
            borrow_resource_manager!(self.token_vault.resource_address()).mint(amount)
          })
        }
        pub fn mint_to_vault(&mut self, amount: Decimal) {
          info!("mint_to_vault");
          assert!(amount > dec!(0), "invalid amount");
          let new_tokens = borrow_resource_manager!(self.token_vault.resource_address())
          .mint(amount);
          self.token_vault.put(new_tokens);
          self.total_supply += amount;
          info!("total token amount: {}", self.token_vault.amount());
        }

        //pub fn withdraw_to_3rd_party(&self, amount: Decimal) {
          //risky... just send tokens to yourself, then deposit them into the 3rd party package!
        //}
        pub fn withdraw_to_bucket(&mut self, amount: Decimal) -> Bucket {
          info!("withdraw_to_bucket");
          //check set_withdrawable_vault()
          assert!(amount > dec!(0), "invalid amount");
          assert!(amount <= self.token_vault.amount(), "not enough amount in the vault");
          self.token_vault.take(amount)
        }
        pub fn deposit_to_vault(&mut self, bucket: Bucket) {
          self.token_vault.put(bucket);
        }

        pub fn burn_in_vault(&mut self, amount: Decimal) {
          info!("burn_in_vault");
          assert!(amount > dec!(0), "invalid amount");
          assert!(amount <= self.token_vault.amount(), "not enough amount in the vault");
          self.total_supply -= amount;
          self.token_vault.take(amount).burn();
        }
        pub fn burn_in_bucket(&mut self, bucket: Bucket)  {
          info!("burn_in_bucket");
          assert!(bucket.resource_address() == self.token_vault.resource_address(), "input token invalid");
          let amount = bucket.amount();
          self.total_supply -= amount;
          bucket.burn();
        }

        // name, symbol, icon_url, url, author, stage
        pub fn update_metadata(&mut self, key: String, value: String) {
          info!("update_metadata");
          borrow_resource_manager!(self.token_vault.resource_address()).set_metadata(key.into(), value.into());
        }//self.auth.authorize(|| {})

        pub fn set_token_stage_three(&self) {
          info!("set_token_stage_three");
          let token_rmgr: &mut ResourceManager =
          borrow_resource_manager!(self.token_vault.resource_address());

          token_rmgr.set_metadata("stage".into(), "Lock mint, withdraw, and update_metadata rules".into());
          //token_rmgr.set_withdrawable(rule!(allow_all));
          //token_rmgr.lock_withdrawable();

          token_rmgr.set_mintable(rule!(deny_all));
          token_rmgr.lock_mintable();

          token_rmgr.set_updateable_metadata(rule!(deny_all));
          token_rmgr.lock_updateable_metadata();

          // With the resource behavior forever locked, our auth no longer has any use
          // We will burn our auth badge, and the holders of the other badges may burn them at will
          // Our badge has the allows everybody to burn, so there's no need to provide a burning authority
        }

        //["name", "symbol", "icon_url", "url", "author", "stage"]
        pub fn get_token_metadata(token_addr: ResourceAddress, keys_owned: Vec<String>) -> (u8, NonFungibleIdType, Decimal, Vec<Option<String>>) {
          let manager: &mut ResourceManager = borrow_resource_manager!(token_addr);

          let mut divisibility: u8 = 255;
          let mut non_fungible_id_type = NonFungibleIdType::U32;
          match manager.resource_type() {
              ResourceType::Fungible{divisibility: div} => {
                  info!("Fungible resource with divisibility {}", div);
                  divisibility = div;
              },
              ResourceType::NonFungible { id_type: NonFungibleIdType } => {
                  info!("Non Fungible resource found with id_type: {}", NonFungibleIdType);
                  non_fungible_id_type = NonFungibleIdType;
              }
          }
          let total_supply = manager.total_supply();
          info!("Total supply: {}", total_supply);

          let mut values: Vec<Option<String>> = vec![];
          for i in keys_owned {
            values.push(manager.get_metadata(i));
          }
          (divisibility, non_fungible_id_type, total_supply, values)
        }

        pub fn get_data(&self) -> (u16, Decimal, Decimal) {
            let amount = self.token_vault.amount();
            info!("Current version: {}, vault_amount: {}, total_supply:{}", self.version, amount, self.total_supply );
            (self.version, amount, self.total_supply)
        }
        pub fn set_version(&mut self, new_version: u16) {
            info!("Current version is {}", self.version);
            assert!(self.version >= 3, "invalid version");
            self.version = new_version;
        }
    }
}
