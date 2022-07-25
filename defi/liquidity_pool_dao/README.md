# DAO-style liquidity pools
This package includes two blueprints: `LiquidtyPool` and `LiquidityPoolDAO`. It implements a DAO governance to allows Liquidity Providers (LP) to change the parameters of the pool. This project was inspired by [Osmosis](https://osmosis.gitbook.io/o/) DEX on Cosmos. The `LiquidtyPool` blueprint is mostly Omar's work from his RaDEX submission to the DEX challenge (no need to reinvent the wheel).

Osmosis is an AMM protocol built using the Cosmos SDK that will allow developers to design, build, and deploy their own customized AMMs. Osmosis is designed such that the most efficient solution is reachable through the process of experimentation. It achieves this by offering deep customizability to AMM designers, and a governance mechanism by which each AMM pool’s stakeholders (i.e. liquidity providers) can govern and direct their pools. The Osmo token is a governance token and primarily used for voting on protocol upgrades, allocating liquidity mining rewards for liquidity pools, and setting the base network swap fee.

Currently, you can only change token weights and swap fees. 

# Main Features
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

# Design Considerations
This package is to demonstrate more examples of existing DeFi protocols on other platforms in an asset-oriented approach. I have not dug deep enough to do a fair evaluation with how Osmosis is designed vs. how it is designed on Radix. All I can say is that the implementation of this idea was quite easy to do given the nature of platform native tokens to allow me to represent votes and proposals.

## Votes
I've used fungible LP Tokens as a way to cast votes. I understand that this can create a situation in which LPs can disperse their LP Tokens across multiple votes. Although, I did not find a good reason to prevent LPs from doing that since doing so seems moot. 

Votes counted this way is quite elegant as the LPs receive LP Token that represent their ownership of the liquidity pool. Therefore, their votes will be weighted as such, giving more influence to those who have more skin in the game. 

## Proposal NFTs
Governance proposals represented by NFTs seems like an intuitive way to capture proposed protocol changes as the NFTs can be quered and identified. It also contains the necessary data to evaluate its popularity and the flexibility to be moved around, change its state, and manage the pool parameters.

## Vote Badge NFTs
LPs who cast their vote will need to deposit and lock their LP Tokens. There may be good reasons to mint a representation of the LP Tokens and be deposited back to LPs, although, that wasn't the focus of this project. Nonetheless, because of Radix's asset-oriented approach, anyone who will be using this package can easily make those changes if they so desire.

As votes are casted, LPs receive an NFT receipt of their vote. LPs can use this to recast their vote if they change their mind or transfer it to someone else if they wish to do so. 

The Vote Badge NFT will be required to be deposited if LPs wish to retrieve their LP Tokens. There is no lockout period for voting.

## Governance
I've primarily followed how governance is done on Cosmos which can be viewed [here](https://github.com/osmosis-labs/governance/blob/main/overview.md).

The basic gestalt is that there is an on-chain governance mechanism where LPs can participate in on-chain governance by creating and voting for governance proposals. These proposal can change the protocol pool parameters. Currently, only the token weights and swap fees can be changed, but can be easily implemented depending on your needs. Only LPs can participate in the creation and voting of governance proposal. 

There are two phases to the governance process: 
1. Deposit phase.
2. Voting period. 

During the deposit phase where a proposal is being promoted, a minimum of 500 XRD or until the deposit phase (5 epoch) lapses, whichever comes first, has to be met for a proposal to move to the voting period stage. The choice of using 5 epoch is for demonstration purposes only. This can be changed to your needs if you wish to use this package for your dApp. 

The voting period is currently 10 epoch. Again, the choice of using 10 epoch is for demonstration purposes only. During the voting period, participants may select a vote of either 'Yes', 'No', or 'No with veto'. Voters may change their vote at any time before the voting period ends.

There are four criteria to passing a proposal:
1. A minimum deposit of 500 XRD is required for the proposal to enter the voting period.
2. The deposit must be reached within 5 epoch.
3. A minimum of 30% of the protocol's voting power (quorum) is required to participate to make the proposal valid.
4. A simple majority (greater than 50%) of the participating voting power must back the 'Yes' vote within 10 epoch.
5. Less than 33.4% of participating voting power votes 'No with veto'

* The `Yes` vote signifies support of the proposal.
* The `No` vote signifies disagreement of the proposal.
* The `No with veto` vote signifies a significantly stronger disagreement, where you may feel the proposal is actively harming the governance 
system by abusing or spamming pointless proposals just to send a message. 

## Deposit fee
The deposit fee asset is chosen by default as XRD to encourage the utilizaiton of the XRD token. Currently, there is no use for the deposit fee and can be considered burnt.

# Demonstration

```sh
OP1=$(resim new-account)
export PRIV_KEY1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY1=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP2=$(resim new-account)
export PRIV_KEY2=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY2=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS2=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

```sh
resim set-default-account $ACC_ADDRESS1 $PRIV_KEY1
```

```sh
M_OP=$(resim run "./transactions/mint_usd.rtm")
export USD=$(echo "$M_OP" | sed -nr "s/└─ Resource: ([[:alnum:]_]+)/\1/p")
```

```sh
PK_OP=$(resim publish ".")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
CP_OP=$(resim run "./transactions/component_creation.rtm")
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/.* Component: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export DAO=$(echo "$CP_OP" | sed -nr "s/.* Component: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
export TT=$(echo "$CP_OP" | sed -nr "s/.* Resource: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
```

```
resim run ./transactions/set_liquidity_pool_address.rtm
```

```sh
export XRD=030000000000000000000000000000000000000000000000000004
```

## Creating our first governance proposal

```sh
CP=$(resim run "./transactions/propose.rtm")
export PROPOSAL_ID=$(echo "$CP" | sed -nr "s/.*The proposal ID is ([[:alnum:]_]+)/\1/p")
```

```sh
resim call-method $DAO check_proposal $PROPOSAL_ID
```

## Depositing towards our proposal

```sh
resim run ./transactions/deposit_proposal.rtm
```

## Voting for our proposal

### Voting Yes
```sh
VP=$(resim run "./transactions/vote_yes.rtm")
export PROOF=$(echo "$VP" | sed -nr "s/.*Your badge proof is ([[:alnum:]_]+)/\1/p")
```

### Voting No


### Recasting our vote

```sh
resim run ./transactions/recast_vote_no.rtm
```

### Resolving our proposal