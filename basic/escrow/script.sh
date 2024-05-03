set -x

export xrd=030000000000000000000000000000000000000000000000000004
resim reset

OP1=$(resim new-account)
export priv1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export pub1=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export acc1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export priv2=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export pub2=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export acc2=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

resim set-default-account $acc2 $priv2
export dgc=$(resim new-token-fixed 8000000000 --description "The first Doge project on Radix" --name "DogeCube" --symbol "DGC" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

resim set-default-account $acc1 $priv1
export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

CP_OP=$(resim call-function $package Escrow new $xrd $dgc $acc1 $acc2)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
export badgeA=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export badgeB=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')

resim transfer 1 $badgeB $acc2
resim call-method $component put_tokens 500,$xrd 1,$badgeA
resim set-default-account $acc2 $priv2
resim call-method $component put_tokens 10,$dgc 1,$badgeB
resim call-method $component accept 1,$badgeB
resim set-default-account $acc1 $priv1
resim call-method $component accept 1,$badgeA
resim call-method $component withdraw 1,$badgeA
resim set-default-account $acc2 $priv2
resim call-method $component withdraw 1,$badgeB

resim show $acc1
resim show $acc2

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