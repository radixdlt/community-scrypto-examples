# Time Lock example

How to test:

## build component

1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `resim publish .` -> save package id into $package
1. `resim new-account` -> save address into $acc1 and public key into $pub1

Create the time lock component with 0.5% fee
1. `resim call-function $package TimeLock new 0.5` -> save component address into $component

## lock XRD
1. `resim call-method $component lock 100,$xrd 100` - lock 100 XRD of user with duration added to the current epoch. Mints one TL badge with corresponding metadata -> save badge ref into $locked1

## Release locked XRD
1. `resim set-current-epoch 101` - increase current epoch to be able to release locked XRD
1. `resim run release.rtm`