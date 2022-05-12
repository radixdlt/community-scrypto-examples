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
2. Create a new account to administer the component. Save the account address to `$a`. Additionally, save the radix token address as `$xrd`
```
resim new-account
xrd=030000000000000000000000000000000000000000000000000004
```
3. Publish the package. Save the package address to `$p`
```
resim publish .
```
4. Instantiate a Substradix Component. You must specific the price to create a character as a parameter.
  Save the necessary ResourceAddresses for later 
  The first two ResourceAddresses generated are for the developer badge (to control system badges)
  and the system badge (too authorize changing, minting, and burning all NFTs)
```
resim call-function $p Substradix new 100
char=[Third Resource Address generated]
item=[Fourth Resource Address generated]
gold=[Fifth Resource Address generated]
greavite=[Sixth Resource Address generated]
wood=[Seventh Resource Address generated]
```
5. Input the data for characters and stages. Sample methods/commands are provided, but you are welcome to change anything of your own.
```
resim call-method $p Substradix new 100
```
