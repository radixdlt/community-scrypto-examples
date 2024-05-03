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