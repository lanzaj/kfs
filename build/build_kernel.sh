#!/bin/bash

nasm -f elf32 src/boot/multiboot_header.asm
nasm -f elf32 src/boot/boot.asm
RUST_TARGET_PATH=$(pwd) xargo build --target=i386-unknown-none

ld -m elf_i386 -n -o kfs.bin -T linker.ld \
    src/boot/multiboot_header.o \
    src/boot/boot.o \
    target/i386-unknown-none/debug/libkfs.a

mv kfs.bin ./isofiles/boot/kernel.bin

# Create ISO image using grub-mkrescue
grub-mkrescue -o kfs.iso ./isofiles