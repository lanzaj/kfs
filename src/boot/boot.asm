global start
extern k_main

section .text
bits 32
start:
    mov esp, stack_top
    call check_multiboot
    call check_cpuid
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

check_cpuid:   ; detection function taken from OSDev wiki
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ; in the FLAGS register. If we can flip it, CPUID is available.

    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret

.no_cpuid:
    ; ERR:  1, CPUID isn't supported by the processor
    mov al, "1"
    jmp error

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .bss
stack_bottom:
    resb 64
stack_top:
