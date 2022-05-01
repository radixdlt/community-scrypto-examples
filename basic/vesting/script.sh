# Getting the current script dir
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

# Resetting resim
resim reset

# Creating two accounts for the admin and beneficiary
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Admin Private Key: $ADMIN_PRIV_KEY"
echo "[•] Admin Public Key: $ADMIN_PUB_KEY"
echo "[•] Admin Address: $ADMIN_ADDRESS"

OP2=$(resim new-account)
export BENEFICIARY_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export BENEFICIARY_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Beneficiary Private Key: $BENEFICIARY_PRIV_KEY"
echo "[•] Beneficiary Public Key: $BENEFICIARY_PUB_KEY"
echo "[•] Beneficiary Address: $BENEFICIARY_ADDRESS"

# Create a new vesting component through the admin account
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY

# Creating a new token to use for the vesting contract
TK_OP=$(resim run $SCRIPT_DIR/transactions/token_creation.rtm)
export USDT=$(echo "$TK_OP" | sed -nr "s/└─ Resource: ([[:alnum:]_]+)/\1/p")
echo "[•] Created USDT Resource: $USDT"

# Publishing the package to resim
PK_OP=$(resim publish "$SCRIPT_DIR")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
echo "[•] Published package: $PACKAGE"

CP_OP=$(resim call-function $PACKAGE Vesting instantiate_vesting)
export COMPONENT=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
echo "[•] Vesting component: $COMPONENT"
echo $CP_OP

# Adding the beneficiary to the vesting component
resim run "$SCRIPT_DIR/transactions/add_beneficiary.rtm"

# Checking if the beneficiary can withdraw any funds before the cliff epoch
echo "[•] Setting the current epoch to 10."
resim set-current-epoch 10
resim set-default-account $BENEFICIARY_ADDRESS $BENEFICIARY_PRIV_KEY
resim run "$SCRIPT_DIR/transactions/withdraw_funds.rtm"

# Checking if the beneficiary can withdraw the correct amount of tokens on cliff
echo "[•] Setting the current epoch to 20."
resim set-current-epoch 20
resim run "$SCRIPT_DIR/transactions/withdraw_funds.rtm"

# Checking if the beneficiary can withdraw the correct amount of tokens in the middle of vesting
echo "[•] Setting the current epoch to 50."
resim set-current-epoch 50
resim run "$SCRIPT_DIR/transactions/withdraw_funds.rtm"

# Checking if the beneficiary can withdraw twice in the same epoch
resim run "$SCRIPT_DIR/transactions/withdraw_funds.rtm # Returned balance should be zero because we have withdrawn all."

# Terminating the beneficiary's vesting schedule
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY
resim run "$SCRIPT_DIR/transactions/terminate_beneficiary.rtm"

# Giving-up admin rights to vesting termination
resim run "$SCRIPT_DIR/transactions/disable_termination.rtm"