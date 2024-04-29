#!/bin/bash

nasm -f elf64 src/assembly/multiboot_header.asm
nasm -f elf64 src/assembly/boot.asm
ld -n -o kfs.bin -T src/assembly/linker.ld \
    src/assembly/multiboot_header.o \
    src/assembly/boot.o

mv kfs.bin ./isofiles/boot/kernel.bin

# Create ISO image using grub-mkrescue
grub-mkrescue -o kfs.iso ./isofiles