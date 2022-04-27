use scrypto::prelude::*;
use std::convert::TryFrom;

blueprint! {
  struct AccumulatingVault {
    badge_def: ResourceAddress,
    mint_badge: Vault,
    vault: Vault,
    rate: Decimal,
    last_update: u64
  }

  impl AccumulatingVault {
    pub fn new(rate: Decimal) -> (ComponentAddress, Bucket) {
      let mut mint_badge = Vault::with_bucket(
        ResourceBuilder::new_fungible()
          .divisibility(DIVISIBILITY_NONE)
          .initial_supply(1)
      );

      let user_badge = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_NONE)
        .mintable(rule!(require(mint_badge.resource_address())), LOCKED)
        .initial_supply(1);

      let vault_def = ResourceBuilder::new_fungible()
        .mintable(rule!(require(mint_badge.resource_address())), LOCKED)
        .no_initial_supply();
      
      let component = Self {
          badge_def: user_badge.resource_address(),
          mint_badge,
          vault: Vault::new(vault_def),
          rate,
          last_update: Runtime::current_epoch()
      }
      .instantiate();

      let access_rules = AccessRules::new()
        .method("update", rule!(allow_all))
        .method("refresh", rule!(allow_all))
        .method("withdraw", rule!(require(user_badge.resource_address())));

      (component.add_access_check(access_rules).globalize(), user_badge)
    }
    
    fn update(&mut self) {
      info!("previous update: {:?}", self.last_update);
      let prev = self.last_update;
      self.last_update = Runtime::current_epoch();
      info!("current update: {:?}", self.last_update);
      let time_passed = Decimal::try_from(self.last_update - prev).unwrap();
      let mut res_def = self.vault.resource_address();
      let bucket = self.mint_badge.authorize(|| { borrow_resource_manager!(res_def).mint(time_passed * self.rate) });
      self.vault.put(bucket);
    }
    
    pub fn refresh(&mut self) -> Decimal {
      self.update();
      self.vault.amount()
    }
    
    pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
      self.update();
      self.vault.take(amount)
    }
  }
}
