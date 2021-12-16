# Set up environment and publish app

resim reset

export ACC=`resim new-account | grep 'address' | cut -d ":" -f2 | xargs`

resim new-token-fixed --name Tether --symbol USDT 10000

export XRD=`resim show $ACC | grep Radix | cut -d " " -f6 | tr ',' ' ' | xargs`
export USDT=`resim show $ACC | grep Tether | cut -d " " -f6 | tr ',' ' ' | xargs`

export PACKAGE="01eb23d0867f32265935d93970aded9033cc868d31795f27d8cb62"

resim publish --address $PACKAGE .

function reload {
  resim publish --address $PACKAGE .
}

# Create market

export XRD_MARKET=`resim call-function $PACKAGE Market open $XRD | grep 'Component' | cut -d ":" -f2 | xargs`

# Create orders

resim call-method $XRD_MARKET buy $USDT 3.21 1000,$XRD
resim call-method $XRD_MARKET buy $USDT 3.27 200,$XRD
resim call-method $XRD_MARKET sell 90,$USDT 3.26
resim call-method $XRD_MARKET sell 100,$USDT 3.29
resim call-method $XRD_MARKET sell 400,$USDT 3.33

export BO2=`resim show $ACC | grep 'Order Ticket #2' | cut -d " " -f6 | tr ',' ' ' | xargs`

resim call-method $XRD_MARKET print_order_book

echo
read -n 1 -s -r -p 'Press any key to fill orders ...';

resim call-method $XRD_MARKET fill_orders

echo
read -n 1 -s -r -p 'Press any key to show order book ...';

resim call-method $XRD_MARKET print_order_book

echo
read -n 1 -s -r -p 'Press any key to withdraw filled order ...';

resim call-method $XRD_MARKET withdraw_purchase 1,$BO2
