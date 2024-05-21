global start
extern k_main

section .text
bits 32
start:
    mov esp, stack_top
    call check_multiboot
    call k_main
    hlt

check_multiboot:
    ; check the bootloader wrote its magic value in eax before loading our kernel
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret

.no_multiboot:
    ; ERR:  0, our kernel wasn't launched by a multiboot compliant bootloader (shouldn't happen with GRUB)
    mov al, "0"
    jmp error

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .bss
stack_bottom:
    resb 20000000
stack_top:
