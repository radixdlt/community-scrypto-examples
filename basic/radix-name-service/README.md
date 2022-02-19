# Radix Name Service (RNS)
This is a very basic implementation of a DNS on Radix. It is conceptually similar to and heavily inspired by the ENS project: https://ens.domains

Something like the RNS would typically be integrated with wallets, to allow users to send funds to a human readable addresses like e.g. `dan.xrd` instead of a cryptic and long ledger addresses like e.g. `02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b`.

# Usage 
An example for how the RNS may be used, can be viewed in the [rns.revup](rns.revup) file.
You can execute the code in that file by calling the following command. Note that you need to have the [revup](https://github.com/RadGuild/revup) utility by @RockHoward installed.
```
revup -r rns.revup
```
