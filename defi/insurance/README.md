# Insurance example

How to test:

## build component
1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `resim publish .` -> save package id into $package
1. `resim new-account` -> save address into $acc1 and public key into $pub1

Create the Insurance component with 100 XRD base assets
1. `resim call-function $package Insurance new 100,$xrd` -> save component address into $component
This will also create and return an org badge

## Create policy
Create policy with following params:
- type "property", 
- 10 coverage (amount of XRD that will be taken if the insurance case happen), 
- 5 price (amount of XRD will be taken on purchase), 
- 100 duration (will be added to the epoch and the purchase time)
- 3 supply (amount of generated policy badges)
1. `resim call-method $component make_policy "property" 10 5 100 3` -> save badge ref into $policy1
This will lock free assets in purpose to cover payments for all created policies 

## Purchase policy
Purchase specific policy by address, with an insurer address and with XRD
1. `resim call-method $component purchase $policy1 $acc1 5,$xrd`
This will assign a new vault with the bucket of IPB for the insurer (supply of badges is equal to the policy coverage) 

## Approve payment 
When an insurance case happen, the org can approve policy payment to the insurer.
It could be the entire coverage or a portion
Purchased policy should not be expired
1. `resim call-method $component approve $acc1 $policy1 10`

## Burn expired purchases
When purchased policy is expired, the org can release locked XRD that weren't approved
1. `resim set-current-epoch 101` - increase current epoch so the purchases become expired
1. `resim call-method $component burn_purchases $acc1 $policy1`

Unlocked XRD will be added to the org assets for withdrawal or making new policies 

## Burn unsold policies
The org may need to release locked assets by burning unsold policies
1. `resim call-method $component burn_policies $acc1`

## Org Supporting Methods
Deposit more XRD to the assets
1. `resim call-method $component deposit 100,$xrd`
Withdraw some free assets 
1. `resim call-method $component withdraw 50`
Get free assets
1. `resim call-method $component assets`
Get locked assets
1. `resim call-method $component locked`

