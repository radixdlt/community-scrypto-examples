# Real Estate Manager: Permissioned, Secure Real Estate Manager Protocol

This is a Permissioned, Secure Real Estate Manager Protocol blueprint package built on Radix Ledger to provide real estate manage solution for authorities.

## Main Features

The blueprint package (included 3 blueprints) is for authorities of countries or local communities *(Hereby refer as **authorities**)* to manage, keep track of ***real estate deeds*** and provide ***real estate services*** for authorized citizens of that communities by making use of Non Fungible Tokens (NFTs). 

- Protocol entities:
1. **Authorities**: The main managers of the protocol, are allowed to authorize new citizen, market place or construction institute, also can authorize land modification, collect and edit tax.
2. **Citizens**: The main users of the protocol, are allowed to request land modification, request construction or trade real estate NFTs.
3. **Market hosts**: The main managers of the Market Place Components, are allowed to collect and edit market fee.
4. **Construction institutes**: The main managers of the Construction Institute Components, are allowed to authorize construction, collect and edit construction service fee.

- Core blueprint ***Real Estate Service***:
1. Use Soul Bound Tokens (SBTs) to include a simple KYC solution for authorities to keep track of their citizens. Citizen's personal data is not recorded on ledger to ensure privacy.
2. Included a method to authorize new land and create associated NFT when the protocol is first implemented on the communities.
3. Included methods for citizens to request land modify (*divide land* or *merge land*) and get the *modify right* NFT of their land to conduct the land modify.
4. Included a method for authorities to review the citizen's land modify requests and give them *modify right* NFT so they are allowed to modify their land.
5. Included methods for authorities to authorize new *market places* or *construction institutes*.
6. Included methods for authorities to collect tax, edit tax of their services.

- Blueprint ***Real Estate Market Place***:
1. After authorized, market hosts can run market places and earn market fee.
2. Included methods for citizens to buy, sell real estates.
3. Included methods for market hosts to collect, edit market fee.

- Blueprint ***Real Estate Construction Institute***:
1. After authorized, construction institutes can earn construction service fee.
2. Included methods for citizens to request construction (*construct a building* or *demolish a building*) and get the *construction right* NFT on their land to conduct the construction.
3. Included a method for institutes to review the citizen's construction requests and give them *construction right* NFT so they are allowed to conduct the construction.
4. Included methods for institutes to collect, edit service fee.

## Quick Start

*For windows user:* if you has git installed with [git bash](https://www.stanleyulili.com/git/how-to-install-git-bash-on-windows/) and [VSCode](https://code.visualstudio.com/), you should be able to run .sh file through git bash

![](gitbash.PNG)

Clone this git repository: `git clone https://github.com/unghuuduc/community-scrypto-examples && cd defi/real-estate-manager`

### Backend quick testing

1. Build the package: `scrypto build`
2. Quick end to end testing: `cd test && ./end_to_end.sh`

## Logic Explaination

Learn about the Real Estate Manager package: `cargo doc --no-deps --document-private-items --open`

### Use of a Medium Token:
The medium token which used in the whole Real Estate Manager protocol will be a token of the user (authority) choose.
If this protocol ever be used at country governments level, the authorities would consider using their CBDC as medium token.

### Simple, Secure, Private KYC by SBTs:
The SBT is locked in the citizen's wallet forever after their wallet have been KYCed.
The SBT not contain any data other than the NFT id, authorities can use that id to keep track of their citizen's private information.
This ensure the security of citizens information unless the citizens or authorities reveal the information.
The protocol is permissioned and require the KYC SBT to use. Anyone who isn't a part of the community cannot use this protocol.
*Real estate NFTs*, *modify right NFT* or *Trade order NFT* cannot be used by any account doesn't have the SBT.

### Real estate NFTs
Real estate deeds of the community will be managed by 2 type of NFT to prove the *owner rights* of citizens who have the NFT on their wallet: 
- Land Right's NFT: The NFT contain land information, included land's location, land's size and the building NFT id on that land (if the land contain a building).
- Building Right's NFT: The NFT contain building information, included building's size, building's floor.
1 land NFT can only contain 1 building NFT, this is for easier management, as well as trading on the market. If citizens want to build another building on their land, they will have to divide the land first.
Organizations can group their real estate NFTs however they want, this also help them to easily show which buildings, lands belong to the organizations.

### Real Estate Modify Right NFTs
There are 2 types of real estate *modify right* NFT on the protocol:
- Land modify right NFT: used on the ***Real Estate Service*** core blueprint, to authorize citizen's land modification.
- Construction right NFT: used on the ***Real Estate Construction Institute*** blueprint, to authorize citizen's building construction or demolition.

### Land Modify or Building Construct Authorization
Any authorized (KYCed) citizen can request a land modification or building construction (or demolition).
To review citizen requests on land modification or construction, authorities can make use of an **Oracle protocol** to feed in their map data and show the modification, construction is **feasible** or not.
- For land divide request: the oracle data must show the divide line is not overlapped with any existed real estate or construction.
- For land merge request: the oracle data must show that the two provided land is next to each other.
- For building construction or demolition request: the oracle data (or manually feeded data) must show that the construction (or demolition) will not cause harm to society (harm to the environment or any other damage).
After authorization, authorities will return the modify right's badge or just reject and delete the request.

### Real Estate Market Place
Working mostly the same as a NFT market place, making use of an *order book* which keep track of NFT's order.
The real estate market place is permissioned, only authorized citizens of the community can use the market. Likewise, their real estate NFTs can only be traded through these authorized markets (to prevent trading on "black market" or any 3rd party market).
After made a sell order, the maker received a *order NFT* to keep track of their order. Later they can use this NFT to cancel the order and take back their real estate NFTs or to take their payment after the order has been filled.

## Security, Utility

### What bad things won't happend on Real Estate Manager?

**Privacy Concern**: Citizen personal data is stored by the authority, there will be no privacy concern unless citizen or the authority reveal the information.

**Identity Concern**: The protocol is permissioned, only allow authorized account which has a clear identity.

### What bad things might happend on Real Estate Manager?

**Real Estate NFTs transfer to an Unauthorized Account**: Unfortunately, current Scrypto version don't have access rule to restrict which account can have a specific resource, so citizens can still deposit their real estate NFTs to unauthorized accounts through the protocol (specifically only through the ***buy method*** of the ***Real Estate Market Place*** protocol).

Although such a bad thing might happend on the protocol, it wouldn't be a critical problem. Authorities or any organization use the citizen's account to verify the real estate NFTs can also do the extra work of verify their KYC SBT. Moreover, the *mis-transfered* real estate NFTs cannot be withdrawn ever again from the unauthorized account unless they are ***recalled*** by authorities or the unauthorized person request a KYC himself.

**Lost of Property Deeds**: The property deeds is keeping tracked and managed by a permissioned protocol on Radix Ledger (instead of real papers). The deeds will never be lost unless the Radix Ledger stop functioning.

**Stealed, Hacked Property Deeds**: The property deeds cannot be stealed or hacked unless the Radix Ledger has security break (a >66% staked amount attack).

### What can't this Real Estate Manager do yet?

**Real estate lending**, **Renting** and **Mortgage**: Will included in future versions.

## P/s

*I'm still an amateur on cryptography and distributed technology, this work may still contain something wrong or haven't taken into account. I'm glad to have any contributor to this work.*

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