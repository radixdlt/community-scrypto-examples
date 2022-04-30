# Getting the current script dir
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Resetting resim
resim reset

# Creating two accounts for the admin and beneficiary
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export BENEFICIARY_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

# Create a new vesting component through the admin account
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY

# Creating a new token to use for the vesting contract
TK_OP=$(resim run $SCRIPT_DIR/transactions/token_creation.rtm)
export USDT=$(echo "$TK_OP" | sed -nr "s/└─ Resource: ([[:alnum:]_]+)/\1/p")

# Publishing the package to resim
PK_OP=$(resim publish "$SCRIPT_DIR")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

CP_OP=$(resim call-function $PACKAGE Vesting instantiate_vesting)
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
export INTERNAL_ADMIN_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export ADMIN_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')
export BENEFICIARY_BADGE=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')

# Building the lookup table
export REPLACEMENT_LOOKUP=" \
    s/<<<account1_address>>>/$ADMIN_ADDRESS/g; \
    s/<<<account2_address>>>/$BENEFICIARY_ADDRESS/g; \
    s/<<<admin_badge>>>/$ADMIN_BADGE/g; \
    s/<<<usdt_token>>>/$USDT/g; \
    s/<<<vesting_component_address>>>/$COMPONENT/g; \
    s/<<<beneficiary_badge>>>/$BENEFICIARY_BADGE/g; \
"

# Replacing the parts from the lookup table
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/add_beneficiary.rtm > $SCRIPT_DIR/transactions/add_beneficiary.rtm
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/disable_termination.rtm > $SCRIPT_DIR/transactions/disable_termination.rtm
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/terminate_beneficiary.rtm > $SCRIPT_DIR/transactions/terminate_beneficiary.rtm
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/token_creation.rtm > $SCRIPT_DIR/transactions/token_creation.rtm
sed "$REPLACEMENT_LOOKUP" $SCRIPT_DIR/raw_transactions/withdraw_funds.rtm > $SCRIPT_DIR/transactions/withdraw_funds.rtm
