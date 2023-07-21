set -x

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
