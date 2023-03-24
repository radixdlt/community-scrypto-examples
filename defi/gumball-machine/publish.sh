

reset_resim_and_publish (){
resim reset

resim new-account
resim new-account
resim publish .

}
# reset_resim_and_publish

export account_1=account_sim1qd8ucx047pcfdatss546ff0nmypzjv4dc9l8scg3cyqs2tz3yz 
export account_2=account_sim1qwyy2z6ucdl3rjnkk4et59yxtgk5tmqtplytuhnnzd4sgx9uvx
export package_address=package_sim1qyrq7hjmzy4mrdktel48xcpag273sswz4v3feyz0xxkspravk7
export xrd_resource_address=resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz8qety



instantiate_gumball_blueprint (){
    resim call-function $package_address GumballMachine instantiate_gumball_machine 0.2
}
# instantiate_gumball_blueprint

export component=component_sim1qfvj8huyg2j2w82zcprgss4ry5sk75szjav50haztfkqkjsrks
export component_resource=resource_sim1qpvj8huyg2j2w82zcprgss4ry5sk75szjav50haztfkqnvgcfc
export admin_badge=resource_sim1qpvj8huyg2j2w82zcprgss4ry5sk75szjav50haztfkqnvgcfc

test_gumball_buy_method(){
    resim call-method $component check_price
    resim call-method $component buy_gumball  3.0,$xrd_resource_address
}

#test_gumball_buy_method

test_admin_access(){
    resim call-method $component check_gumballs --proofs "1,$admin_badge"
    resim call-method $component mint_gumballs 100.0 -p "1,$admin_badge"
    resim call-method $component set_price 0.3 -p "1,$admin_badge"
    resim call-method $component withdraw 3,$xrd_resource_address -p "1,$admin_badge"

}

test_admin_access