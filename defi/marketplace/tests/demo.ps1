# Set up environment and publish app

resim reset

$BUYER_OUT = resim new-account
$BUYER_ACC = $BUYER_OUT | Select-String 'Account address: (\w+)' | %{ $_.Matches.Groups[1].Value }
$BUYER_PUB = $BUYER_OUT | Select-String 'Public key: (\w+)' | %{ $_.Matches.Groups[1].Value }

$SELLER_OUT = resim new-account
$SELLER_ACC = $SELLER_OUT | Select-String 'Account address: (\w+)' | %{ $_.Matches.Groups[1].Value }
$SELLER_PUB = $SELLER_OUT | Select-String 'Public key: (\w+)' | %{ $_.Matches.Groups[1].Value }

$XRD = resim show $BUYER_ACC | Select-String 'resource_def: (\w+), name: "Radix"' | %{ $_.Matches.Groups[1].Value }
$USDT = resim new-token-fixed --name Tether --symbol USDT 1000 | Select-String 'ResourceDef: (\w+)' | %{ $_.Matches.Groups[1].Value }

resim transfer 1000,$USDT $SELLER_ACC

$PACKAGE = "01eb23d0867f32265935d93970aded9033cc868d31795f27d8cb62"

resim publish --address $PACKAGE .

function Reload {
  resim publish --address $PACKAGE .
}

# Create market

$XRD_MARKET = resim call-function $PACKAGE Market open $XRD | Select-String 'Component: (\w+)' | %{$_.Matches.Groups[1].Value }

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

$BO2 = resim show $BUYER_ACC | Select-String 'resource_def: (\w+), name: "Order Ticket #2"' | %{ $_.Matches.Groups[1].Value }
$SO3 = resim show $SELLER_ACC | Select-String 'resource_def: (\w+), name: "Order Ticket #3"' | %{ $_.Matches.Groups[1].Value }
$BO6 = resim show $BUYER_ACC | Select-String 'resource_def: (\w+), name: "Order Ticket #6"' | %{ $_.Matches.Groups[1].Value }

resim call-method $XRD_MARKET print_order_book

Write-Output ''
Write-Host -NoNewLine 'Press any key to withdraw filled order ...';
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');

resim set-default-account $BUYER_ACC $BUYER_PUB

resim call-method $XRD_MARKET withdraw_purchase 1,$BO2

resim set-default-account $SELLER_ACC $SELLER_PUB

resim call-method $XRD_MARKET withdraw_sale 1,$SO3

$BOUGHT_USDT = resim show $BUYER_ACC | Select-String 'amount: (\d+)\..*, name: "Tether"' | %{ $_.Matches.Groups[1].Value }

Write-Output ""
Write-Output "Buyer bought around $BOUGHT_USDT USDT, via limit order"
Write-Output ""

if ($BOUGHT_USDT -eq "61") {
    resim set-default-account $BUYER_ACC $BUYER_PUB

    resim call-method $XRD_MARKET withdraw_purchase 1,$BO6

    $BOUGHT_USDT = resim show $BUYER_ACC | Select-String 'amount: (\d+)\..*, name: "Tether"' | %{ $_.Matches.Groups[1].Value }

    Write-Output ""
    Write-Output "Buyer bought another 122 USDT ($BOUGHT_USDT altogether) roughly, via market order"

    if ($BOUGHT_USDT -eq "183") {
        $MARKET_PRICE = resim call-method $XRD_MARKET print_market_prices | Select-String 'USDT \|\s*(\d\.\d+).*' | %{ $_.Matches.Groups[1].Value }

        if ($MARKET_PRICE -eq "3.29") {
            Write-Output ""
            Write-Output "Market price is 3.29 as last sell order filled was SO-4"
            Write-Output ""
            Write-Output "Success! :)"
        } else {
          Write-Output "Failure... (wrong market price) :("
        }
    } else {
      Write-Output "Failure... (wrong market buy amount) :("
    }
} else {
    Write-Output "Failure... (wrong limit buy amount) :("
}

Write-Output ""
