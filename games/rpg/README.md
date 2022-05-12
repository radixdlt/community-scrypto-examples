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
  and the system badge (to authorize changing, minting, and burning all NFTs)
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
resim call-method $c upload_test_data
resim call-method $c upload_stage_data 1 10 7 10 10 10 7 10 10 10 7 10 10 0 0 0 5 0 0 0 5 0 0 0 0 1 1 6
resim call-method $c upload_stage_data 2 15 10 11 12 15 10 11 12 15 10 11 12 0 0 0 6 0 0 0 6 0 0 0 1 1 1 7
resim call-method $c upload_stage_data 3 17 12 12 15 17 12 12 15 17 12 12 15 0 0 0 8 0 0 0 8 0 1 1 1 1 1 10
resim call-method $c upload_stage_data 4 20 15 14 15 20 15 14 15 20 15 14 15 0 0 0 11 0 0 0 11 0 1 1 1 2 2 12
resim call-method $c upload_stage_data 5 25 15 18 17 25 15 18 17 25 15 18 17 0 0 0 13 0 0 0 14 0 1 1 2 2 2 15
resim call-method $c upload_stage_data 6 75 18 30 22 0 0 0 0 0 0 0 0 0 0 0 50 0 0 0 0 0 0 0 2 2 2 0
```
