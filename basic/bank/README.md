## Usage


# Simple bank example

A very simple example which allows you to withdraw and deposit from a bank.

## Setup

Reset your environment: `resim reset`

Create a new account: `resim new-account`


Build and deploy the blueprint on the local ledger: `resim publish .`


Call the start function to instantiate a component with the two Vaults, `bank_account` and `cash` and a Badge for the identification of beeing the owner of the bank account.

This is done by executing `resim call-function <package> Bank start`


## Calling the methods

You can call the methods with the following commands

`resim call-method <component> balances`

`resim call-method <component> bank_deposit <exchange>`

`resim call-method <component> bank_withdraw <exchange> 1,<ResourceDef>`


## Hints

To make it easier, I use local variables on a Mac.
You can safe the `component adress` and the `ResourceDef adress` for the admin badge in local variables and call them easier.

#### Hit example
Lets assume this is your console and it logs 
`Component: 02db915b48cbb0a659a2680398715f7e7cff09c3b250f832742cf7`

 I'd safe that to a local variable with that command 
 
 `export component=02db915b48cbb0a659a2680398715f7e7cff09c3b250f832742cf7` 
 
 to call it then here for instance.

`resim call-method $component bank_deposit 200`