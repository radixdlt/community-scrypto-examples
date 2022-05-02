# Multi signature transaction maker

## How to test

1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `scrypto build`
1. `resim publish .` -> save package into $package

In this example, we want $acc1 and $acc2 to approve a transaction of 1 000 000 XRD to $acc3

1. Create 3 accounts `resim new-account` -> Save addresses in $acc1, $acc2 and $acc3. Save private key in $priv1, $priv2, $priv3.
1. Create the component: `resim call-function $package MultiSigMaker new 2 2 $acc3 1000000,$xrd` -> save component into $component, and the second Resource Address into $badge
1. Send one of the two badges to acc2: `resim transfer 1 $badge $acc2`
1. Approve the transaction as $acc1: `resim call-method $component approve 1,$badge`
1. Switch to acc2: `resim set-default-account $acc2 $priv2`
1. Approve the transaction as $acc2: `resim call-method $component approve 1,$badge`
1. Now account 3 should have received the tokens: `resim show $acc3`