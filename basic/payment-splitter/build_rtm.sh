# # Getting the current script dir
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

# Resetting resim
resim reset

# Creating some accounts
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export SHAREHOLDER1_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP3=$(resim new-account)
export SHAREHOLDER2_PRIV_KEY=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_PUB_KEY=$(echo "$OP3" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_ADDRESS=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP4=$(resim new-account)
export SHAREHOLDER3_PRIV_KEY=$(echo "$OP4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_PUB_KEY=$(echo "$OP4" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_ADDRESS=$(echo "$OP4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

# Setting the first account as the default account
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY

# Publishing the package
PK_OP=$(resim publish ".")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

# Building the component creation rtm as we will need to run it to get the component address
sed "s/<<<package_address>>>/$PACKAGE/g; s/<<<account1_address>>>/$ADMIN_ADDRESS/g" $SCRIPT_DIR/raw_transactions/component_creation.rtm > $SCRIPT_DIR/transactions/component_creation.rtm

# Creating a new payment splitter
CP_OP=`resim run "$SCRIPT_DIR/transactions/component_creation.rtm"`
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")

export ADMIN_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export INTERNAL_ADMIN_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')
export SHAREHOLDER_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')

echo "[•] Component: $COMPONENT"
echo "[•] Admin Badge: $ADMIN_BADGE"
echo "[•] Internal Admin Badge: $INTERNAL_ADMIN_BADGE"
echo "[•] Shareholder Badge: $SHAREHOLDER_BADGE"

# Building a lookup table of all of the things that we'll be replacing:
export REPLACEMENT_LOOKUP=" \
    s/<<<shareholder_badge>>>/$SHAREHOLDER_BADGE/g; \
    s/<<<admin_badge>>>/$ADMIN_BADGE/g;
    s/<<<splitter_component_address>>>/$COMPONENT/g; \
    s/<<<account1_address>>>/$ADMIN_ADDRESS/g; \
    s/<<<account2_address>>>/$SHAREHOLDER1_ADDRESS/g; \
    s/<<<account3_address>>>/$SHAREHOLDER2_ADDRESS/g; \
    s/<<<account4_address>>>/$SHAREHOLDER3_ADDRESS/g; \
    s/<<<splitter_component_address>>>/$COMPONENT/g; \
"

# Building the "adding shareholder" file
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/adding_shareholders.rtm > $SCRIPT_DIR/transactions/adding_shareholders.rtm

# Building the "funding the splitter" file
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/funding_the_splitter.rtm > $SCRIPT_DIR/transactions/funding_the_splitter.rtm

# Building the "withdraw owed amount" file
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/withdrawing_owed_amount.rtm > $SCRIPT_DIR/transactions/withdrawing_owed_amount.rtm