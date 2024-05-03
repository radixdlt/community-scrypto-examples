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