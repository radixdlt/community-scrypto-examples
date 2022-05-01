# How to test

## Setup
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `resim reset`
1. `resim new-account` -> save address in $acc1 and private key to $priv1
1. `resim new-account` -> save address in $acc2 and private key to $priv2
1. `resim set-default-account $acc2 $priv2`
1. Create a new token to test with: `resim new-token-fixed 8000000000 --description "The first Doge project on Radix" --name "DogeCube" --symbol "DGC"` -> save address in $dgc
1. `resim set-default-account $acc1 $priv1` 
1. `scrypto build`
1. `resim publish .` -> save package address in $package

## Trading DGC for XRD through the escrow
$acc1 will be account A and $acc2 will be account B. A wants to trade 500 XRD for 10 DGC

1. Instantiate the component for the trade: `resim call-function $package Escrow new $xrd $dgc $acc1 $acc2` -> save the component's address in $component, the first ResourceAddress into $badgeA and the second into $badgeB
1. Send badge B to the account with who you want to trade with: `resim transfer 1 $badgeB $acc2`
1. Send the 500 XRD from account A: `resim call-method $component put_tokens 500,$xrd 1,$badgeA`
1. `resim set-default-account $acc2 $priv2`
1. Send the 10 DGC from account B: `resim call-method $component put_tokens 10,$dgc 1,$badgeB`
1. Accept trade from account B: `resim call-method $component accept 1,$badgeB`
1. `resim set-default-account $acc1 $priv1`
1. Accept Trade from account A: `resim call-method $component accept 1,$badgeA`
1. Withdraw the DGC to account A: `resim call-method $component withdraw 1,$badgeA`
1. `resim set-default-account $acc2 $priv2`
1. Withdraw the XRD to account B: `resim call-method $component withdraw 1,$badgeB`
1. Look at the balances of both accounts, the trade should have worked ! (`resim show $acc1` and `resim show $acc2`)

## To cancel the trade (can only be done before the two parties accepted)
1. `resim call-method $component cancel 1,[badge_of_account]`
Withdraw the tokens
1. `resim call-method $component withdraw 1,$badgeA`
1. `resim set-default-account $acc2 $priv2`
`resim call-method $component withdraw 1,$badgeB`