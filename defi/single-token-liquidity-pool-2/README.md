
# Single Token Liquidity Pool

This package provide a Blueprint of a single token liquidity or staking pool which can reward liquidity provider or staker. It uses AMM liquidity pool token mechanics to ensure fair distribution of collected fees or reward based on when liquidity provider deposit or remove tokens.
## Function
```
new(tokens: Bucket, lp_symbol: String, lp_name: String) -> (ComponentAddress,Bucket)
```
Creates pool component with an initial supply and details for a Liquidity Provider (LP) token creation. LP token will represent share in the pool.

## Methods
### add_liquidity
```
add_liquidity(&mut self, tokens: Bucket) -> Bucket
```
Add liquidity to the pool and mint LP tokens returned to the LP
### remove_liquidity
```
remove_liquidity(&mut  self, lp_tokens:  Bucket) ->  Bucket
```
Remove liquidity share corresponding to the provided LP tokens. LP Tokens will be burn.
### add_collected_fee
```
add_collected_fee(&mut self, tokens: Bucket)
```
Add token to the pool without LP Token minting. This method should be call from other component to add "revenue" to the pool. Like fee collected after utilization of the liquidity available in the pool or staking rewards.
