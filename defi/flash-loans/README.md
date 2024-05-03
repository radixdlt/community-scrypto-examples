# Flash Loans example

How to test:

## build flashloan component

1. `resim reset`
1. `export xrd=030000000000000000000000000000000000000000000000000004`
1. `cd flashloan`
1. `scrypto build`
1. `resim publish .` -> save package id into $fl_package
1. `resim new-account` -> save address into $acc1 and private key into $priv1

Create the flashloan component with 900 000 XRD of funds and a 5% interest
1. `resim call-function $fl_package FlashLoan new 900000,$xrd 5` -> save component address into $flashloan

## build caller component

1. `cd ../caller`
1. `scrypto build`
1. `resim publish .` -> save package id into $caller_package
1. `resim call-function $caller_package Caller new $flashloan 10000,$xrd` -> save component address into $caller

## ask for a flashloan
1. `resim call-method $caller call $caller` - We have to send the address of the caller component since we cannot yet get it from the Context

## Testing what happend when not returning fully owed amount
Call the same command again, this time, the opportunity bucket contains 0 XRD so you are not able to pay for the loan !


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