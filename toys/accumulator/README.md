# scryptoAccumulator

radix scrypto blueprint for accumulating vault

# what it does

the creator of the instance receive a badge which allow him to withdraw token from the vault. The vault is accumulating token at a rate specified during the creation of the instance.

# test

`resim reset`

`resim new-account`

keep the account addr in an env variable (`$account`)

`resim publish .`

keep the package addr in a env variable (`$package`)

`resim set-current-epoch 0`

`resim call-function $package AccumulatingVault new 1.0`

keep the instance addr in a env variable (`$component`)
keep also the second resource definition, that's the badge reference to access the vault (`$badge`)
`resim set-current-epoch 10`

`resim call-method $component withdraw 3.0 1,$badge`

if you do `resim show $component` you should see that the vault has now 7 tokens

if you do `resim show $account` you should see that the account has now 3 tokens

