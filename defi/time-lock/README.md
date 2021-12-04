# Time Lock example

How to test:

## build component

1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `resim publish .` -> save package id into $package
1. `resim new-account` -> save address into $acc1 and public key into $pub1

Create the time lock component with 0.5% fee
1. `resim call-function $package TimeLock make_timelock 0.5` -> save component address into $component
1. `resim call-method $component new_user` -> save user ref into $user

## lock XRD
1. `resim call-method $component lock 1,$user 100,$xrd 100` - lock 100 XRD of user with duration added to the current epoch 

## Add more XRD to existing lock
1. `resim call-method $component add 1,$user 100,$xrd` - add 100 XRD in addition to already locked amount 

## Get locked XRD
1. `resim call-method $component locked 1,$user` - returns locked amount value and the release epoch

## Release locked XRD
1. `resim set-current-epoch 101` - increase current epoch to be able to release locked XRD
1. `resim call-method $component release 1,$user 99.5` - we release available XRD from the lock, partial or the entire amount

