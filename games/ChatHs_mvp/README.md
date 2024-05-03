# ChatHs-mvp
## Introduction
This is an MVP-version of a component that relays prompts to writers and holds payments in escrow until their responses are submitted within a set timeframe. The prompters' payment is initially set at 20XRD and the response must be submitted within 24 hours for the writer to claim it. However, as additional prompts are added to the stack, the required payment doubles and the timeframe is extended by a further 24 hours each time. As each active 24 hours elapses, the required payment is reduced again accordingly.
## Writer
The writer instantiates the component from the blueprint with a 'bio' and awaits prompts. Instantiation returns a soulbound 'account' NFT required to respond to prompts.

Prompt NFTs are minted by the component upon receipt of the prompt and the appropriate payment. The writer can edit each NFT to add the necessary response within the required timeframe. This action requires the 'account' NFT. This action returns the payment received from the prompter minus a 10% commission.
## Prompter
The prompter sends a prompt together with the appropriate payment. The ChatHs component returns a 'receipt' NFT. With the 'receipt', the prompter can later claim the NFT comprising the completed 'prompt-and-response'. Alternatively, the prompter can claim a refund in the event that the writer does not submit a valid response within the required timeframe.
## Runthrough <br>
Reset resim with `resim reset` and make a new account with `resim new-account`. Then, publish the package with `resim publish .`. Note your account address and the package address.

To create a ChatHs account with resim, use:

`resim call-function [your package address] ChatHs instantiate_with_bio [your bio]`

Alternatively, to do so with a transaction manifest use:

```js
CALL_METHOD
    Address("your account component address")
    "lock_fee"
    Decimal("100");
CALL_FUNCTION
    Address("your package address")
    "ChatHs"
    "instantiate_with_bio"
    "your bio your bio your bio your bio your bio your bio your bio your bio your bio your bio your bio your bio your bio your";
CALL_METHOD
    Address("your account component address")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
```
This will instantiate the 'ChatHs account component' and return the soulbound NFT necessary to respond to prompts. The resources generated are, in order, the 'soulbound ChatHs account badge', the 'Internal NFT Mint Badge', the 'Exchange NFT' and the 'Receipt NFT'. 

Make a note of the addresses of the 'ChatHs account component', your 'soubound ChatHs account badge' and your 'Receipt NFT'.

To send a prompt with resim use:

`resim call-method [your ChatHs account component address] prompt resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:20 [your prompt]`

Alternatively, with a transaction manifest:

```js
CALL_METHOD
    Address("your account component address")
    "lock_fee"
    Decimal("100");
CALL_METHOD
    Address("your account component address")
    "withdraw"
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Decimal("20");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("20")
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Bucket("bucket1");
CALL_METHOD
    Address("your ChatHs account component address")
    "prompt"
    Bucket("bucket1")
    "your prompt your prompt your prompt your prompt your prompt your prompt your prompt your prompt your prompt your prompt your";
CALL_METHOD
    Address("your account component address")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
```
Note that it required 20XRD to send a prompt, but if you send another at this point it will require 40XRD, then 80XRD, and so on.

You will receive receipt NFT for each prompt you send. Each receipt NFT has the NonFungibleLocalId of the 'Exchange NFT' corresponding to your prompt. To get this information, use `resim show [your account]`. The one you're after is contained in the 'immutable_data' bit of the receipt data. The other one, which you don't need, is the NonFungibleLocalId of the receipt itself.

Note that the receipt data also includes the 'due_time' (as a u64) at which a response is required. Before you respond to a prompt, you may adjust resim's epoch with `set-current-epoch`. However, if you set it after the due_time, or more than '24' before, you will not be able to respond.

To respond to a prompt with resim use:

`resim call-method [your ChatHs account component address] respond ("{your prompt NonFungibleLocalId in curly brackets}") [your response] --proofs [your soulbound 'ChatHs account badge' resource address]:1`

Alternatevely, with a manifest:

```js
CALL_METHOD
    Address("your account component address")
    "create_proof_by_amount"
    Address("your soulbound ChatHs account badge' resource address")
    Decimal("1");
CALL_METHOD
    Address("your account component address")
    "lock_fee"
    Decimal("100");
CALL_METHOD
    Address("your ChatHs account component address")
    "respond"
    NonFungibleLocalId("{your prompt NonFungibleLocalId in curly brackets}")
    "your response your response your response your response your response your response your response your response your";
CALL_METHOD
    Address("your account component address")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
```
This will return the payment minus 10% commission.

To be able to claim a completed prompt-and-response 'Exchange NFT' you will need to adjust resim's epoch to after its due_time with `set-current-epoch` (eg. `resim set-current-epoch 26`). Then, send the 'receipt' NFT to the ChatHs component. To do this with resim:

`resim call-method [your ChatHs account component address] claim_nft [your receipt resource address]:1`

In a manifest:

```js
CALL_METHOD
    Address("your account component address")
    "lock_fee"
    Decimal("100");
CALL_METHOD
    Address("your account component address")
    "withdraw"
    Address("your receipt resource address")
    Decimal("1");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("1")
    Address("your receipt resource address")
    Bucket("bucket1");
CALL_METHOD
    Address("your ChatHs account component address")
    "claim_nft"
    Bucket("bucket1");
CALL_METHOD
    Address("your account component address")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
```
You will now receive your completed prompt-and-response 'Exchange NFT'.

If an 'Exchange NFT' is not completed in the appropriate timeframe, you can claim a refund. In resim:

`resim call-method [your ChatHs account component address] claim_refund [your receipt resource address]:1`

With a manifest:

```js
CALL_METHOD
    Address("your account component address")
    "lock_fee"
    Decimal("100");
CALL_METHOD
    Address("your account component address")
    "withdraw"
    Address("your receipt resource address")
    Decimal("1");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("1")
    Address("your receipt resource address")
    Bucket("bucket1");
CALL_METHOD
    Address("your ChatHs account component address")
    "claim_refund"
    Bucket("bucket1");
CALL_METHOD
    Address("your account component address")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
```


## Planned enhancements

### Frontend
It is anticipated that a frontend website will index all the accounts (all the ChatHs components instantiated from the ChatHs blueprint), all the prompts, and all the responses. The website should provide a means to create transactions to create accounts, submit prompts and responses, and claim NFTs or refunds. It should also display prompts and responses for individuals browsing the site. Any NFT-trading could happen elsewhere.

The 10% commission charged by ChatHs components upon completion of 'prompt-and-response' NFTs is necessary in order to deter individuals from submitting prompts to their own ChatHs component. It may also provide sufficient funding to maintain the website.

A component to receive the commissions will be instantiated before the ChatHs blueprint is added to the blueprint catalog, and its address hardcoded into the blueprint.
### Wordcounts
Wordcounts are not implemented in the MVP. However, it is anticipated that a 'bio' will only be accepted if it's 1-300 words, prompts will only be accepted up to 100 words and responses will only be accepted if they are between 3-500 words.
### Payments
In the MVP, payments required to submit a prompt begin at 20XRD and double from there. It is anticipated that required payments will start at $20 (though still be payable in XRD). An manual update-price function or oracle will be employed to set the base-payment.
### Timestamps
The MVP uses epoch time but this will be changed to UTC date time.


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