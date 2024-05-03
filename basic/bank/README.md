## Usage


# Simple bank example

A very simple example which allows you to withdraw and deposit from a bank.

## Setup

Reset your environment: `resim reset`

Create a new account: `resim new-account`


Build and deploy the blueprint on the local ledger: `resim publish .`


Call the start function to instantiate a component with the two Vaults, `bank_account` and `cash` and a Badge for the identification of beeing the owner of the bank account.

This is done by executing `resim call-function <package> Bank instantiate_bank`


## Calling the methods

You can call the methods with the following commands

`resim call-method <component> balances`

`resim call-method <component> bank_deposit <exchange>`

## Hints

To make it easier, I use local variables on a Mac.
You can safe the `component address` and the `Resource address` for the admin badge in local variables and call them easier.

#### Hit example
Lets assume this is your console and it logs 
`Component: 02db915b48cbb0a659a2680398715f7e7cff09c3b250f832742cf7`

I'd save that to a local variable with that command 

`export component=02db915b48cbb0a659a2680398715f7e7cff09c3b250f832742cf7` 

to call it then here for instance.

`resim call-method $component bank_deposit 200`

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