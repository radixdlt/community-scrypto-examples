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
    pub payment: Vault
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
            self.payment.amount() == 0.into() || self.payment.amount() < self.price
        } else {
            self.purse.is_empty()
        }
    }

    pub fn is_market_order(&self) -> bool {
        self.price == 0.into()
    }

    pub fn is_buy_order(&self) -> bool {
        self.buy
    }

    pub fn is_sell_order(&self) -> bool {
        !self.is_buy_order()
    }
}

#[derive(NftData)]
pub struct OrderTicket {
  pub order_number: i64,
  pub order_token_address: String,
  pub order_currency: String
}

#[derive(Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct MarketPrices {
    asset_prices: HashMap<String, Decimal>
}

#[allow(dead_code)]
impl MarketPrices {
    pub fn new() -> MarketPrices {
        MarketPrices { asset_prices: HashMap::new() }
    }

    pub fn assets(&self) -> Vec<String> {
        self.asset_prices.keys().cloned().collect()
    }

    pub fn get(&self, asset_symbol: String) -> Option<Decimal> {
        self.asset_prices.get(&asset_symbol).cloned()
    }

    pub fn for_address(&self, address: Address) -> Option<Decimal> {
        let resource_def = ResourceDef::from(address);
        let asset_symbol = resource_def.metadata()["symbol"].clone();

        self.get(asset_symbol)
    }

    pub fn update(&mut self, asset_symbol: String, price: Decimal) {
        self.asset_prices.insert(asset_symbol, price);
    }
}
