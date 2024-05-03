# Krulkswap simple exchange
This is a Blueprint for a simple object oriented DEX in Scrypto.
<h2> Warning! This is a simple example of scrypto code, and is not meant to be used in production as is.</h2> <br>

The structure of this project consists of one main component which keeps track of multiple different liquidity pool components. This main component is the component that you use when making calls to the exchange. The liquidity pool components are instantiated by the main component when the `add_pool()` method is called. These sattelite components are owned by the main component.

It swaps using the very standard and simple `k = a * b` equation used in a lot of decentralized exchanges.
You cannot create multiple pools for a token pair. All liquidity for a token pair is aggregated into a single pool.

# Instantiation
To instantiate a krulkswap component, first publish this package using the Radix dashboard.

Then, you can create an instance of the krulkswap component using this transaction manifest:

```js
CALL_FUNCTION PackageAddress("[exchange_package]") "Exchange" "new_exchange";
```

Where `[Exchange_package]` Should be the address of the exchange package you got from publishing the blueprint.

Then, to create a pool component, try to send this transaction to the network.

```js
CALL_METHOD ComponentAddress("[account_address]") "withdraw" Decimal("[amount]") ResourceAddress("[token_a_address]");

CALL_METHOD ComponentAddress("[account_address]") "withdraw" Decimal("[amount]") ResourceAddress("[token_b_address]");

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("[exchange_component]") "add_pool";

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("[recipient_account_address]") "deposit_batch";
```
Where: <br>
 * `[account_address]` should be the address of the account component of your own radix testing wallet.

 * `[amount]` should be the amount of tokens you would like to use of each type.

 * `[token_a_address]` and `[token_b_address]` should be the token addresses for the tokens you would like to use from your wallet to create a pool respectively.

 * `[exchange_component]` is the component address of the main exchange component, obtained from the instantiation of the component using the `new_exchange()` function.

 * `[recipient_account_address]` should be the account address where the LP tokens obtained from creating the pool are deposited.

Now, the exchange is set up with one pool. You can add more pools for other token pairs or try to swap now.

The code contains plenty of comments for you to understand what's going on. Have fun :)

Please report any mistakes I may have made. I did not have the opportunity to thoroughly test this blueprint.

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