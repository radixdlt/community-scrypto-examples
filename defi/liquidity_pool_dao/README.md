## DAO-style liquidity pools
This package includes two blueprints: `LiquidtyPool` and `LiquidityPoolDAO`. It implements a DAO governance to allows Liquidity Providers (LP) to change the parameters of the pool. This project was inspired by [Osmosis](https://osmosis.gitbook.io/o/) DEX on Cosmos. The `LiquidtyPool` blueprint is mostly Omar's work from his [RaDEX submission](https://github.com/radixdlt/scrypto-challenges/tree/main/1-exchanges/RaDEX) to the DEX challenge (no need to reinvent the wheel).

Osmosis is an AMM protocol built using the Cosmos SDK that will allow developers to design, build, and deploy their own customized AMMs. Osmosis is designed such that the most efficient solution is reachable through the process of experimentation. It achieves this by offering deep customizability to AMM designers, and a governance mechanism by which each AMM pool’s stakeholders (i.e. liquidity providers) can govern and direct their pools. The Osmo token is a governance token and primarily used for voting on protocol upgrades, allocating liquidity mining rewards for liquidity pools, and setting the base network swap fee.

Currently, you can only change token weights and swap fees. How the voting proposal and voting process works is explained in the [Governance](#governance) section.

## Main Features
* **Propose protocol change** - Allows LPs to propose a change to the pool parameters.
* **Deposit to proposal** - Allows LPs to deposit XRD to the proposal they want to advance to the next stage.
* **Vote proposal** - Allows LPs to vote on the proposal they wish to pass.
* **Recast votes** - Allows LPs to change their votes.
* **Resolve proposal** - Evaluates the governance proposal.
* **Retrieve LP Tokens** - Redeem the LP Tokens used to allocate votes.

# Misc. Features
* **Check Proposal** - Retrieves the proposal information.
* **Check vote** - Retrieves your Vote Badge NFT information.
* **See Proposal** - Retrieves the NonFungibleId of the proposals in Voting Process stage.

## Design Considerations
This package is to demonstrate more examples of existing DeFi protocols on other platforms in an asset-oriented approach. I have not dug deep enough to do a fair evaluation with how Osmosis is designed vs. how it is designed on Radix. All I can say is that the implementation of this idea was quite easy to do given the nature of platform native tokens to allow me to represent votes and proposals.

### Votes
I've used fungible LP Tokens as a way to cast votes. I understand that this can create a situation in which LPs can disperse their LP Tokens across multiple votes. Although, I did not find a good reason to prevent LPs from doing that since doing so seems moot. 

Votes counted this way is quite elegant as the LPs receive LP Token that represent their ownership of the liquidity pool. Therefore, their votes will be weighted as such, giving more influence to those who have more skin in the game. 

### Proposal NFTs
Governance proposals represented by NFTs seems like an intuitive way to capture proposed protocol changes as the NFTs can be quered and identified. It also contains the necessary data to evaluate its popularity and the flexibility to be moved around, change its state, and manage the pool parameters.

### Vote Badge NFTs
LPs who cast their vote will need to deposit and lock their LP Tokens. There may be good reasons to mint a representation of the LP Tokens and be deposited back to LPs, although, that wasn't the focus of this project. Nonetheless, because of Radix's asset-oriented approach, anyone who will be using this package can easily make those changes if they so desire.

As votes are casted, LPs receive an NFT receipt of their vote. LPs can use this to recast their vote if they change their mind or transfer it to someone else if they wish to do so. 

The Vote Badge NFT will be required to be deposited if LPs wish to retrieve their LP Tokens. There is no lockout period for voting.

### Governance
I've primarily followed how governance is done on Cosmos which can be viewed [here](https://github.com/osmosis-labs/governance/blob/main/overview.md).

The basic gestalt is that there is an on-chain governance mechanism where LPs can participate in on-chain governance by creating and voting for governance proposals. These proposal can change the protocol pool parameters. Currently, only the token weights and swap fees can be changed, but can be easily implemented depending on your needs. Only LPs can participate in the creation and voting of governance proposal. 

There are two phases to the governance process: 
1. Deposit phase.
2. Voting period. 

During the deposit phase where a proposal is being promoted, a minimum of 500 XRD or until the deposit phase (5 epoch) lapses, whichever comes first, has to be met for a proposal to move to the voting period stage. The choice of using 5 epoch is for demonstration purposes only. This can be changed to your needs if you wish to use this package for your dApp. 

The voting period is currently 10 epoch. Again, the choice of using 10 epoch is for demonstration purposes only. During the voting period, participants may select a vote of either 'Yes', 'No', or 'No with veto'. Voters may change their vote at any time before the voting period ends.

There are four criteria to passing a proposal:
1. A minimum deposit of 500 XRD is required for the proposal to enter the voting period.
2. The deposit must be reached within 5 epochs.
3. A minimum of 30% of the protocol's voting power (quorum) is required to participate to make the proposal valid.
4. A simple majority (greater than 50%) of the participating voting power must back the 'Yes' vote within 10 epochs.
5. Less than 33.4% of participating voting power votes 'No with veto'

* The `Yes` vote signifies support of the proposal.
* The `No` vote signifies disagreement of the proposal.
* The `No with veto` vote signifies a significantly stronger disagreement, where you may feel the proposal is actively harming the governance 
system by abusing or spamming pointless proposals just to send a message. 

### Deposit fee
The deposit fee asset is chosen by default as XRD to encourage the utilization of the XRD token. Currently, there is no use for the deposit fee and can be considered burnt.

## Demonstration

### Getting Started

Let's begin by setting up our accounts.

```sh
OP1=$(resim new-account)
export PRIV_KEY1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY1=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP2=$(resim new-account)
export PRIV_KEY2=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY2=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS2=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP3=$(resim new-account)
export PRIV_KEY3=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY3=$(echo "$OP3" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS3=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

Let's also create an environment variable for XRD for convenience. 

```sh
export XRD=030000000000000000000000000000000000000000000000000004
```

We shall use $ACC_ADDRESS1 to mint resources, transfer funds, and set up our initial pool.

```sh
resim set-default-account $ACC_ADDRESS1 $PRIV_KEY1
```

We can now mint our "USD" tokens.

```sh
M_OP=$(resim run "./transactions/mint_usd.rtm")
export USD=$(echo "$M_OP" | sed -nr "s/└─ Resource: ([[:alnum:]_]+)/\1/p")
```

And fund our other two accounts using the following transaction manifest file.

```sh
resim transfer 100000 $USD $ACC_ADDRESS2
resim transfer 100000 $USD $ACC_ADDRESS3
```

We'll instantiate the package using the following commands. 

```sh
PK_OP=$(resim publish ".")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
CP_OP=$(resim run "./transactions/component_creation.rtm")
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/.* Component: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export DAO=$(echo "$CP_OP" | sed -nr "s/.* Component: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
export TT=$(echo "$CP_OP" | sed -nr "s/.* Resource: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
```

So these shortcuts, while convenient, frankly, doesn't illucidate what is going on. So I'll explain how we instantiated the package and created our initial pool.

The `LiquiduityPool` component calls for a few arguments in order for it to be instantiated and create a pool.
The arguments we must pass are as follows:

```rust
pub fn new(
    token_1_weight: Decimal,
    token_2_weight: Decimal,
    token1: Bucket,
    token2: Bucket,
    fee_to_pool: Decimal
) 
```

The transactions manifest file [./transactions/component_creation.rtm](./transactions/component_creation.rtm) already have a predetermined set of arguments that I've passed for demonstration purposes. For simplicity, I passed the weight for Token 1 to be 50%, the weight for Token 2 to be 50%, and likewise and equally, the amounts for each tokens to be 50. Finally, I set the swap fee to 2%. 

At this point, if you view the wallet of $ACC_ADDRESS1 by using the following command:

```sh
resim show $ACC_ADDRESS1
```
```sh
└─ { amount: 100, resource address: 034b34b5e68d5de29bee9fa7e11d94503e078c28c9a27e404125a3, name: "03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39-XRD LP Tracking Token", symbol: "TT" }
```

You will see that `$ACC_ADDRESS1` has received 100 Tracking Tokens or otherwise known as LP Tokens, representing the ownership to the pool. The math that tracks the percentage ownership of the pool is `Tracking Token Amount / Total Tracking Token Supply`. Since `$ACC_ADDRESS1` instantiated the pool, it is the first LP, therefore only 100 Tracking Tokens have been minted and thus represents 100% ownership of the pool. This will quickly change as more LP adds liquidity supply to the pool.

Let's head over to `$ACC_ADDRESS2` to do just that.

```sh
resim set-default-account $ACC_ADDRESS2 $PRIV_KEY2
```

And similarly to `$ACC_ADDRESS1` we'll add 50 of XRD and USD tokens to the pool by running the [resim run ./transactions/add_liquidity_from_acc2.rtm](resim run ./transactions/add_liquidity_from_acc2.rtm) transaction manifest file. Note that we can only add equivalent amounts of each given that our weights are set to 50% each. Attempting to provide different amounts outside of the bounds of the token weights will provide an error.

```sh
resim run ./transactions/add_liquidity_from_acc2.rtm
```
```sh
Logs: 4
├─ [INFO ] [Add Liquidity]: Requested adding liquidity of amounts, 030000000000000000000000000000000000000000000000000004: 50, 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 50
├─ [INFO ] [Add Liquidity]: Current reserves: 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 50, 030000000000000000000000000000000000000000000000000004: 50
├─ [INFO ] [Add Liquidity]: Liquidity amount to add: 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 50, 030000000000000000000000000000000000000000000000000004: 50
└─ [INFO ] [Add Liquidity]: Owed amount of tracking tokens: 100
```

So likewise, `$ACC_ADDRESS2` also received 100 LP Tokens. How did it arrive to this? Understanding how ownership is represented in this liquidity pool will help you understand how votes are weighted in the governance voting.

The math to calculating the Tracking Tokens (Liquidity Provider Tokens) is as follow:
```rust
let tracking_amount: Decimal = if tracking_tokens_manager.total_supply() == Decimal::zero() { 
    dec!("100.00") 
} else {
    amount1 * tracking_tokens_manager.total_supply() / m
};
```
Where `amount1` is the amount provided for Token 1 and `m` is the current total reserve for Token 1. Remember that `$ACC_ADDRESS1` deposited 50 XRD, which currently represents `m=50` and 100 LP Tokens were minted; which represents 100 total supply of Tracking Tokens (LP Tokens).

Since `$ACC_ADDRESS2` has provided 50 XRD as well, which represents `amount1=50`, we come to the conclusion as follows: `50 (amount1) * 100 (tracking_tokens_manager.total_supply()) / 50 (m) = 100`

Now how can we tell the owernship percentage between these two accounts? Since each owns 100 LP Tokens and now there are a total supply of 200 LP Tokens minted, each now own 50% of the liquidity pool.

Let's continue on with `$ACC_ADDRESS3` and add liquidity for the account by running the [./transactions/add_liquidity_from_acc3.rtm](./transactions/add_liquidity_from_acc3.rtm).

```sh
resim set-default-account $ACC_ADDRESS3 $PRIV_KEY3
```
```sh
resim run ./transactions/add_liquidity_from_acc3.rtm
```
```sh
Logs: 4
├─ [INFO ] [Add Liquidity]: Requested adding liquidity of amounts, 030000000000000000000000000000000000000000000000000004: 50, 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 50
├─ [INFO ] [Add Liquidity]: Current reserves: 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 100, 030000000000000000000000000000000000000000000000000004: 100
├─ [INFO ] [Add Liquidity]: Liquidity amount to add: 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39: 50, 030000000000000000000000000000000000000000000000000004: 50
└─ [INFO ] [Add Liquidity]: Owed amount of tracking tokens: 100
```

To keep things conistent for simplicity, `$ACC_ADDRESS3` also supplied 50 of each tokens to the liquidity pool. It should now be no surprise to you how 100 LP Tokens were minted for `$ACC_ADDRESS3`. You can also always play around with the amount supplied to see the dynamic of how LP Tokens are minted.

Now each accounts own 33% of the pool and we can begin enacting a simple demonstration how the voting process works!

## Creating our first governance proposal

Before we begin making our first governance proposal, since `LiquidityPool` and `LiquidityPoolDAO` are paired, we need to set the `ComponentAddress` of the `LiquidityPool` to the `LiquidityPoolDAO` component so that it can make method calls to the `LiquidityPool` component. We can do so by simply running the [./transactions/set_liquidity_pool_address.rtm](./transactions/set_liquidity_pool_address.rtm) transaction manifest file.

```sh
resim run ./transactions/set_liquidity_pool_address.rtm
```

Any LPs can propose to the protocol what they want to change. Here are the arguments that LPs must provide to begin proposing.

```rust
pub fn propose(
    &mut self,
    lp_proof: Proof,
    token_1_weight: Decimal,
    token_2_weight: Decimal,
    fee_to_pool: Decimal,
)
```

LPs must provide a `Proof` that they are an LP of the liquidity pool. For your convenience, I've provided the parameters for you in the following shortcut. You can view the transaction manifest file here [./transactions/propose.rtm](./transactions/propose.rtm). 

```sh
CP=$(resim run "./transactions/propose.rtm")
export PROPOSAL_ID=$(echo "$CP" | sed -nr "s/.*The proposal ID is ([[:alnum:]_]+)/\1/p")
```

We've successfully created a proposal to vote upon!

For some reason, running the shortcut doesn't provide output information. Otherwise, this output would be shown:
```sh
Logs: 8
├─ [INFO ] [Governance Proposal]: Proposal has has been created!
├─ [INFO ] [Governance Proposal]: The proposal ID is 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Governance Proposal]: Token 1 weight adjustment to 0.7 from 0.5
├─ [INFO ] [Governance Proposal]: Token 2 weight adjustment to 0.3 from 0.5
├─ [INFO ] [Governance Proposal]: Pool fee adjustment to 0.01 from 0.02
├─ [INFO ] [Governance Proposal]: The conditions to advance this proposal to the Voting Period are as follows:
├─ [INFO ] [Governance Proposal]: Condition 1 - A minimum of 500 in XRD must be deposited.
└─ [INFO ] [Governance Proposal]: The Voting Period will end in 5 epoch.
```

Alternatively, we can check the proposal we created by running following transaction manifest file [./transactions/check_proposal.rtm](./transactions/check_proposal.rtm).

```sh
resim run ./transactions/check_proposal.rtm
```
```sh
Logs: 12
├─ [INFO ] [Proposal Info]: Proposal ID: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Proposal Info]: Token 1 weight: 0.7
├─ [INFO ] [Proposal Info]: Token 2 weight: 0.3
├─ [INFO ] [Proposal Info]: Fee to Pool: 0.01
├─ [INFO ] [Proposal Info]: Amount required to be deposited to advance this proposal: 500
├─ [INFO ] [Proposal Info]: Deposit resource: 030000000000000000000000000000000000000000000000000004
├─ [INFO ] [Proposal Info]: Amount deposited to this proposal: 0
├─ [INFO ] [Proposal Info]: Yes: 0
├─ [INFO ] [Proposal Info]: No: 0
├─ [INFO ] [Proposal Info]: No with veto: 0
├─ [INFO ] [Proposal Info]: Abstain: 1
├─ [INFO ] [Proposal Info]: Current stage of the proposal: DepositPhase
├─ [INFO ] [Proposal Info]: Voting period ends in: 10 epoch
├─ [INFO ] [Proposal Info]: Deposit Phase period ends in: 5 epoch
├─ [INFO ] [Proposal Info]: Current epoch: 0
└─ [INFO ] [Proposal Info]: Resolution: InProcess
```

Note that your proposal ID may be different. As we can see, this provides some useful information to just about all the essential things we need to know about the status of the proposal. 

## Depositing towards our proposal

Now that we've created our proposal, we need to deposit a minimum of 500 XRD within 5 epochs in order to advance this proposal to the next stage. We can do so by running the following transaction manifest file [./transactions/deposit_proposal.rtm](./transactions/deposit_proposal.rtm).

```sh
resim run ./transactions/deposit_proposal.rtm
```
```sh
Logs: 5
├─ [INFO ] [Depositing Proposal]: This proposal has now advanced to the Voting Period phase!
├─ [INFO ] [Depositing Proposal]: The conditions to pass this proposal are as follows:
├─ [INFO ] [Depositing Proposal]: Condition 1 - Simple majority will need to vote 'Yes' in order to pass this proposal
├─ [INFO ] [Depositing Proposal]: Condition 2 - A minimum of 0.3 of the protocol's voting power (quorum) is required.
└─ [INFO ] [Depositing Proposal]: Condition 3 - Less than 0.334 of participating voting power votes 'no-with-veto'.
```

Note that only one proposal can be advanced at a time. Attempting to advance multiple proposal will result in the following error:

```sh
Logs: 1
└─ [ERROR] Panicked at 'assertion failed: `(left == right)`
  left: `2`,
 right: `1`: Cannot have more than one proposal in the voting period.', src\liquidity_pool_dao.rs:370:17
 ```

## Voting for our proposal

Now that we have advanced our proposal to the Voting Period, it's now time to have our accounts vote to simulate different scenarios!

### Voting Yes

Since we're still on `$ACC_ADDRESS3`, let's have it vote `Yes` to this proposal and see the result.

The arguments required to call the `vote_proposal` method is as follows:
```rust
pub fn vote_proposal(
    &mut self,
    lp_tokens: Bucket,
    vote_submission: Vote,
    proposal_id: NonFungibleId,
) 
```
and we can run the [./transactions/acc3_vote_yes.rtm](./transactions/acc3_vote_yes.rtm) transaction manifest file in order to vote `Yes`. 

```sh
VP=$(resim run "./transactions/acc3_vote_yes.rtm")
export PROOF=$(echo "$VP" | sed -nr "s/.*Your badge proof is ([[:alnum:]_]+)/\1/p")
```

Again, using this shortcut won't provide the output, but I'll show what it would look like if we were to run the transaction manifest file directly.
```sh
Logs: 5
├─ [INFO ] [Voting]: The current count for the vote is: Yes - 0.333333333333333333 | No - 0 | No with veto - 0 | Abstain - 0.666666666666666666
├─ [INFO ] [Voting]: You have voted 'Yes' for the proposal.
├─ [INFO ] [Voting]: The weight of your vote is 0.333333333333333333.
├─ [INFO ] [Voting]: You've received a badge for your vote!
└─ [INFO ] [Voting]: Your badge proof is 035ef327d7814deb897a38e5bb334ad041c6ac8029f666ed43bc6c
```

Calling this method will provide us with a Vote Badge NFT. We can view our Vote Badge NFT by running the following transaction manifest file [./transactions/acc3_acc3_check_vote.rtm](./transactions/acc3_check_vote.rtm).

```sh
resim run ./transactions/acc3_check_vote.rtm
```
```sh
Logs: 4
├─ [INFO ] [Vote Info]: Proposal: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Vote Info]: Vote: Yes
├─ [INFO ] [Vote Info]: Vote weight: 0.333333333333333333
└─ [INFO ] [Vote Info]: LP tokens allocated: 100
```

We can also look at the status of the proposal now that its first vote has been casted.

```sh
resim run ./transactions/check_proposal.rtm
```
```sh
Logs: 12
├─ [INFO ] [Proposal Info]: Proposal ID: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Proposal Info]: Token 1 weight: 0.7
├─ [INFO ] [Proposal Info]: Token 2 weight: 0.3
├─ [INFO ] [Proposal Info]: Fee to Pool: 0.01
├─ [INFO ] [Proposal Info]: Amount required to be deposited to advance this proposal: 500
├─ [INFO ] [Proposal Info]: Deposit resource: 030000000000000000000000000000000000000000000000000004
├─ [INFO ] [Proposal Info]: Amount deposited to this proposal: 500
├─ [INFO ] [Proposal Info]: Yes: 0.333333333333333333
├─ [INFO ] [Proposal Info]: No: 0
├─ [INFO ] [Proposal Info]: No with veto: 0
├─ [INFO ] [Proposal Info]: Abstain: 0.666666666666666667
├─ [INFO ] [Proposal Info]: Current stage of the proposal: VotingPeriod
├─ [INFO ] [Proposal Info]: Voting period ends in: 10 epoch
├─ [INFO ] [Proposal Info]: Deposit Phase period ends in: 5 epoch
├─ [INFO ] [Proposal Info]: Current epoch: 0
└─ [INFO ] [Proposal Info]: Resolution: InProcess
```

As you can see a few things have changed from when we last saw the proposal. For one, the stage of the proposal has changed from `DepositPhase` to `VotingPeriod` indicating that the proposal is ready to be voted upon. We also see that there is a current weight of 33% that's been allocated to the `Yes` vote, which mirrors what is recorded in our Vote Badge NFT.


### Voting No

Let's head over to `$ACC_ADDRESS1` to cast a `No` vote and see what happens.

```sh
resim set-default-account $ACC_ADDRESS1 $PRIV_KEY1
```

[./transactions/acc1_vote_no.rtm](./transactions/acc1_vote_no.rtm)

```sh
resim run ./transactions/acc1_vote_no.rtm
```
```sh
Logs: 5
├─ [INFO ] [Voting]: The current count for the vote is: Yes - 0.333333333333333333 | No - 0.333333333333333333 | No with veto - 0 | Abstain - 0.333333333333333334
├─ [INFO ] [Voting]: You have voted 'No' for the proposal.
├─ [INFO ] [Voting]: The weight of your vote is 0.333333333333333333.
├─ [INFO ] [Voting]: You've received a badge for your vote!
└─ [INFO ] [Voting]: Your badge proof is 035ef327d7814deb897a38e5bb334ad041c6ac8029f666ed43bc6c
```

Let's also have `$ACC_ADDRESS2` vote `No`.

```sh
resim set-default-account $ACC_ADDRESS2 $PRIV_KEY2
```

[./transactions/acc2_vote_no.rtm](./transactions/acc2_vote_no.rtm)

```sh
resim run ./transactions/acc2_vote_no.rtm
```
```sh
Logs: 5
├─ [INFO ] [Voting]: The current count for the vote is: Yes - 0.333333333333333333 | No - 0.666666666666666666 | No with veto - 0 | Abstain - 0.000000000000000001
├─ [INFO ] [Voting]: You have voted 'No' for the proposal.
├─ [INFO ] [Voting]: The weight of your vote is 0.333333333333333333.
├─ [INFO ] [Voting]: You've received a badge for your vote!
└─ [INFO ] [Voting]: Your badge proof is 035ef327d7814deb897a38e5bb334ad041c6ac8029f666ed43bc6c
```

We can check the proposal again.

```sh
resim run ./transactions/check_proposal.rtm
```
```sh
Logs: 12
├─ [INFO ] [Proposal Info]: Proposal ID: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Proposal Info]: Token 1 weight: 0.7
├─ [INFO ] [Proposal Info]: Token 2 weight: 0.3
├─ [INFO ] [Proposal Info]: Fee to Pool: 0.01
├─ [INFO ] [Proposal Info]: Amount required to be deposited to advance this proposal: 500
├─ [INFO ] [Proposal Info]: Deposit resource: 030000000000000000000000000000000000000000000000000004
├─ [INFO ] [Proposal Info]: Amount deposited to this proposal: 500
├─ [INFO ] [Proposal Info]: Yes: 0.333333333333333333
├─ [INFO ] [Proposal Info]: No: 0.666666666666666666
├─ [INFO ] [Proposal Info]: No with veto: 0
├─ [INFO ] [Proposal Info]: Abstain: 0.000000000000000001
├─ [INFO ] [Proposal Info]: Current stage of the proposal: VotingPeriod
├─ [INFO ] [Proposal Info]: Voting period ends in: 10 epoch
├─ [INFO ] [Proposal Info]: Deposit Phase period ends in: 5 epoch
├─ [INFO ] [Proposal Info]: Current epoch: 0
└─ [INFO ] [Proposal Info]: Resolution: InProcess
```

### Recasting our vote

Since all three accounts have casted their votes, we can now resolve this proposal. However, to show more functionalities, let's first take a look at what recasting our vote looks like.

The arguments for calling `recast_vote` is as follows:
```rust
pub fn recast_vote(
    &mut self,
    vote_badge: Proof,
    vote_submission: Vote,
)
```

And we can run the following transaction manifest file [./transactions/acc2_recast_vote_yes.rtm](./transactions/acc2_recast_vote_yes.rtm)

```sh
resim run ./transactions/acc2_recast_vote_yes.rtm
```
```sh
Logs: 5
├─ [INFO ] [Vote Recast]: Your allocations towards your 'No' cast has been decreased by 100.
├─ [INFO ] [Vote Recast]: Your weighted vote towards 'No' cast has been decreased by 0.333333333333333333
├─ [INFO ] [Vote Recast]: You have voted 'Yes' for the proposal.
├─ [INFO ] [Vote Recast]: The weight of your vote is 0.333333333333333333.
└─ [INFO ] [Vote Recast]: The current count for the vote is: Yes - 0.666666666666666666 | No - 0.333333333333333333 | No with veto - 0 | Abstain - 0.000000000000000001
```
We can check our proposal again.

```sh
resim run ./transactions/check_proposal.rtm
```
```sh
Logs: 12
├─ [INFO ] [Proposal Info]: Proposal ID: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Proposal Info]: Token 1 weight: 0.7
├─ [INFO ] [Proposal Info]: Token 2 weight: 0.3
├─ [INFO ] [Proposal Info]: Fee to Pool: 0.01
├─ [INFO ] [Proposal Info]: Amount required to be deposited to advance this proposal: 500
├─ [INFO ] [Proposal Info]: Deposit resource: 030000000000000000000000000000000000000000000000000004
├─ [INFO ] [Proposal Info]: Amount deposited to this proposal: 500
├─ [INFO ] [Proposal Info]: Yes: 0.666666666666666666
├─ [INFO ] [Proposal Info]: No: 0.333333333333333333
├─ [INFO ] [Proposal Info]: No with veto: 0
├─ [INFO ] [Proposal Info]: Abstain: 0.000000000000000001
├─ [INFO ] [Proposal Info]: Current stage of the proposal: VotingPeriod
├─ [INFO ] [Proposal Info]: Voting period ends in: 10 epoch
├─ [INFO ] [Proposal Info]: Deposit Phase period ends in: 5 epoch
├─ [INFO ] [Proposal Info]: Current epoch: 0
└─ [INFO ] [Proposal Info]: Resolution: InProcess
```

And check our Vote Bage NFT.

```sh
resim call-method $DAO check_vote 1,$PROOF
```
```sh
Logs: 4
├─ [INFO ] [Vote Info]: Proposal: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Vote Info]: Vote: Yes
├─ [INFO ] [Vote Info]: Vote weight: 0.333333333333333333
└─ [INFO ] [Vote Info]: LP tokens allocated: 100
```

### Retrieving LP Token

At this point we can resolve the proposal and since the proposal currently has a simply majority vote of `Yes` the proposal will pass. But let's take the opportunity to take a look at what happens when an LP retrieves their LP Token in the middle of a Voting Period.

Let's run the transaction manifest file

```sh
resim run ./transactions/acc2_retrieve_lp_tokens.rtm
```
```sh
Logs: 3
├─ [INFO ] [Retrieve LP Tokens]: You have withdrawn 100 LP Tokens
├─ [INFO ] [Retrieve LP Tokens]: Your `Yes` vote weight has been decreased by 0.333333333333333333
└─ [INFO ] [Retrieve LP Tokens]: Your Vote Badge NFT has been burnt.
```
Let's check the proposal again

```sh
resim run ./transactions/check_proposal.rtm
```
```sh
Logs: 12
├─ [INFO ] [Proposal Info]: Proposal ID: 0bac4138262c7c050f46dfdc67acfcd8
├─ [INFO ] [Proposal Info]: Token 1 weight: 0.7
├─ [INFO ] [Proposal Info]: Token 2 weight: 0.3
├─ [INFO ] [Proposal Info]: Fee to Pool: 0.01
├─ [INFO ] [Proposal Info]: Amount required to be deposited to advance this proposal: 500
├─ [INFO ] [Proposal Info]: Deposit resource: 030000000000000000000000000000000000000000000000000004
├─ [INFO ] [Proposal Info]: Amount deposited to this proposal: 500
├─ [INFO ] [Proposal Info]: Yes: 0.333333333333333333
├─ [INFO ] [Proposal Info]: No: 0.333333333333333333
├─ [INFO ] [Proposal Info]: No with veto: 0
├─ [INFO ] [Proposal Info]: Abstain: 0.333333333333333334
├─ [INFO ] [Proposal Info]: Current stage of the proposal: VotingPeriod
├─ [INFO ] [Proposal Info]: Voting period ends in: 10 epoch
├─ [INFO ] [Proposal Info]: Deposit Phase period ends in: 5 epoch
├─ [INFO ] [Proposal Info]: Current epoch: 0
└─ [INFO ] [Proposal Info]: Resolution: InProcess
```

Since your Vote Badge NFT has been burnt we can review our wallet.

```sh
resim show $ACC_ADDRESS2
```
```sh
Resources:
├─ { amount: 0, resource address: 035ef327d7814deb897a38e5bb334ad041c6ac8029f666ed43bc6c, name: "Vote Badge", symbol: "VB" }
├─ { amount: 100, resource address: 034b34b5e68d5de29bee9fa7e11d94503e078c28c9a27e404125a3, name: "03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39-XRD LP Tracking Token", symbol: "TT" }
├─ { amount: 999950, resource address: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
└─ { amount: 99950, resource address: 03f59d354b010c9c003dd8ce40a28cbd8e7f1fd8d11218f1d1fc39 }
```

We can now see that we've retrieved `$ACC_ADDRESS2` LP Tokens and we no longer have a Vote Badge NFT.

### Resolving our proposal

To show what proposal resolution looks like let's have `$ACC_ADDRESS2` recast their vote again.

This time `$ACC_ADDRESS2` will be voting `Yes` by running the transaction manifest file [./transactions/acc2_vote_yes.rtm](./transactions/acc2_vote_yes.rtm)

```sh
resim run ./transactions/acc2_vote_yes.rtm
```
```sh
Logs: 5
├─ [INFO ] [Voting]: The current count for the vote is: Yes - 0.666666666666666666 | No - 0.333333333333333333 | No with veto - 0 | Abstain - 0.000000000000000001
├─ [INFO ] [Voting]: You have voted 'Yes' for the proposal.
├─ [INFO ] [Voting]: The weight of your vote is 0.333333333333333333.
├─ [INFO ] [Voting]: You've received a badge for your vote!
└─ [INFO ] [Voting]: Your badge proof is 035ef327d7814deb897a38e5bb334ad041c6ac8029f666ed43bc6c
```

If you wish, you can check the proposal once more, but for the sake of brevity, we'll now begin to move on to passing the proposal since it now has a simply majority vote of `Yes`.

Let's first check again what our pool parameters are to see how the changes happen with this proposal being passed. Run the transactions manifest file [./transactions/get_pool_info.rtm](./transactions/get_pool_info.rtm)

```sh
resim run ./transactions/get_pool_info.rtm
```
```sh
Logs: 3
├─ [INFO ] Token 1 weight: 0.5
├─ [INFO ] Token 2 weight: 0.5
└─ [INFO ] Pool fee: 0.02
```

[./transactions/resolve_proposal.rtm](./transactions/resolve_proposal.rtm)

```sh
resim run ./transactions/resolve_proposal.rtm
```
```sh
Logs: 5
├─ [INFO ] [Proposal Resolution]: The proposal 0bac4138262c7c050f46dfdc67acfcd8 has passed!
├─ [INFO ] [Proposal Resolution]: 'Yes' count (0.666666666666666666) has the simple majority against the 'No' count (0.333333333333333333).
├─ [INFO ] [Proposal Resolution]: The pool paramaters has now changed to:
├─ [INFO ] [Proposal Resolution]: Token 1 weight: 0.7 | Token 2 weight: 0.3 | Fee to pool: 0.01
```

As the old Russian adage goes, "trust, but verify."

```sh
resim run ./transactions/get_pool_info.rtm
```
```sh
Logs: 3
├─ [INFO ] Token 1 weight: 0.7
├─ [INFO ] Token 2 weight: 0.3
└─ [INFO ] Pool fee: 0.01
```

Success! That's the rundown of how proposals are advanced through different stage. The Proposal NFT has now been burnt and LPs and the protocol has now succesfully been changed through a DAO governance vote.

So what happens when the proposal fails? The Proposal is simply burnt!

```sh
Logs: 3
├─ [INFO ] [Proposal Resolution]: The proposal 0e645ff3d871a92f25182dcdea985371 has failed!
├─ [INFO ] [Proposal Resolution]: 'No' count (1) has the simple majority against the 'Yes' count (0).
```

But let's get back to the scenario where the proposal has passed. What happens to the token if the they are over or under the new weights? Currently, nothing happens to the tokens.
Theoretically, one asset would be more expensive or cheaper priced than the rest of the market when the weight changes. Market forces takes place and arbitrageurs can take advantage of the free value until it meets equilibrium with the rest of the market. Although, I've yet to see this happen in real time.