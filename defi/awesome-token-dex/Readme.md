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