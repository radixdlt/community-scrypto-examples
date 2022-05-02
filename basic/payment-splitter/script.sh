# Getting the current script dir
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo $SCRIPT_DIR

# Resetting resim
resim reset

# Creating some accounts
OP1=$(resim new-account)
export ADMIN_PRIV_KEY=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export ADMIN_PUB_KEY=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ADMIN_ADDRESS=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Created Admin Account: $ADMIN_ADDRESS"

OP2=$(resim new-account)
export SHAREHOLDER1_PRIV_KEY=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_PUB_KEY=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER1_ADDRESS=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Created Admin Account: $SHAREHOLDER1_ADDRESS"

OP3=$(resim new-account)
export SHAREHOLDER2_PRIV_KEY=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_PUB_KEY=$(echo "$OP3" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER2_ADDRESS=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Created Admin Account: $SHAREHOLDER2_ADDRESS"

OP4=$(resim new-account)
export SHAREHOLDER3_PRIV_KEY=$(echo "$OP4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_PUB_KEY=$(echo "$OP4" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export SHAREHOLDER3_ADDRESS=$(echo "$OP4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
echo "[•] Created Admin Account: $SHAREHOLDER3_ADDRESS"

# Setting the first account as the default account
resim set-default-account $ADMIN_ADDRESS $ADMIN_PRIV_KEY

# Publishing the package
PK_OP=$(resim publish ".")
export PACKAGE=$(echo "$PK_OP" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
echo "[•] Published Package: $PACKAGE"

# Creating a new payment splitter
resim run "$SCRIPT_DIR/transactions/component_creation.rtm"

# Adding shareholders to the payment splitter
resim run "$SCRIPT_DIR/transactions/adding_shareholders.rtm"

# Funding the payment splitter
resim run "$SCRIPT_DIR/transactions/funding_the_splittter.rtm"

# Switching to the first shareholder' account and withdrawing the funds.
resim set-default-account $SHAREHOLDER1_ADDRESS $SHAREHOLDER1_PRIV_KEY
resim run "$SCRIPT_DIR/transactions/withdrawing_owed_amount.rtm"