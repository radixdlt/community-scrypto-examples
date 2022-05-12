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
This generates data for a character of class 1, as well as stage data for the first six stages. Simply alter the digits according to the comments to add more stages/change stages.
```
resim call-method $c upload_test_data
resim call-method $c upload_stage_data 1 10 7 10 10 10 7 10 10 10 7 10 10 0 0 0 5 0 0 0 5 0 0 0 0 1 1 6
resim call-method $c upload_stage_data 2 15 10 11 12 15 10 11 12 15 10 11 12 0 0 0 6 0 0 0 6 0 0 0 1 1 1 7
resim call-method $c upload_stage_data 3 17 12 12 15 17 12 12 15 17 12 12 15 0 0 0 8 0 0 0 8 0 1 1 1 1 1 10
resim call-method $c upload_stage_data 4 20 15 14 15 20 15 14 15 20 15 14 15 0 0 0 11 0 0 0 11 0 1 1 1 2 2 12
resim call-method $c upload_stage_data 5 25 15 18 17 25 15 18 17 25 15 18 17 0 0 0 13 0 0 0 14 0 1 1 2 2 2 15
resim call-method $c upload_stage_data 6 75 18 30 22 0 0 0 0 0 0 0 0 0 0 0 50 0 0 0 0 0 0 0 2 2 2 0
```
6. Create your character. Since only the data for class 1 is uploaded, use that. (Making more classes is too much work)
You must pass in xrd according to your set game.price as of instantiation. I'd avoid making more than one charcter as it'll confuse you a lot.
```
resim call-method $c create_character 100,$xrd 1
```
7. Before you fight, export the NFT IDs of everything you got when you created a character.
```
resim show $a
weapon=[ResourceAddress of the NFT with string "Stick"]
helmet=[ResourceAddress of the NFT with string "Helmet"]
chest=[ResourceAddress of the NFT with string "Chest"]
pants=[ResourceAddress of the NFT with string "Pants"]
gloves=[ResourceAddress of the NFT with string "Gloves"]
belt=[ResourceAddress of the NFT with string "Belt"]
shoes=[ResourceAddress of the NFT with string "Shoes"]
```
8. Okay now you can fight.
Upon running the transaction, your character will simulate fighting the enemies with their stats set earlier as of step 5
You will return with your NFT having gotten some EXP according to their preformance, as well as some tokens. Those tokens will be used to craft better gear.
Because damage is randomly generated within a range on each attack, repeating the same stage can have a different outcome!
These early test stages are really easy though so you shouldn't be losing
```
resim call-method $c stage 1,$char "#$weapon,#$helmet,#$chest,#$pants,#$gloves,#$belt,#$shoes,$item" 1
resim show $a
```
Want to do other stages? Simply change the last parameter of the method. Stage 2:
```
resim call-method $c stage 1,$char "#$weapon,#$helmet,#$chest,#$pants,#$gloves,#$belt,#$shoes,$item" 2
resim show $a
```
9. Get Stronger!
By fighting stages, your character will naturally accumulate EXP and level up after combat. This will boost your stats. But how do you get better gear?
You craft it. Once you have enough materials from stages, craft some gear.
Each piece of gear costs 1 gold, 1 greavite, and 1 wood for now.
Make sure you re-export the NFTID of that gear if you want to use it!
|**NOTE**| Lines are as follows: 

Crafts Sword

Crafts Helmet

Crafts Chest

Crafts Pants

Crafts Gloves

Crafts Belt

Crafts Shoes|
|----|-----|
Lines are as follows: 

Crafts Sword

Crafts Helmet

Crafts Chest

Crafts Pants

Crafts Gloves

Crafts Belt

Crafts Shoes

```
resim call-method $c create_weapon_1 1,$gold 1,$greavite 1,$wood 1
resim call-method $c create_armor_1 1,$gold 1,$greavite 1,$wood 1
resim call-method $c create_armor_1 1,$gold 1,$greavite 1,$wood 2
resim call-method $c create_armor_1 1,$gold 1,$greavite 1,$wood 3
resim call-method $c create_accessory_1 1,$gold 1,$greavite 1,$wood 1
resim call-method $c create_accessory_1 1,$gold 1,$greavite 1,$wood 2
resim call-method $c create_accessory_1 1,$gold 1,$greavite 1,$wood 3
resim show $a
```
Crafting stats are randomized between 75% and 125% of base, so craft multiple items for good rolls!

10. Get EVEN Stronger!
Items can be fused together, granting a 20% increase to stats. This stacks! Make sure the items you're fusing together are the same type, and the same level.

For example, a lvl 2 weapon requires two lvl 1 weapons. A lvl 5 weapon requires 2 lvl 4s, which is 4 lvl 3s, which is 8 lvl 2s, which is 16 lvl 1 weapons.

For fusing Weapons, `number` = 1. For fusing Armor, `number` = 2 For fusing Accessories, `number` = 3
```
resim call-method $c fuse_items number "#$Weapon1NFTID,#$Weapon2NFTID,$item"
```
The stats of the upgraded weapon is based on the NFT ID of the first Weapon. `Weapon1NFTID`
