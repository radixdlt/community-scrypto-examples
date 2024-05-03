
# Vesting Blueprint
One of the things often seen in DeFI is the idea of some amount of tokens vesting or being unlocked over a period of time which is usually done either for tax purposes or to protect other shareholders from private investors. This makes token vesting a very important concept in the world of DeFI. Much like everything in this space, beneficiaries do not want to trust an external entity when it comes to the vesting of tokens; rather, they want the vesting to be handled by a trustless entity which performs the token vesting correctly, immediately, and without error.

## Motivations

The motivation behind building this blueprint is to showcase how trustless vesting functionality similar to that of Ethereum's OpenZeppelin [VestingWallet](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/finance/VestingWallet.sol) can be implemented on Radix using v0.4.0 of Scrypto which introduces the new authorization model. This implementation has more functionality than that of OpenZeppelin and allows for many exciting and interesting things with the new authorization model.

## Features
The vesting blueprint comes with quite a number of features. Such as:
* Allows for quick, easy, and immediate vesting of tokens for beneficiaries.
* Allows for multiple admins and multiple beneficiaries to exist on a single component.
* Allows admins to terminate the vesting of tokens belonging to a certain beneficiary.
* Gives beneficiaries security against termination by allowing admins to give-up termination rights.
* In multi-admin vesting components, the adding of new admins requires the approval of 50% of the admins.
* In multi-admin vesting components, the giving up of termination rights requires the approval of 50% of the admins.
## Details of Design

### Mathematics Behind Vesting

The current implementation of the vesting blueprint follows a linear approach to the vesting of tokens whereby no tokens are vested between the beginning period (enrollment epoch) and the cliff epoch. From there, the funds are vested linearly from the initial cliff amount until the end amount, this relationship can be seen in the linear graph below:

![The vesting graph used by the vesing blueprint.](./images/linear.svg)

From this typical vesting graph, we may derive a mathematical equation to model the behavior of the graph such that we can implement it in code later on. 

We can see that the graph behaves differently in different regions due to the cliff period, therefore, we can model this as a piecewise function where the function outputs 0 for all epoch values smaller than the cliff epoch and outputs something else when we're above the cliff period.

The behavior following the cliff epoch is just linear which means that it can be modeled using `y = mx + c`. The gradient may be found through the points (`e_end`, `f_total`) and (`e_cliff`, `f_cliff`) and the value of the y-intercept my be found through any of the two points given above. After calculating the `m` and `c` we end up with the following equation:
```
f(e) = ((f_total - f_cliff) / (c_end - c_cliff)) (e - c_cliff) + f_cliff
```

We would like to introduce an upper bound to this function such that the vested amount at epoch (e_end + 1) is only the total amount which can be vested. Therefore, we define `f(e)` as:
```
f(e) = min( ((f_total - f_cliff) / (c_end - c_cliff)) (e - c_cliff) + f_cliff, f_total )
```

This is the main mathematical function that governs the vesting of tokens in this vesting blueprint. Additional functions may be introduced later on to allow for exponential and decaying vesting schedules.

### Blueprint Details

The vesting blueprint is designed around the existence of two main types of entities in vesting components:

* **Admin:** Each vesting component has an admin who is given the authority to add more beneficiaries to the vesting component by paying the funds and setting the vesting schedule that they wish to provide to the beneficiary. In addition to that, the admin has the right to terminate the vesting of tokens of a beneficiary from the component and take the unclaimed funds. However, vesting components also allow admins to give up their termination authority if they wish to do so which protects beneficiaries. 
* **Beneficiary:** A Beneficiary is the party for which the tokens are being vested. A single vesting component may have multiple beneficiaries added to it and using it to take their tokens. Beneficiaries can withdraw their tokens from their vesting component through the `VestingSchedule` badge that they're given which keeps track of their schedule and how much tokens have vested so far. When a beneficiary is terminated by an admin, they lose access to all of their unclaimed tokens (including tokens which have already vested). 

The vesting blueprint has multi-admin support which is noticeable once additional admin badges are minted. In a multi-admin vesting component, there are a number of actions which require agreement from the majority of admins before they can be called.

Lets take a look at an example where a blueprint similar to this does not have additional auth on methods and how this impacts its security. The method `giveup_termination_rights` allows admins (as a whole) to give up their right to terminate vesting schedules. If any admin can call this method, then a rogue admin might call this method as a way to cause the other admins harm. In addition to that, if no additional auth is put on methods, a rogue admin would be able to call `add_admin` to create an infinite number of admin badges which they could later sell to the highest bidder who wants to terminate all vesting schedules and get away with the funds.

Therefore, it is important that certain methods are only callable when a majority of the admins have agreed to call them. You can find out which methods require a single admin and which methods require multiple admins from the table below. Methods where the intended user is `Multiple Admins` are methods which require agreement from multiple admins before they can be called. 

This package contains a single blueprint which is the `Vesting` blueprint. This blueprint is designed to hold the tokens of the beneficiaries in its vaults, issue the admin, and beneficiary badges, and terminate the vesting of tokens for a given beneficiary. The methods and functions defined for the `Vesting` blueprint are:

| Function / Method Name      | Auth Type | Intended User    | Description |
| --------------------------- | --------- | ---------------- | ----------- |
| `new`                       |           |                  | This function creates a new `Vesting` component. In doing that, this function creates a number of resources which are used for the internal admin badge, admin badge, and the beneficiary's badge. The addition of beneficiaries to the component is out of the scope of this function and is the job of a separate function.
| `add_beneficiary`           | Auth Zone | Any Admin        | This method allows for an admin to add a beneficiary to the vesting component alongside a bucket of tokens which we would like to vest over a period of time. 
| `terminate_beneficiary`     | Auth Zone | Multiple Admins  | This method allows for admins to terminate the vesting of tokens of a given beneficiary and take away all of the unclaimed and unvested tokens that the beneficiary has in their vesting vault. This method is authenticated and requires that 50% or more of the admins present their admin badges in order for the termination of the beneficiary to go through.
| `add_admin`                 | Auth Zone | Multiple Admins | This method allows for admins to add additional admins to the vesting component. This method is authenticated and requires that 50% or more of the admins present their admin badges in order for the minting of new admin tokens to be allowed.
| `giveup_termination_rights` | Auth Zone | Multiple Admins  | This method allows for admins of the vesting component to give up their termination rights while keeping their right to add additional beneficiaries to the vesting component. This method is authenticated and requires that 50% or more of the admins present their admin badges in order for the termination rights to be given away.
| `withdraw_funds`            | Pass By Intent | Beneficiary | This method allows beneficiaries to withdraw the funds that have vested so far from the vesting component.

As you can see from the able above, there are currently two main types of authentication possible with the new version of Scrypto: Authorization through the Auth Zone, and Authorization by passing a `Proof` by intent. 

Of course, you might be wondering: **what do these two types mean?**
* Authorization through the Auth Zone: This is the new system of authorization introduced with v0.4.0 of Scrypto where Proofs can be put in an Authorization Zone and any method or function that requires these Proofs can use them without the need for explicitly providing the `Proof`s in the method/function calls. This method makes authorization much easier and the syntax that it uses allows for components to be transparent about their authorization requirements. 
* Authorization by passing a Proof by intent: This is the old method of authorization whereby a method/function or method would be made to accept a `Proof` (formerly called `BucketRef`) and based on the amount, resource address, resource type, or other information the method or function is coded to behave differently 

You might be wondering, **How do I go about determining which authorization model is right my methods and function?**
As a good general rule of thumb, whenever we need to read the data present in a `Proof` then we need to pass it by intent, otherwise, if the presence of the `Proof` alone is sufficient for authorization, then we can use the Auth Zone.

## Examples

The examples for the vesting component are implemented in both `.rtm` files and as unit tests. Feel free to following along whichever one. 

In this example, we would like to create two accounts which will serve the role of the admin and beneficiary. The admin will add the beneficiary to the vesting component for their funds to vest over 100 epochs. The beneficiary will test the withdrawal of tokens at different epochs to ensure that the amount withdrawn is correct.

### Method 1: Using `script.sh` file

If you are looking to quickly try out the vesting component without needing to type out all of the commands by hand, then run the `script.sh` file which contains all of the commands that you might need. You can do that by running:
```sh
./script.sh
```

### Method 2: Using transaction manifest files

Lets begin by resetting `resim` to ensure tha the package, component, and resource addresses on my local machine match with yours:
```sh
resim reset
```

We now need to create two accounts which will be used by the admin and the beneficiary. To quickly create and save the two accounts to our environment variables we can run the following commands:
```sh
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP2=$(resim new-account)
export BENEFICIARY_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

With two accounts now created, we now need to ensure that the first account is being used as the default account as this is the account which we will be using to publish the package and eventually create the `Vesting` component.
```sh
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY
```

The token which we will be using for the vesting examples is an example token we're calling USDT. To create the token and deposit it back into the admin's account, run the following command:
```sh
TK_OP=$(resim run ./transactions/token_creation.rtm)
export USDT=$(echo "$TK_OP" | sed -nr "s/└─ Resource: ([[:alnum:]_]+)/\1/p")
```

With the account and the token created, lets publish the package and instantiate a new vesting component.
```sh
PK_OP=$(resim publish ".")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
CP_OP=$(resim call-function $PACKAGE Vesting instantiate_vesting)
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
```

Lets check to ensure that the admin component has an a Vesting admin badge in their vaults.
```sh
$ resim show $ADMIN_ADDRESS
Resources:
├─ { amount: 1000000, resource address: 03ce55ad1ab8ae1fd3081fa9d32c842f33c6139959ad9319af8361, name: "Tether", symbol: "USDT" }
├─ { amount: 1, resource address: 036076f8a8006d9619baed5fafee0d0c0b975903fccdef0e9d4c06, name: "Vesting Admin Badge" }
└─ { amount: 1000000, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
```

We can indeed see that the admin's account has the 1,000,000 USDT tokens that we created and in addition to that it has a single "Vesting Admin Badge". This badge can be used by the admin to add or terminate beneficiaries.

We would now like to add a new beneficiary with a new vesting schedule to the vesting component. The details of the vesting schedule of this beneficiary are as follows:

| Term          | Value |
| ------------- | -------------- |
| Amount        | 1,000,000 USDT |
| Cliff Epoch   | 20 Epochs from Now |
| End Epoch     | 100 Epochs from Now |
| Percentage Unlocked on Cliff     | 20% |

The [`add_beneficiary.rtm`](./transactions/add_beneficiary.rtm) file contains the instructions used by the admin to add a beneficiary with the above mentioned details to the vesting component and to then send the beneficiary their authentication NFT to them.

The NFT that beneficiaries are given does not only perform authentication of beneficiaries, it also acts as the beneficiary's vesting schedule so it includes all of the information about what their enrollment, cliff, and end epochs are and what the total amount they're owed are. We can add the beneficiary to the vesting component by running: 
```sh
resim run "./transactions/add_beneficiary.rtm"
```

Since the cliff epoch has been set to be 20 epochs in the future let's test out the component by going 10 epochs into the future and seeing if the component would return any tokens or not when the beneficiary attempts to withdraw. The expected behavior is that an empty bucket should be returned as the cliff epoch as not yet passed. The file [`withdraw_funds.rtm`](./transactions/withdraw_funds.rtm) contains the instructions needed to withdraw funds from the beneficiary's account.
```sh
$ resim set-default-account $BENEFICIARY_ADDRESS $BENEFICIARY_PRIV_KEY
$ resim set-current-epoch 10
$ resim run "./transactions/withdraw_funds.rtm"
Logs: 1
└─ [INFO ] [Withdraw Funds]: Withdraw successful. Withdrawing 0 tokens
New Entities: 0
```

As we can see, since the cliff epoch has not passed, no funds were withdrawn when we attempted to withdraw them. We can now try going to the cliff epoch to see if the amounts of funds given are as expected. We expect that when the cliff happens the beneficiary will get 20% * 1,000,000 = 200,000 USDT tokens. 
```sh
$ resim set-current-epoch 20
$ resim run "./transactions/withdraw_funds.rtm"
Logs: 1
└─ [INFO ] [Withdraw Funds]: Withdraw successful. Withdrawing 200000 tokens
New Entities: 0
```

That's the amount that we were expecting! We can now try to terminate the beneficiary's account to see how the component behaves when that happens. The [`terminate_beneficiary.rtm`](./transactions/terminate_beneficiary.rtm) file contains the instructions required to terminate the beneficiary.

```sh
$ resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY
$ resim run "$SCRIPT_DIR/transactions/terminate_beneficiary.rtm"
$ resim set-default-account $BENEFICIARY_ADDRESS $BENEFICIARY_PRIV_KEY
$ resim run "./transactions/withdraw_funds.rtm"
Logs: 1
└─ [ERROR] Panicked at '[Withdraw Funds]: Vesting has been terminated. Contact your admin for more information.', src/vesting.rs:267:13
New Entities: 0
```

As you can see, after the beneficiary has been terminated, they may no longer withdraw vested funds from the component and their funds are sent back to the admin. There is no restriction on adding the terminated beneficiary back to the vesting component with a new vesting schedule.

| NOTE | If you try running any of the transaction manifest files and you find an error saying that some addresses are incorrect, please run the `build_rtm.sh` file to build the `.rtm` files again with consistent addresses. |
|------|:----|

## Future Improvements

There are a number of ways in which the vesting blueprint can be improved:

* Making the blueprint support other types of graphs other than the linear graph. Example: A decaying or exponential vesting curve.

## Conclusion

Scrypto v0.4.0 allows us to implement vesting functionality on Radix with minimal effort and friction. The changes made to the auth model makes it so that complex auth concepts such as an multi-admin operations to be simple.

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