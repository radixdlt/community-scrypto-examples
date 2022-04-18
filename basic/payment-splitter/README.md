# Payment Splitter

Just like in the physical world, when multiple people come together to build an NFT or a DeFI project they typically want to split the profits from such project in a clear and trustworthy way. The approach to have one entity to trust with the funds to later send to other shareholders simply bears too much risk of one party going rogue and refusing to send the shareholders their stake. This calls for a better way for funds from a project to be managed and split across the different shareholders in the project. The `PaymentSplitter` is a Scrypto blueprint which uses NFTs to authenticate shareholders and allow shareholders to have a clear and trustworthy way of withdrawing their profits from a project when they wish to do so.

## Motivations

The motivation behind this blueprint is to build a payment splitter similar to Ethereum's OpenZeppelin [implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/finance/PaymentSplitter.sol) but to build it on Radix using all of the new and exciting concepts and ideas that Radix provides.

## Features
The `PaymentSplitter` blueprint comes with quite a number of features. Such as:

* Allows for an easy way to split funds between multiple different entities.
* Supports XRD as well as other fungible tokens.
* Allows the the admin of the payment splitter to disable the addition of new shareholders (to protect current shareholders).
* Allows shareholders to withdraw their owed funds in full or in part.
* Allows shareholders to give up their shares if they choose to.

## Getting Started

If you wand to try out this blueprint there are three main ways to do that, the first is by using a new feature with Scrypto v0.3.0 which is the new transaction model and the transaction manifests, the second is by using an `script.sh` file which runs all of the needed CLI commands for the example, and the third and final method is by typing in the commands manually. 

### Method 1: Using transaction manifest files

Transaction manifests is an extremely cool new feature introduced with v0.3.0 of scrypto which allows for transaction instructions to be written in an external `.rtm` file and run by resim. This feature allows for extremely powerful transactions to be created where multiple different actions take place in a single transaction. 

Lets begin by resetting the radix engine simulator by running the following command:

```sh
resim reset
```

We will need a number of accounts for the examples that we will be running. So, let's create four different accounts using the following command:

```sh
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export SHAREHOLDER1_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP3=$(resim new-account)
export SHAREHOLDER2_PRIV_KEY=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_PUB_KEY=$(echo "$OP3" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_ADDRESS=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP4=$(resim new-account)
export SHAREHOLDER3_PRIV_KEY=$(echo "$OP4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_PUB_KEY=$(echo "$OP4" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_ADDRESS=$(echo "$OP4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

Now that we have created four different accounts, let's set the first account to be the default account by running the following command:

```sh
resim set-default-account $ADMIN_ADDRESS $ADMIN_PUB_KEY $ADMIN_PRIV_KEY
```

Let's now publish the package to our local radix engine simulator by running the following:

```sh
resim publish .
```

Now begins the exciting stuff with the new transaction model! The very first radix transaction manifest file that we will using is a file that creates a PaymentSplitter component on the local simulator. To run this file, run the following command:

```
$ resim run transactions/component_creation.rtm
```

What we would like to do now is to add shareholders to our PaymentSplitter component and to send the shareholders the NFTs that prove their identity. If we decide not to use the new transaction manifest feature we would have twice as many transaction as shareholders we wish to add (i.e. 10 shareholders = 20 transactions) as each shareholder would require a transaction for the minting of the NFT and another transaction for the sending of the NFT. However, with the new transaction model and the introduction of the transaction manifest, we can perform all of that in a single transaction! We have created a radix transaction manifest (rtm) file for you that performs the minting and sending of the tokens all in a single transaction! You may run it through the following command:

```sh
$ resim run ./transactions/adding_shareholders.rtm
```

After the above command runs you can check the balances of the four accounts that we have and you will see that each one of those accounts now have 1 shareholder NFT.

We would now like to test the PaymentSplitter to make sure that it does indeed do what we expect it to do. For that purpose we will deposit some funds into the payment splitter from account 1 and then we will withdraw them the share of the second account of the funds.

Let's run the transaction manifest file containing the instruction of the depositing of tokens through account 1.
```sh
$ resim run ./transactions/funding_the_splitter.rtm 
```

We may now switch to account 2 and try to withdraw the funds.

```sh
$ resim set-default-account $SHAREHOLDER1_ADDRESS $SHAREHOLDER1_PUB_KEY $SHAREHOLDER1_PRIV_KEY
$ resim run ./transactions/withdrawing_owed_amount.rtm 
```

And that's it! Now if you check the balance of account 2 you would see that account 2 now has more XRD as a result of the splitting of profits made.

### Method 2: Automatic method using `script.sh`.

The `script.sh` file included with this blueprint is a quick bash script written during the implementation of the blueprint to allow for a quick way for the package to be redeployed and for accounts to be created quickly and with ease.

When you run the `script.sh` file the following will take place:

* Your resim will be reset so that any accounts previously created or packages deployed are deleted.
* Four new accounts will be created. The script will keep track of their addresses and public keys.
* The package will be published and the script will store the package address.
* The script will call the `new` function on the blueprint to create the PaymentSplitter component. When creating the component, a number of resource definitions will be created for the badges that blueprint creates.
* The script will then add all four of the addresses that we created as shareholders by calling the `add_shareholder` method on the PaymentSplitter component.
* Some XRD will be deposited by account 1 (by calling the `deposit` method) and then later account 2 will attempt to withdraw their share of it (by calling the `withdraw` method).

You do not need to install any additional packages to run this `script.sh` file. This file used the standard python 3 library and packages. So, to run this example just type the following into your command line:

```sh
./build_rtm.sh
./script.sh
```

| NOTE | If you try running the `script.sh` script and you get errors such saying that some addresses were not found, then please run the `build_rtm.sh` script first and then the `script.sh`. |
|------|-----|

## Future Work and Improvements

This is certainly a simple example of what a payment splitter might look like if built on Radix using Scrypto where NFTs are used for authentication and tracking of data. Of course, there are a number of improvements that can be made to this blueprint to make it function even better:

* Allowing for a voting mechanism whereby the admin would regain the to add shareholders if a majority of the shareholders agree that the admin should be given this power back.