use scrypto::prelude::*;
use std::convert::TryFrom;

blueprint! {
  struct AccumulatingVault {
    badge_def: ResourceDef,
    mint_badge: Vault,
    vault: Vault,
    rate: Decimal,
    last_update: u64
  }

  impl AccumulatingVault {
    pub fn new(rate: Decimal) -> (Component, Bucket) {
      let mint_badge = Vault::with_bucket(ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1));
      let badge_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).flags(MINTABLE).badge(mint_badge.resource_def(), MAY_MINT).no_initial_supply();
      let vault_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM).flags(MINTABLE).badge(mint_badge.resource_def(), MAY_MINT).no_initial_supply();
      let user_badge = mint_badge.authorize(|b| badge_def.mint(1, b));
      let component = Self {
          badge_def: badge_def,
          mint_badge,
          vault: Vault::new(vault_def),
          rate,
          last_update: Context::current_epoch()
      }
      .instantiate();
      (component, user_badge)
    }
    
    fn update(&mut self) {
      info!("previous update: {:?}", self.last_update);
      let prev = self.last_update;
      self.last_update = Context::current_epoch();
      info!("current update: {:?}", self.last_update);
      let time_passed = Decimal::try_from(self.last_update - prev).unwrap();
      let res_def = self.vault.resource_def();
      let bucket = self.mint_badge.authorize(|b| { res_def.mint(time_passed * self.rate, b) });
      self.vault.put(bucket);
    }
    
    pub fn refresh(&mut self) -> Decimal {
      self.update();
      self.vault.amount()
    }
    
    #[auth(badge_def)]
    pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
      self.update();
      self.vault.take(amount)
    }
  }
}
