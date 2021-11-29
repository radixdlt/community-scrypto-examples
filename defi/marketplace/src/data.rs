use scrypto::prelude::*;

#[derive(Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct BuyOrder {
  /// Order number (starting at 1)
  pub number: i64,
  /// Kind of token that is being bought
  pub token: ResourceDef,
  /// Price (in market's currency) the buyer is willing to bid
  pub bid: Decimal,
  /// Vault holding the purchased tokens
  pub purchase: Vault,
  /// Vault from which the payment for any purchases will be withdrawn
  /// (must be in market's currency)
  pub payment: Vault,
  /// The ticket resource address is used to confirm that a used ticket is valid
  pub ticket_resource_address: Address
}

#[allow(dead_code)]
impl BuyOrder {
  pub fn token_symbol(&self) -> String {
    self.token.metadata()["symbol"].clone()
  }

  pub fn currency(&self) -> String {
    self.payment.resource_def().metadata()["symbol"].clone()
  }

  pub fn is_filled(&self) -> bool {
    self.payment.amount() < self.bid
  }
}

#[derive(Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct SellOrder {
  /// Order number (starting at 1)
  pub number: i64,
  /// Kind of token being sold
  pub token: ResourceDef,
  /// Price (in market's currency) the seller is asking for
  pub ask: Decimal,
  /// Vault holding the (reserved) tokens to be sold
  pub sale: Vault,
  /// Vault into which the payment for any sales will be deposited
  /// (must be in market's currency)
  pub payment: Vault,
  /// The ticket resource address is used to confirm that a used ticket is valid
  pub ticket_resource_address: Address
}

#[allow(dead_code)]
impl SellOrder {
  pub fn token_symbol(&self) -> String {
    self.token.metadata()["symbol"].clone()
  }

  pub fn currency(&self) -> String {
    self.payment.resource_def().metadata()["symbol"].clone()
  }

  pub fn is_filled(&self) -> bool {
    self.sale.is_empty()
  }
}
