# Multi signature transaction maker

## How to test

1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `scrypto build`
1. `resim publish .` -> save package into $package

In this example, we want $acc1 and $acc2 to approve a transaction of 1 000 000 XRD to $acc3

1. Create 3 accounts `resim new-account` -> Save public key in $pub1, $pub2 and $pub3. Save addresses in $acc1, $acc2 and $acc3
1. Create the component: `resim call-function $package MultiSigMaker new 2 2 $acc3 1000000,$xrd` -> save component into $component, and the second ResourceDef into $badge
1. Send one of the two badges to acc2: `resim transfer 1 $badge $acc2`
1. Approve the transaction as $acc1: `resim call-method $component approve 1,$badge`
1. Switch to acc2: `resim set-default-account $acc2 $pub2`
1. Approve the transaction as $acc2: `resim call-method $component approve 1,$badge`