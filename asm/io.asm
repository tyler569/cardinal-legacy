; vim: syntax=nasm :

section .text

; outN(port, value)
global outb
outb:
    mov eax, esi
    mov edx, edi
    out dx, al
    ret

global outw
outw:
    mov eax, esi
    mov edx, edi
    out dx, ax
    ret

global outl
outl:
    mov eax, esi
    mov edx, edi
    out dx, eax
    ret


; inN(port)
global inb
inb:
    mov edx, edi
    in al, dx
    ret

global inw
inw:
    mov edx, edi
    in ax, dx
    ret

global inl
inl:
    mov edx, edi
    in eax, dx
    ret


global asm_enable_irqs
asm_enable_irqs:
    sti
    ret

global asm_disable_irqs
asm_disable_irqs:
    cli
    ret

global asm_read_cr2
asm_read_cr2:
    mov rax, cr2
    ret

global asm_pause
asm_pause:
    hlt

global asm_kernel_start
extern _kernel_phy_start
asm_kernel_start:
    mov rax, _kernel_phy_start
    ret

global asm_kernel_end
extern _kernel_phy_end
asm_kernel_end:
    mov rax, _kernel_phy_end
    ret
