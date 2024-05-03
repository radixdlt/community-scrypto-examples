# Token creator
This is a simple token creator package, that can be used to create a web app that creates the tokens for you. Of course this can also be done with the transaction manifest, but this component also stores users and their associated tokens on-ledger. It may be nice for creating a simple token creation app where users can see their tokens, but no database on a regular server is needed.

The structure of this project consists of one main component which keeps track of multiple users in a hashmap. This main component is the component that you use when making calls. The user components are instantiated by the main component when the `new_user()` method is called. These satellite components are owned by the main component.


# Instantiation
To instantiate a component, first publish this package using the Radix dashboard.

Then, you can create an instance of the token_creator component using transaction manifests.

for example:

```js
CALL_FUNCTION PackageAddress("[token_creator_package]") "TokenCreator" "new";
```

Where `[token_creator_package]` Should be the address of the package you got from publishing the blueprint.

Then, to create a new user, try to send this transaction to the network.

```js
CALL_METHOD ComponentAddress("[component_address]") "new_user";

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("[account_address]") "deposit_batch";
```
Where: <br>
 * `[component_address]` is the component address of the main token_creator component, obtained from the instantiation of the component using the `new()` function.

 * `[account_address]` should be your account address. A badge will be returned from the token_creator component which acts as authentication for a user.

Now, the component is set up with one user. You can add more users or try out the token creation functionality by writing some more transaction manifests.

Please report any mistakes I may have made. I did not have the opportunity to thoroughly test this blueprint, and it is not meant to be used in production as is.

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