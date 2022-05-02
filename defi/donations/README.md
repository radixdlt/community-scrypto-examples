# Donations example

 How to test:

 ## build component
 1. `resim reset`
 1. `export xrd=030000000000000000000000000000000000000000000000000004`
 1. `resim publish .` -> save package id into $package
 1. `resim new-account` -> save address into $acc1 and public key into $pub1

 Create the Donations component with 2% fee
 1. `resim call-function $package Donations new 2` -> save component address into $component
 This will also create and return an admin badge

 ## Create policy
 Create badge with following params:
 - owner - The Address that will receive donations, 
 - identifier - any identifier that will help to track badges on other services, 
 - title, 
 - description
 - url
 - price 
 - supply - the amount of badges to be minted
 1. `resim call-method $component make_badge $acc1 "Level#1" "Level 1 donator" "Opens first level color" "Some URL" 5 100`

 ## Get available badges
 1. `resim call-method $component get_badges $acc1` -> save the badge address into $badge1

 ## Donate 
 Donator will pay for this badge according to the price
 The corresponding fee will be taken from the receiver bucket
 1. `resim call-method $component donate $acc1 $badge 5,$xrd `

 ## Admin Supporting Methods
 The withdraw method is the only method which is currently only allowed for the admin. All of the other methods are allowed for all users. This method requires that an admin badge be present in the auth zone for a call to be successful and to be considered authenticated. You can use the following transaction manifest instructions with the addresses of your components to perform this action.

```sh
CALL_METHOD ComponentAddress("<Account Address>") "create_proof_by_amount" Decimal("1") ResourceAddress("<Admin Badge Resource Address>");
CALL_METHOD ComponentAddress("<Component Address>") "withdraw" Decimal("<Amount to Withdraw>");
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("<Account Address>") "deposit_batch";
```