global start
extern long_mode_start

section .text
bits 32
start:
    ; update stack pointer so we're ready to call functions
    mov esp, stack_top
    call check_multiboot
    call check_cpuid
    call check_long_mode
    call set_up_page_tables
    call enable_paging
    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

enable_paging:
    ; load p4 address to cr3 register (cpu uses this register to access p4 table)
    mov eax, p4_table
    mov cr3, eax
    ; enable Physical Address Extension flag in cr4
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    ; set the long mode bit in the EFER Model Specific Register
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    ret
set_up_page_tables:
    mov eax, p3_table ; map first p4 entry to p3 table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    mov eax, p2_table ; same for p3 and p2
    or eax, 0b11
    mov [p3_table], eax
    mov ecx, 0  ; index

.map_p2_table: ; map each P2 entry to a 2MiB page (loop)
    mov eax, 0x200000 ; 2MiB
    mul ecx ; index * 2MiB to get the start address of each page
    or eax, 0b10000011 ; present + writable + huge
    mov [p2_table + ecx * 8], eax ;
    inc ecx ; index++
    cmp ecx, 512 ; P2 has 512 entry
    jne .map_p2_table ; loop back
    ret
check_multiboot:
    ; check the bootloader wrote its magic value in eax before loading our kernel
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret

.no_multiboot:
    ; ERR:  0, our kernel wasn't launched by a multiboot compliant bootloader (should'nt happen with GRUB)
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
    ; ERR:  2, CPUID isn't supported by the processor
    mov al, "1"
    jmp error

check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret

.no_long_mode:
    mov al, "2"
    jmp error

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 64
stack_top:

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64