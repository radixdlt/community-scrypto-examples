set -x

export xrd=030000000000000000000000000000000000000000000000000004

resim reset

OP1=$(resim new-account)
export privkey1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

CP_OP=$(resim call-function $package Insurance new 100,$xrd)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
export fundraiser_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export patron_mint_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')

CM_OP=$(resim call-method $component make_policy "property" 10 5 100 3)
export policy1=$(echo "$CM_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
resim call-method $component purchase $policy1 $acc1 5,$xrd
resim call-method $component approve $acc1 $policy1 10

resim set-current-epoch 101
resim call-method $component burn_purchases $acc1 $policy1
resim call-method $component burn_policies $policy1

resim call-method $component deposit 100,$xrd
resim call-method $component withdraw 50
resim call-method $component assets
resim call-method $component locked