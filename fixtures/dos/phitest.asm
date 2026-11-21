; PHITEST.COM: original PhiCade DOS fixture; source assembled by NASM in CI.
; BIOS VGA mode 13h, magenta framebuffer, PIT speaker, Esc-to-exit polling.
; Keyboard/mouse guest delivery and DOS-side exit remain unqualified at this rung.
bits 16
org 0x100

start:
    mov ax, 0x0013
    int 0x10
    mov ax, 0xa000
    mov es, ax
    xor di, di
    cld
    mov cx, 320*200
    mov al, 0x0d
    rep stosb

    mov al, 0xb6
    out 0x43, al
    mov ax, 2712
    out 0x42, al
    mov al, ah
    out 0x42, al
    in al, 0x61
    or al, 0x03
    out 0x61, al

.keyboard_loop:
    mov ah, 0x01
    int 0x16
    jz .keyboard_loop
    mov ah, 0x00
    int 0x16
    cmp al, 27
    jne .keyboard_loop

    in al, 0x61
    and al, 0xfc
    out 0x61, al
    mov ax, 0x0003
    int 0x10
    mov ax, 0x4c00
    int 0x21
