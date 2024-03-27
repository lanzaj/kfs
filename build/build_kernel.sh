#!/bin/bash

nasm -f elf64 src/multiboot_header.asm
nasm -f elf64 src/boot.asm
ld -n -o kfs.bin -T src/linker.ld src/multiboot_header.o src/boot.o

mv kfs.bin ./isofiles/boot/kernel.bin

# Create ISO image using grub-mkrescue
grub-mkrescue -o kfs.iso ./isofiles