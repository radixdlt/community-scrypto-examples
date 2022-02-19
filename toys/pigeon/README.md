# Pigeon Example

## How to test
1. `resim reset`
1. `resim new-account` -> save address in $acc1
1. `resim publish .` -> save package address in $package
1. `resim call-function $package Pigeon new` 
    save the component's address in $component,
    the first resource def in $head
    the second resource def in $body
    the third resource def in $wing
    the fourth resource def in $leg

1. Try to fly: `resim call-method $component fly`. You should get an error. We need to first assemble the pigeon !
1. Add the head: `resim call-method $component add_part 1,$head`
1. Add the body: `resim call-method $component add_part 1,$body`
1. Add the wings: `resim call-method $component add_part 2,$wing`
1. Add the legs: `resim call-method $component add_part 2,$leg`
1. Now it should fly: `resim call-method $component fly`