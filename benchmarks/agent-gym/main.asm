; PhiCade Φ-Agent Gym v1
; SPDX-License-Identifier: MIT
;
; A deliberately tiny Game Boy benchmark ROM.
; Task: move the solid 8x8 player block to the visible X target.
; Input: D-pad only. Movement: 2 pixels per emulated frame.
;
; The benchmark harness scores from rendered pixels, not emulator RAM.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF PLAYER_START_X EQU 24  ; OAM x = screen x + 8  => screen x 16
DEF PLAYER_START_Y EQU 40  ; OAM y = screen y + 16 => screen y 24
DEF TARGET_X       EQU 144 ; screen x 136
DEF TARGET_Y       EQU 128 ; screen y 112
DEF MOVE_STEP      EQU 2

SECTION "Header", ROM0[$100]
    nop
    jp Start
    ds $150 - @, 0

SECTION "Main", ROM0[$150]
Start:
    di
    ld sp, $FFFE

    ; Disable LCD before touching VRAM/OAM.
    xor a
    ldh [rLCDC], a

    ; DMG palettes: color 0 white -> color 3 black.
    ld a, $E4
    ldh [rBGP], a
    ldh [rOBP0], a

    ; Copy three tiles to VRAM:
    ; tile 0 = blank, tile 1 = solid player, tile 2 = X target.
    ld de, TileData
    ld hl, $8000
    ld bc, TileDataEnd - TileData
.copyTiles:
    ld a, [de]
    ld [hli], a
    inc de
    dec bc
    ld a, b
    or c
    jr nz, .copyTiles

    ; Clear the 32x32 background map to blank tile 0.
    xor a
    ld hl, $9800
    ld bc, 32 * 32
.clearBg:
    ld [hli], a
    dec bc
    ld a, b
    or c
    jr nz, .clearBg

    ; Clear all OAM.
    xor a
    ld hl, $FE00
    ld b, 160
.clearOam:
    ld [hli], a
    dec b
    jr nz, .clearOam

    ; Player sprite: solid block, tile 1.
    ld a, PLAYER_START_Y
    ld [$FE00], a
    ld [wPlayerY], a
    ld a, PLAYER_START_X
    ld [$FE01], a
    ld [wPlayerX], a
    ld a, 1
    ld [$FE02], a
    xor a
    ld [$FE03], a

    ; Target sprite: X glyph, tile 2.
    ld a, TARGET_Y
    ld [$FE04], a
    ld a, TARGET_X
    ld [$FE05], a
    ld a, 2
    ld [$FE06], a
    xor a
    ld [$FE07], a

    ; LCD on, OBJ on, BG on, unsigned tile data at $8000.
    ld a, %10010011
    ldh [rLCDC], a

MainLoop:
.waitVBlank:
    ldh a, [rLY]
    cp 144
    jr c, .waitVBlank

    call ReadAndMove
    call WritePlayerOam

.waitVisible:
    ldh a, [rLY]
    cp 144
    jr nc, .waitVisible
    jr MainLoop

ReadAndMove:
    ; Select D-pad (P14 low), then read active-low input.
    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    ld b, a

    ; Right, bit 0.
    bit 0, b
    jr z, .left
    ld a, [wPlayerX]
    cp 160
    jr nc, .left
    add MOVE_STEP
    ld [wPlayerX], a

.left:
    bit 1, b
    jr z, .up
    ld a, [wPlayerX]
    cp 10
    jr c, .up
    sub MOVE_STEP
    ld [wPlayerX], a

.up:
    bit 2, b
    jr z, .down
    ld a, [wPlayerY]
    cp 18
    jr c, .down
    sub MOVE_STEP
    ld [wPlayerY], a

.down:
    bit 3, b
    jr z, .done
    ld a, [wPlayerY]
    cp 144
    jr nc, .done
    add MOVE_STEP
    ld [wPlayerY], a

.done:
    ld a, $30
    ldh [rP1], a
    ret

WritePlayerOam:
    ld a, [wPlayerY]
    ld [$FE00], a
    ld a, [wPlayerX]
    ld [$FE01], a
    ret

SECTION "Tiles", ROM0
TileData:
    ; Tile 0: blank.
    REPT 8
        db %00000000, %00000000
    ENDR

    ; Tile 1: solid black 8x8 block.
    REPT 8
        db %11111111, %11111111
    ENDR

    ; Tile 2: black X on transparent/white background.
    db %10000001, %10000001
    db %01000010, %01000010
    db %00100100, %00100100
    db %00011000, %00011000
    db %00011000, %00011000
    db %00100100, %00100100
    db %01000010, %01000010
    db %10000001, %10000001
TileDataEnd:

SECTION "State", WRAM0[$C000]
wPlayerX: ds 1
wPlayerY: ds 1
