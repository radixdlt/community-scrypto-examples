# Substradix: A fully blockchain based autobattler RPG
The structure of the game is heavily based on Nine Chronicles (https://nine-chronicles.com/), a Blockchain MMOPRG. 
However, I haven't looked at their code or anything so my implementation is likely very different in many ways. I also haven't played in almost a year lol.

# How to play the game:

First, download the files for the game. No special crates are required besides the crates for scrypto. 

In a terminal:

1. Reset the simulator
```
resim reset
```
2. Create a new account to administer the RNS component. Save the account address to `$admin_account`
```
resim new-account
```
3. Publish the package. Save the package address to `$package`
```
resim publish .
```

