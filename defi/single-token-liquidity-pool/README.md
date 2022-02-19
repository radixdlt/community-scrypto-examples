# Single Token Liquidity Pool
Pool where users can contribute tokens in exchange of LP tokens.
The component can collect fees that will be given when removing the liquidity.

## Methods

### add_liquidity(bucket: Bucket) -> Bucket
Allow a user to add liquidity to the pool in exchange of a share of the LP tokens.

Returns the share of LP tokens

### remove_liquidity(bucket: Bucket) -> Bucket
Allow a user to remove its liquidity and collected fees from the pool. 

Returns a bucket containing the liquidity and fees.

### add_fees(bucket: Bucket)
Called from another component to add collected fees to the pool