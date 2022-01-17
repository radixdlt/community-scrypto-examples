resim reset

echo -e "\n\nCreate test admin account"
ADMIN_ACCOUNT_DETAILS=$(resim new-account)
admin_pubkey=$(echo -e "\n\n$ADMIN_ACCOUNT_DETAILS" | grep "Public key" | cut -d " " -f3)
admin_account=$(echo -e "\n\n$ADMIN_ACCOUNT_DETAILS" | grep "Account address" | cut -d " " -f3)
export admin_pubkey
export admin_account

echo -e "\n\nCreate test account 1"
ACCOUNT_1_DETAILS=$(resim new-account)
pubkey_1=$(echo -e "\n\n$ACCOUNT_1_DETAILS" | grep "Public key" | cut -d " " -f3)
account_1=$(echo -e "\n\n$ACCOUNT_1_DETAILS" | grep "Account address" | cut -d " " -f3)
export pubkey_1
export account_1

echo -e "\n\nCreate test account 2"
ACCOUNT_2_DETAILS=$(resim new-account)
pubkey_2=$(echo -e "\n\n$ACCOUNT_2_DETAILS" | grep "Public key" | cut -d " " -f3)
account_2=$(echo -e "\n\n$ACCOUNT_2_DETAILS" | grep "Account address" | cut -d " " -f3)
export pubkey_2
export account_2

echo -e "\n\nStore XRD address as variable"
xrd=$(resim show "$admin_account" | grep XRD | cut -d " " -f6 | cut -d "," -f1)
export xrd

echo -e "\n\nBuild app"
scrypto build
package=$(resim publish . | grep Package | cut -d " " -f3)
export package

echo -e "\n\nCreate instance of app"
resim set-default-account "$admin_account" "$admin_pubkey"
lending_pool=$(resim call-function "$package" LendingPlatform new | grep Component | tail -1 | cut -d " " -f3)
lending_platform_admin_badge=$(resim show "$admin_account" | grep 'Lending Platform Admin Badge' | cut -d ' ' -f6 | cut -d "," -f1)
export lending_platform_admin_badge
export lending_pool

echo -e "\n\nAdd XRD to lending pool"
resim set-default-account "$admin_account" "$admin_pubkey"
resim call-method "$lending_pool" new_asset "$xrd" 0.85 1,"$lending_platform_admin_badge"

echo -e "\n\nRegister User 1"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" new_user
lending_platform_badge_1=$(resim show "$account_1" | grep 'Lending Platform Badge' | cut -d ' ' -f6 | cut -d "," -f1)
export lending_platform_badge_1

echo -e "\n\nRegister User 2"
resim set-default-account "$account_2" "$pubkey_2"
resim call-method "$lending_pool" new_user
lending_platform_badge_2=$(resim show "$account_2" | grep 'Lending Platform Badge' | cut -d ' ' -f6 | cut -d "," -f1)
export lending_platform_badge_2

echo -e "\n\nDeposit asset User 1 - Deposit 1"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" deposit_asset 1,"$lending_platform_badge_1" 10,"$xrd"

echo -e "\n\nDeposit asset User 1 - Deposit 2"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" deposit_asset 1,"$lending_platform_badge_1" 20,"$xrd"

echo -e "\n\nDeposit asset User 2"
resim set-default-account "$account_2" "$pubkey_2"
resim call-method "$lending_pool" deposit_asset 1,"$lending_platform_badge_2" 90,"$xrd"

echo -e "\n\nWithdrawal asset User 2"
resim set-default-account "$account_2" "$pubkey_2"
resim call-method "$lending_pool" withdrawal_asset 1,"$lending_platform_badge_2" "$xrd" 27

echo -e "\n\nBorrow asset User 1 - Loan 1 - Should be successful"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" borrow_asset 1,"$lending_platform_badge_1" "$xrd" 10

echo -e "\n\nBorrow asset User 1 - Loan 2 - Should be successful"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" borrow_asset 1,"$lending_platform_badge_1" "$xrd" 15

echo -e "\n\nBorrow asset User 1 - Loan 3 - Should fail"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" borrow_asset 1,"$lending_platform_badge_1" "$xrd" 15

echo -e "\n\nRepay asset User 1 - Successful"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" repay_asset 1,"$lending_platform_badge_1" 20,"$xrd"

echo -e "\n\nBorrow asset User 1 - Loan 4 - Should be successful"
resim set-default-account "$account_1" "$pubkey_1"
resim call-method "$lending_pool" borrow_asset 1,"$lending_platform_badge_1" "$xrd" 18
