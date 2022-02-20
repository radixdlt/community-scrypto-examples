#!/bin/bash
export package=01ff3eae9463d913a0dba37b78896414eadf59ce144a9143c8018f
export acc=0293c502780e23621475989d707cd8128e4506362e5fed6ac0c00a
export pub=005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
export component=0236ca00316c8eb5ad51b0cb5e3f232cb871803a85ec3847b36bb4
export token=03d527faee6d0b91e7c1bab500c6a986e5777a25d704acc288d542
resim reset
resim new-account
resim publish .
resim call-function $package SlotMachine new 10000
resim call-method $component free_token
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim show $acc
resim show $component