target remote 127.0.0.1:1234
file target/aarch64virt/release/rustpi
add-symbol-file trusted/target/aarch64/release/trusted
#add-symbol-file target/aarch64virt/release/rustpi -o -0xffffff8000000000
break *0x40080000
set confirm off
display/i $pc
set print asm-demangle on