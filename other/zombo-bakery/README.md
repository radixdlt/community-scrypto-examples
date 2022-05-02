Zombo Bakery

Purpose:

In this example we will be minting a Baker NFT, purchasing ingredients (badges) to make bread, and feeding the bread to the Baker NFT.

How to use: 

1. resim reset
2. resim new-account
	2.1 export account=...
3. resim publish .
	3.1 export package=...
4. resim call-function package ZomboBakery new 50 100 
	4.1 export nft_minter=...
	4.2 export nft=…
	4.3 export material_minter=…
	4.4 export bread=…
	4.5 export butter=…
	4.6 export cake_pan=…
	4.7 export flour=…
	4.8 export water=…
	4.9 export yeast=...
	4.10 export zombo=…
	4.11 export component=...
5. resim show account 
	5.1 export xrd=...
6. resim show component - Component contains both minter badges and zombo
7. resim call-method component mint_nft 50,xrd
	7.1 resim show $account - account contains NFT with mutable data = 1

8. Methods such as `buy_flour`, `buy_water`, `buy_yeast` and `buy_butter` are authenticated methods which require that the baker non-fungible token is present in the auth-zone to authorize calls to these methods. Thus, we need to use a transaction manifest file to make use of the auth-zone. The following instructions may be used to call these four different methods on the component:

```sh
CALL_METHOD ComponentAddress("<Account Address>") "create_proof_by_amount" Decimal("1") ResourceAddress("<Baker Badge Resource Address>");
CALL_METHOD ComponentAddress("Account Address") "withdraw_by_amount" Decimal("4") ResourceAddress("<Zombo Token Resource Address>");

TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("1") ResourceAddress("<Zombo Token Resource Address>") Bucket("zombo_tokens_bucket_1");
CALL_METHOD ComponentAddress("<Component Address>") "buy_flour" Bucket("zombo_tokens_bucket_1");

TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("1") ResourceAddress("<Zombo Token Resource Address>") Bucket("zombo_tokens_bucket_2");
CALL_METHOD ComponentAddress("<Component Address>") "buy_water" Bucket("zombo_tokens_bucket_2");

TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("1") ResourceAddress("<Zombo Token Resource Address>") Bucket("zombo_tokens_bucket_3");
CALL_METHOD ComponentAddress("<Component Address>") "buy_yeast" Bucket("zombo_tokens_bucket_3");

TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("1") ResourceAddress("<Zombo Token Resource Address>") Bucket("zombo_tokens_bucket_4");
CALL_METHOD ComponentAddress("<Component Address>") "buy_butter" Bucket("zombo_tokens_bucket_4");

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("<Account Address>") "deposit_batch";
```

9. resim call-method $component make_bread 1,$flour 1,$water 1,$yeast
10. resim call-method $component eat_bread 1,$bread 1,$butter 1,$nft
11. resim show account - NFT mutable data = 2. Need mutable data = 3.  Repeat steps 8-13 one more time.
12. resim call-method $component next_level 1,$nft