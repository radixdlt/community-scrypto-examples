#!/bin/bash
export package=013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f
export acc=02c1897261516ff0597fded2b19bf2472ff97b2d791ea50bd02ab2
export pub=04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
export component=02b8a2383a7d462575e673153ae12e5ed78ee5142abe2a8abcab58
export token=03eb23d0867f32265935d93970aded9033cc868d31795f27d8cb62
resim reset
resim new-account
resim publish .
resim call-function $package SlotMachine new 10000
resim call-method $component free_token
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim call-method $component play "1,$token"
resim show $acc
resim show $component