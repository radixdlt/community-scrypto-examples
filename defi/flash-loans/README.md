# Flash Loans example

How to test:

## build flashloan component

1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `cd flashloan`
1. `scrypto build`
1. `resim publish .` -> save package id into $fl_package
1. `resim new-account` -> save address into $acc1 and private key into $priv1

Create the flashloan component with 900 000 XRD of funds and a 5% interest
1. `resim call-function $fl_package FlashLoan new 900000,$xrd 5` -> save component address into $flashloan

## build caller component

1. `cd ../caller`
1. `scrypto build`
1. `resim publish .` -> save package id into $caller_package
1. `resim call-function $caller_package Caller new $flashloan 10000,$xrd` -> save component address into $caller

## ask for a flashloan
1. `resim call-method $caller call $caller` - We have to send the address of the caller component since we cannot yet get it from the Context

## Testing what happend when not returning fully owed amount
Call the same command again, this time, the opportunity bucket contains 0 XRD so you are not able to pay for the loan !

