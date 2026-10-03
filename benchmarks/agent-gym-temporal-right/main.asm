; PhiCade Φ-Agent Gym Temporal Cue RIGHT v1
; SPDX-License-Identifier: MIT
;
; Memory probe:
; 1. A visible RIGHT arrow is shown.
; 2. A dismisses the cue.
; 3. The ROM enters a 90-frame lockout and shows an identical two-door chamber.
; 4. After lockout, the D-pad must first be neutral before LEFT/RIGHT can move.
; 5. Correct endpoint for this ROM is the right door.
;
; The neutral-arm rule prevents a direction scheduled during the cue turn from
; being carried through the lockout. A later controller turn must choose again.
; Scoring is rendered-framebuffer-only.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF PLAYER_START_X EQU 80  ; screen x 72
DEF PLAYER_START_Y EQU 112 ; screen y 96
DEF MOVE_STEP      EQU 2
DEF WAIT_FRAMES    EQU 90

DEF STATE_CUE      EQU 0
DEF STATE_WAIT     EQU 1
DEF STATE_CHOICE   EQU 2

DEF CUE_MAP        EQU $9800 + 5 * 32 + 8
DEF LEFT_DOOR_MAP  EQU $9800 + 12 * 32 + 3
DEF RIGHT_DOOR_MAP EQU $9800 + 12 * 32 + 15

SECTION "Header", ROM0[$100]
    nop
    jp Start
    ds $150 - @, 0

SECTION "Main", ROM0[$150]
Start:
    di
    ld sp, $FFFE

    xor a
    ldh [rLCDC], a

    ld a, $E4
    ldh [rBGP], a
    ldh [rOBP0], a

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

    xor a
    ld hl, $9800
    ld bc, 32 * 32
.clearBg:
    ld [hli], a
    dec bc
    ld a, b
    or c
    jr nz, .clearBg

    ; Large RIGHT cue: two shaft tiles + head.
    ld hl, CUE_MAP
    ld a, 5
    ld [hli], a
    ld [hli], a
    ld a, 4
    ld [hl], a

    xor a
    ld hl, $FE00
    ld b, 160
.clearOam:
    ld [hli], a
    dec b
    jr nz, .clearOam

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

    xor a
    ld [wState], a
    ld [wArmed], a
    ld [wTimer], a

    ld a, %10010011
    ldh [rLCDC], a

MainLoop:
.waitVBlank:
    ldh a, [rLY]
    cp 144
    jr c, .waitVBlank

    call ReadInput
    call UpdateWait
    call WritePlayerOam

.waitVisible:
    ldh a, [rLY]
    cp 144
    jr nc, .waitVisible
    jr MainLoop

ReadInput:
    ld a, [wState]
    cp STATE_CUE
    jr z, ReadCue
    cp STATE_CHOICE
    jr z, ReadChoice
    ret

ReadCue:
    ; Select buttons. A is bit 0 when active-low input is inverted.
    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    bit 0, a
    jr z, .done
    call EnterWait
.done:
    ld a, $30
    ldh [rP1], a
    ret

EnterWait:
    ld a, STATE_WAIT
    ld [wState], a
    ld a, WAIT_FRAMES
    ld [wTimer], a
    xor a
    ld [wArmed], a

    ; Remove the cue and reveal the same two-door chamber used by the LEFT ROM.
    ld hl, CUE_MAP
    xor a
    ld [hli], a
    ld [hli], a
    ld [hl], a

    ld hl, LEFT_DOOR_MAP
    ld a, 6
    ld [hl], a
    ld hl, RIGHT_DOOR_MAP
    ld [hl], a
    ret

UpdateWait:
    ld a, [wState]
    cp STATE_WAIT
    ret nz
    ld a, [wTimer]
    and a
    jr z, .unlock
    dec a
    ld [wTimer], a
    ret nz
.unlock:
    ld a, STATE_CHOICE
    ld [wState], a
    xor a
    ld [wArmed], a
    ret

ReadChoice:
    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    ld b, a

    ; A neutral D-pad frame is mandatory after lockout.
    and a
    jr nz, .nonNeutral
    ld a, 1
    ld [wArmed], a
    jr .done

.nonNeutral:
    ld a, [wArmed]
    and a
    jr z, .done

    bit 1, b
    jr z, .right
    ld a, [wPlayerX]
    cp 10
    jr c, .right
    sub MOVE_STEP
    ld [wPlayerX], a

.right:
    bit 0, b
    jr z, .done
    ld a, [wPlayerX]
    cp 160
    jr nc, .done
    add MOVE_STEP
    ld [wPlayerX], a

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
    ; 0 blank.
    REPT 8
        db %00000000, %00000000
    ENDR

    ; 1 solid player. Unique 64-dark-pixel locator.
    REPT 8
        db %11111111, %11111111
    ENDR

    ; 2 reserved blank-ish marker.
    REPT 8
        db %00000000, %00000000
    ENDR

    ; 3 LEFT arrow head.
    db %00010000, %00010000
    db %00110000, %00110000
    db %01110000, %01110000
    db %11111111, %11111111
    db %11111111, %11111111
    db %01110000, %01110000
    db %00110000, %00110000
    db %00010000, %00010000

    ; 4 RIGHT arrow head.
    db %00001000, %00001000
    db %00001100, %00001100
    db %00001110, %00001110
    db %11111111, %11111111
    db %11111111, %11111111
    db %00001110, %00001110
    db %00001100, %00001100
    db %00001000, %00001000

    ; 5 arrow shaft. Sparse enough not to compete with player locator.
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000
    db %11111111, %11111111
    db %11111111, %11111111
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000

    ; 6 identical door outline used on both sides of both ROMs.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10100101, %10100101
TileDataEnd:

SECTION "State", WRAM0[$C000]
wPlayerX: ds 1
wPlayerY: ds 1
wState:   ds 1
wTimer:   ds 1
wArmed:   ds 1
