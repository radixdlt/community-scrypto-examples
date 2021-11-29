# Set up environment and publish app

resim reset

$ACC = resim new-account | Select-String 'Account address: (\w+)' | %{ $_.Matches.Groups[1].Value }

resim new-token-fixed --name Tether --symbol USDT 10000

$XRD = resim show $ACC | Select-String 'resource_def: (\w+), name: "Radix"' | %{ $_.Matches.Groups[1].Value }
$USDT = resim show $ACC | Select-String 'resource_def: (\w+), name: "Tether"' | %{ $_.Matches.Groups[1].Value }

$PACKAGE = "01eb23d0867f32265935d93970aded9033cc868d31795f27d8cb62"

resim publish --address $PACKAGE .

function Reload {
  resim publish --address $PACKAGE .
}

# Create market

$XRD_MARKET = resim call-function $PACKAGE Market open $XRD | Select-String 'Component: (\w+)' | %{$_.Matches.Groups[1].Value }

# Create orders

resim call-method $XRD_MARKET buy $USDT 3.21 1000,$XRD
resim call-method $XRD_MARKET buy $USDT 3.27 200,$XRD
resim call-method $XRD_MARKET sell 90,$USDT 3.26
resim call-method $XRD_MARKET sell 100,$USDT 3.29
resim call-method $XRD_MARKET sell 400,$USDT 3.33

$BO2 = resim show $ACC | Select-String 'resource_def: (\w+), name: "Order Ticket #2"' | %{ $_.Matches.Groups[1].Value }

resim call-method $XRD_MARKET print_order_book

Write-Output ''
Write-Host -NoNewLine 'Press any key to fill orders ...';
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');

resim call-method $XRD_MARKET fill_orders

Write-Output ''
Write-Host -NoNewLine 'Press any key to show order book ...';
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');

resim call-method $XRD_MARKET print_order_book

Write-Output ''
Write-Host -NoNewLine 'Press any key to withdraw filled order ...';
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');

resim call-method $XRD_MARKET withdraw_purchase 1,$BO2
