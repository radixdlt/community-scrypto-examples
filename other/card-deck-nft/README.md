# CARD-DECK-NFT

This is a very basic implementation to mint an NFT with a bit more complex structure.

The idea here is to create an NFT game asset that can be used to play a standard-52 card game from within or another blueprint that will utilize its structure.

I have included some basic calls to demonstate the usage of the data field in the struct.

The code has lots of beginner friendly comments to help guide other new aspiring Scrypto devs.

This was written in **_Scrypto "v0.9.0"_**

---

## Features

- The main Struct **Deck** contains a field with a vector another Struct **Card**

```
pub struct Card {
    suit: Suit,
    value: u8,
}
pub struct Deck {
    #[mutable]
    pub cards: Vec<Card>,
}
```

- Human Friendly debug output display

```
Logs: 1
└─ [INFO ] Your Deck of Cards (52 cards): A♦️, 2♦️, 3♦️, 4♦️, 5♦️, 6♦️, 7♦️, 8♦️, 9♦️, 10♦️, J♦️, Q♦️, K♦️, A♥️, 2♥️, 3♥️, 4♥️, 5♥️, 6♥️, 7♥️, 8♥️, 9♥️, 10♥️, J♥️, Q♥️, K♥️, A♣️, 2♣️, 3♣️, 4♣️, 5♣️, 6♣️, 7♣️, 8♣️, 9♣️, 10♣️, J♣️, Q♣️, K♣️, A♠️, 2♠️, 3♠️, 4♠️, 5♠️, 6♠️, 7♠️, 8♠️, 9♠️, 10♠️, J♠️, Q♠️, K♠️

Logs: 1
└─ [INFO ] Shuffled Deck:  5♦️ 6♦️ 7♦️ 8♦️ 9♦️ 10♦️ J♦️ Q♦️ A♦️ 2♦️ 3♦️ 4♦️ Q♥️ K♥️ A♣️ 2♣️ 3♣️ 4♣️ 5♣️ 6♣️ 7♣️ 8♣️ 9♣️ 10♣️ J♣️ Q♣️ K♣️ A♠️ 2♠️ 3♠️ 4♠️ 5♠️ 6♠️ 7♠️ 8♠️ 9♠️ 10♠️ J♠️ Q♠️ K♠️ K♦️ A♥️ 2♥️ 3♥️ 4♥️ 5♥️ 6♥️ 7♥️ 8♥️ 9♥️ 10♥️ J♥️

Logs: 1
└─ [INFO ] Your hand 5 cards:  A♠️ J♥️ 3♣️ 4♠️ 8♥️
```

---

## Usage

### **_Manual Usage_**

Run each line (one at a time) at the terminal to execute the command.
You dont have to export [some variable]=value each time.
Running each line of code will automatically save it into the variable to which you can call

```
echo $[variable name]
```

```
export xrd=resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k
resim reset
OP1=$(resim new-account)
export account=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
PB_OP=$(resim publish .)
export package=$(echo "$PB_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
CP_OP=$(resim call-function $package NFTcards instantiate_new)
export comp=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")

MINT_OP=$(resim call-method $comp mint_deck $xrd:5 | tail -c 600)
nft=$(echo "$MINT_OP" | grep -oP ', Address: \K[^\n]*(?=, Delta: \+{#)')

resim call-method $comp shuffle_cards $nft:1
resim call-method $comp deal $nft:1 5
```

### **_Automated Usage_** using scripts

Make sure the file **_instantiate.sh_** is in the root directory.

Navigate to your root directory.

Execute this command on your bash terminal

```
chmod +x myscript.sh
```

This allows the script to be executable (absolutely necessary). Then execute the following command in your terminal.

```
./instantiate.sh
```

You can then run other methods from there.

---

## Callable Methods

### **Instantiates the blueprint.**

```
resim call-function [package address] NFTcards instantiate_new
```

### **Mints a new NFT and sends to your wallet**

NOTE: That this will only take in XRD as payment and mint cost of 5 XRD

```
resim call-method [component address] mint_deck [radix resource address]:5
```

```
Logs: 1
└─ [INFO ] Your Deck of Cards (52 cards): A♦️, 2♦️, 3♦️, 4♦️, 5♦️, 6♦️, 7♦️, 8♦️, 9♦️, 10♦️, J♦️, Q♦️, K♦️, A♥️, 2♥️, 3♥️, 4♥️, 5♥️, 6♥️, 7♥️, 8♥️, 9♥️, 10♥️, J♥️, Q♥️, K♥️, A♣️, 2♣️, 3♣️, 4♣️, 5♣️, 6♣️, 7♣️, 8♣️, 9♣️, 10♣️, J♣️, Q♣️, K♣️, A♠️, 2♠️, 3♠️, 4♠️, 5♠️, 6♠️, 7♠️, 8♠️, 9♠️, 10♠️, J♠️, Q♠️, K♠️
```

---

### **Shuffles and display the cards**

NOTE: "[nft resource address]:1" this means that you are passing the NFT address to the component and 1 means that you are sending 1 NFT (This tripped me a lot at first when I was learning and I hope this helps you as well).

```
resim call-method [component address] shuffle_cards [nft resource address]:1
```

```
Logs: 1
└─ [INFO ] Shuffled Deck:  5♦️ 6♦️ 7♦️ 8♦️ 9♦️ 10♦️ J♦️ Q♦️ A♦️ 2♦️ 3♦️ 4♦️ Q♥️ K♥️ A♣️ 2♣️ 3♣️ 4♣️ 5♣️ 6♣️ 7♣️ 8♣️ 9♣️ 10♣️ J♣️ Q♣️ K♣️ A♠️ 2♠️ 3♠️ 4♠️ 5♠️ 6♠️ 7♠️ 8♠️ 9♠️ 10♠️ J♠️ Q♠️ K♠️ K♦️ A♥️ 2♥️ 3♥️ 4♥️ 5♥️ 6♥️ 7♥️ 8♥️ 9♥️ 10♥️ J♥️
```

---

### **Deal the cards**

NOTE: Take note of the additional arguement at the end "5". This method takes an additional arguement **_N_** number of cards to DEAL to you.

```
resim call-method [component address] deal [nft resource address]:1 5
```

If you look at the log after you run the command you should see this:

```
Logs: 1
└─ [INFO ] Your hand 5 cards:  A♠️ J♥️ 3♣️ 4♠️ 8♥️
```

---

## Acknowledgement

By no means that is great code but I hope it helps better show and example of NFT minting as an example.

I plan on working with this more and refining it.

I apologize of having so many comments in the code.
This was meant to help people understand the code and making it easier to digest.


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