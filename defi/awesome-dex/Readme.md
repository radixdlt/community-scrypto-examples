Steps to run:

1. Do a resim reset:
```
    resim reset
```

2. Create an account and store account_address:
``` 
   export account_address=$(resim new-account | sed -n 's/.*Account component address: //p')
```

3. Publish the package and store package_address:
```
   export package_address=$(resim publish . | sed -n 's/.*Success! New Package: //p')
```

4. Create token_a and token_b resources with 10_000 tokens for each.
```
    export token_a=$(resim new-token-fixed 10000 | sed -n 's/.*Resource: //p')
    export token_b=$(resim new-token-fixed 10000 | sed -n 's/.*Resource: //p')
```

5. Now you can use `instantiate_dex.rtm` to instantiate the DEX which provides some default liquidity of 1000 `token_a` plus 50 `token_b`. Modify the amounts as you like.

use:

```
    resim run instantiate_dex.rtm
```

5. Run Instantiate.rtm file (takes 1000  units of token_a and 50 units of token_b and create a liquidity pool with them, gives LP token back. also create a large supply of Awesome tokens (incentives to be paid out to people who Swap)

```
    resim run instantiate_dex.rtm
```

Note in the output of step above set values of component_address and lp_token. component_address is obvious under new entities. "lp_token" is the  resource address corresponding to the second last resource. 

6. Example: Run swap.rtm to swap 10 units of token_a. Remember: component_address and lp_token variables must be set from the step above

```
   resim run swap.rtm
```
   
7. Example: Add liquidity and Remove Liquidity. Note: checkout the files for details, these are just example amounts of liquidity being added or removed.

```
    resim run add_liquidity.rtm
    resim run remove_liquidity.rtm
```