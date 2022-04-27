resim reset

OP1=$(resim new-account)
export privkey1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acct1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export privkey2=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acct2=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export usd=$(resim new-token-fixed 1000 --description "The American dollar" --name "dollar" --symbol "USD" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
export eur=$(resim new-token-fixed 1000 --description "The European euro" --name "euro" --symbol "EUR" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

resim transfer 500 $usd $acct2
resim transfer 500 $eur $acct2

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

CP_OP=$(resim call-function $package Transit new 10 1 $usd $eur)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")

export american_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export european_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')
export mint_auth=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')
export ticket=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '4!d')

resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$eur
resim call-method $component buy_ticket 10,$eur

resim transfer 1 $european_badge $acct2

resim set-default-account $acct2 $privkey2
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$usd
resim call-method $component buy_ticket 10,$eur
resim call-method $component buy_ticket 10,$eur

resim call-method $component ride 1,$ticket "European"
resim call-method $component ride 1,$ticket "American"

resim set-current-epoch 5
resim call-method $component ride 1,$ticket "American"
echo "CALL_METHOD ComponentAddress(\"$acct2\") \"create_proof\" ResourceAddress(\"$european_badge\");" > ./withdraw_euros.rtm
echo "CALL_METHOD ComponentAddress(\"$component\") \"withdraw_euros\" false;" >> ./withdraw_euros.rtm
echo "CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$acct2\") \"deposit_batch\";" >> ./withdraw_euros.rtm
resim run ./withdraw_euros.rtm
resim call-method $component ride 1,$ticket "European"

resim set-default-account $acct1 $privkey1
echo "CALL_METHOD ComponentAddress(\"$acct1\") \"create_proof\" ResourceAddress(\"$american_badge\");" > ./withdraw_dollars.rtm
echo "CALL_METHOD ComponentAddress(\"$component\") \"withdraw_dollars\" false;" >> ./withdraw_dollars.rtm
echo "CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$acct1\") \"deposit_batch\";" >> ./withdraw_dollars.rtm
resim run ./withdraw_dollars.rtm