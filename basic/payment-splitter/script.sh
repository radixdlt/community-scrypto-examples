resim reset

export op1=$(resim new-account)
export pub_key1=$(echo $op1 | sed -nr 's/Public key: ([[:alnum:]_]+)/\1/p')
export address1=$(echo $op1 | sed -nr 's/Account address: ([[:alnum:]_]+)/\1/p')
export op2=$(resim new-account)
export pub_key2=$(echo $op2 | sed -nr 's/Public key: ([[:alnum:]_]+)/\1/p')
export address2=$(echo $op2 | sed -nr 's/Account address: ([[:alnum:]_]+)/\1/p')
export op3=$(resim new-account)
export pub_key3=$(echo $op3 | sed -nr 's/Public key: ([[:alnum:]_]+)/\1/p')
export address3=$(echo $op3 | sed -nr 's/Account address: ([[:alnum:]_]+)/\1/p')
export op4=$(resim new-account)
export pub_key4=$(echo $op4 | sed -nr 's/Public key: ([[:alnum:]_]+)/\1/p')
export address4=$(echo $op4 | sed -nr 's/Account address: ([[:alnum:]_]+)/\1/p')

resim set-default-account $address1 $pub_key1

resim publish . 

resim call-function 011773788de8e4d2947d6592605302d4820ad060ceab06eb2d4711 PaymentSplitter new --manifest m.out