resim reset

# Create test account
ACCOUNT_DETAILS=$(resim new-account)
pubkey=$(echo "$ACCOUNT_DETAILS" | grep "Public key" | cut -d " " -f3)
account=$(echo "$ACCOUNT_DETAILS" | grep "Account component address" | cut -d " " -f4)
xrd=$(resim show "$account" | grep XRD | cut -d " " -f7 | cut -d "," -f1)
export xrd
export pubkey
export account

# Build app
scrypto build
package=$(resim publish . | grep Package: | cut -d " " -f4)
export package

# Create instance of app
component=$(resim call-function "$package" VirtualXrd new | grep Component: | cut -d " " -f3)
export component

#resim call-method "$component" swap_xrd_for_exrd 10,"$xrd"