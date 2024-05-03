## Restricted Asset

  

This Rust script defines a restricted asset implementation using the Scrypto library. The implementation allows creating and managing accounts with restricted balances while ensuring that transfers and other operations are only possible if the accounts hold valid authorization badges and follow the specified rules.

  

### Data Structures

  

**BalanceData**: A structure containing two mutable fields - max_balance and current_balance, both of type Decimal.

  

### Blueprint

  

**Struct**: The blueprint comprises three fields - component_badge, balance_res_address, and local_id_counter.

 
### functions

 
**instantiate()**: Creates a new instance of the RestrictedAsset struct, sets up component_badge with a unique authorization badge, and creates a non-fungible balance_res_address with rules that require the component_badge.

  

### Methods

  

**create_account()**: Mints a new BalanceData non-fungible token with a max_balance of 5000 and a current_balance of 0, and returns a Bucket containing the new token.

**transfert()**: Transfers an amount from one account to another, validating balance proofs and ensuring that neither the origin nor destination account's balances violate their constraints.

**deposit()**: Deposits an amount of tokens to a specified balance. It first checks if the updated balance doesn't exceed the max_balance. If the check passes, it updates the current_balance of the specified balance_id.

**withdraw()**: Withdraws an amount of tokens from a specified balance. It first checks if the current_balance is sufficient. If the check passes, it updates the current_balance of the specified balance_id.

  

**withdraw()** and **deposit()** methods are meant to be used in a controlled environment were we can relate this oparation to real world events from the user.

  

### Utility Functions

  

**_get_new_local_id()**: Increments the local_id_counter and returns its value. This is used for minting new BalanceData tokens with unique local IDs.

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