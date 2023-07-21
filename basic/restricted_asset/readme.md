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