use scrypto::prelude::*;

#[derive(Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct Order {
    /// Order number (starting at 1)
    pub number: i64,
    /// True if this is a buy order, false if it is a sell order
    pub buy: bool,
    /// Kind of token that is being bought or sold
    pub token: ResourceDef,
    /// Price (in market's currency) the buyer is willing to bid or seller is asking
    pub price: Decimal,
    /// Vault holding the purchased (or to be sold) tokens
    pub purse: Vault,
    /// Vault from which the payment for any purchases or sales will be withdrawn
    /// (must be in market's currency)
    pub payment: Vault,
    /// The ticket resource address is used to confirm that a used ticket is valid
    pub ticket_resource_address: Address
}

#[allow(dead_code)]
impl Order {
    pub fn token_symbol(&self) -> String {
        self.token.metadata()["symbol"].clone()
    }

    pub fn currency(&self) -> String {
        self.payment.resource_def().metadata()["symbol"].clone()
    }

    pub fn is_filled(&self) -> bool {
        if self.buy {
            self.payment.amount() < self.price
        } else {
            self.purse.is_empty()
        }
    }
}
