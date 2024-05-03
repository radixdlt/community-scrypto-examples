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
1. `resim call-method $component burn_policies $policy1`

## Org Supporting Methods
Deposit more XRD to the assets
1. `resim call-method $component deposit 100,$xrd`
Withdraw some free assets 
1. `resim call-method $component withdraw 50`
Get free assets
1. `resim call-method $component assets`
Get locked assets
1. `resim call-method $component locked`


# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.