
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


# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.