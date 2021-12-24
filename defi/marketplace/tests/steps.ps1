# Set up environment and publish app

resim reset

$BUYER_OUT = resim new-account
$BUYER_ACC = Write-Output "$BUYER_OUT" | Get-Account-Address
$BUYER_PUB = Write-Output "$BUYER_OUT" | Get-Public-Key

$SELLER_OUT = resim new-account
$SELLER_ACC = Write-Output "$SELLER_OUT" | Get-Account-Address
$SELLER_PUB = Write-Output "$SELLER_OUT" | Get-Public-Key

$XRD = resim show $BUYER_ACC | Get-Resource-Def "Radix"
$USDT = resim new-token-fixed --name Tether --symbol USDT 1000 | Get-New-Def

resim transfer 1000,$USDT $SELLER_ACC

$PACKAGE = Write-Output "01eb23d0867f32265935d93970aded9033cc868d31795f27d8cb62"

resim publish --address $PACKAGE .

# Create market

$XRD_MARKET = resim call-function $PACKAGE Market open $XRD | Get-Component

# Create orders

resim set-default-account $BUYER_ACC $BUYER_PUB

resim call-method $XRD_MARKET limit_buy $USDT 3.21 1000,$XRD # BO-1, not filled
resim call-method $XRD_MARKET limit_buy $USDT 3.27 200,$XRD # BO-2, not filled
#
resim set-default-account $SELLER_ACC $SELLER_PUB

resim call-method $XRD_MARKET limit_sell 90,$USDT 3.26 # SO-3, partly filled (fills BO-2 fully)
resim call-method $XRD_MARKET limit_sell 100,$USDT 3.29 # SO-4, not filled
resim call-method $XRD_MARKET limit_sell 400,$USDT 3.33 # SO-5, not filled
#
resim set-default-account $BUYER_ACC $BUYER_PUB

resim call-method $XRD_MARKET market_buy $USDT 400,$XRD # BO-6, fills rest of SO-3 and most part of SO-4, SO-5 remains unfilled

# lookup address for order ticket NFT
$ORDER_TICKET = resim show $BUYER_ACC | Get-Resource-Def "Order Ticket"

resim call-method $XRD_MARKET print_order_book

Wait-For-User 'Press any key to withdraw filled order ...'

resim set-default-account $BUYER_ACC $BUYER_PUB

# "#2,$ADDR" takes the NFT with the ID 2 of the NFT with the given address from the current account's vault
resim call-method $XRD_MARKET withdraw_purchase "#2,$ORDER_TICKET"

resim set-default-account $SELLER_ACC $SELLER_PUB

resim call-method $XRD_MARKET withdraw_sale "#3,$ORDER_TICKET"

$BOUGHT_USDT = resim show $BUYER_ACC | Get-Resource-Amount "Tether"

Write-Output ""
Write-Output "Buyer bought around $BOUGHT_USDT USDT, via limit order"
Write-Output ""

Exit-Unless-Equal $BOUGHT_USDT "62" "Failure... (wrong limit buy amount) :("

resim set-default-account $BUYER_ACC $BUYER_PUB

resim call-method $XRD_MARKET withdraw_purchase "#6,$ORDER_TICKET"

$BOUGHT_USDT = resim show $BUYER_ACC | Get-Resource-Amount "Tether"

Write-Output ""
Write-Output "Buyer bought another 122 USDT ($BOUGHT_USDT altogether) roughly, via market order"

Exit-Unless-Equal $BOUGHT_USDT "183" "Failure... (wrong market buy amount) :("

$MARKET_PRICE = resim call-method $XRD_MARKET print_market_prices | Get-Market-Price "USDT"

Exit-Unless-Equal $MARKET_PRICE "3.29" "Failure... (wrong market price) :("

Write-Output ""
Write-Output "Market price is 3.29 as last sell order filled was SO-4"
Write-Output ""
Write-Output "Success! :)"

Write-Output ""
