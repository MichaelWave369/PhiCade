; PhiCade Φ-Agent Gym Branch Selector TRIANGLE v1
; SPDX-License-Identifier: MIT
;
; Conditional branch probe:
; 1. Both modules are visible in both variants: TRIANGLE left, SQUARE right.
; 2. A central selector cue indicates which module is valid for this run.
; 3. Press A on the matching module to acquire it.
; 4. Pressing A on the wrong module enters an irreversible FAIL state.
; 5. Return to the shared generator, install the selected module, power the system.
; 6. Open the powered gate and reach the visible X target.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF MOVE_STEP EQU 2

; OAM coordinates = screen coordinate + (8,16).
DEF PLAYER_START_X EQU 80
DEF PLAYER_START_Y EQU 128
DEF MODULE_LEFT_X  EQU 32
DEF MODULE_RIGHT_X EQU 128
DEF MODULE_Y       EQU 128
DEF GENERATOR_X    EQU 80
DEF GENERATOR_Y    EQU 128
DEF GATE_CENTER_X  EQU 80
DEF GATE_STOP_Y    EQU 96
DEF TARGET_X       EQU 80
DEF TARGET_Y       EQU 40

DEF MODULE_LEFT_MAP  EQU $9800 + 14 * 32 + 3
DEF MODULE_RIGHT_MAP EQU $9800 + 14 * 32 + 15
DEF SELECTOR_MAP     EQU $9800 + 4 * 32 + 9
DEF MODULE_BADGE_MAP EQU $9800 + 1 * 32 + 1
DEF GENERATOR_MAP    EQU $9800 + 12 * 32 + 9
DEF GATE_ROW         EQU $9800 + 9 * 32

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

    ; Closed horizontal gate.
    ld hl, GATE_ROW
    ld b, 20
.drawGate:
    ld a, 4
    ld [hli], a
    dec b
    jr nz, .drawGate

    ; Shared unpowered generator.
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a

    ; Both branch modules exist in both variants.
    ld hl, MODULE_LEFT_MAP
    ld a, 3
    ld [hl], a
    ld hl, MODULE_RIGHT_MAP
    ld a, 7
    ld [hl], a

    ; TRIANGLE selector cue for this variant.
    ld hl, SELECTOR_MAP
    ld a, 3
    ld [hl], a

    xor a
    ld hl, $FE00
    ld b, 160
.clearOam:
    ld [hli], a
    dec b
    jr nz, .clearOam

    ; Player.
    ld a, PLAYER_START_Y
    ld [wPlayerY], a
    ld [$FE00], a
    ld a, PLAYER_START_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, 1
    ld [$FE02], a
    xor a
    ld [$FE03], a

    ; Visible X target.
    ld a, TARGET_Y
    ld [$FE04], a
    ld a, TARGET_X
    ld [$FE05], a
    ld a, 2
    ld [$FE06], a
    xor a
    ld [$FE07], a

    xor a
    ld [wHasModule], a
    ld [wFailed], a
    ld [wPowerOn], a
    ld [wGateOpen], a

    ld a, %10010011
    ldh [rLCDC], a

MainLoop:
.waitVBlank:
    ldh a, [rLY]
    cp 144
    jr c, .waitVBlank

    call ReadAction
    call ReadMove
    call WritePlayerOam

.waitVisible:
    ldh a, [rLY]
    cp 144
    jr nc, .waitVisible
    jr MainLoop

ReadAction:
    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    bit 0, a
    jr z, .done

    ld a, [wFailed]
    and a
    jr nz, .done

    ld a, [wPowerOn]
    and a
    jr nz, .tryGate

    ld a, [wHasModule]
    and a
    jr nz, .tryGenerator

    ld a, [wPlayerY]
    cp MODULE_Y
    jr nz, .done

    ld a, [wPlayerX]
    cp MODULE_LEFT_X
    jr z, .acceptModule
    cp MODULE_RIGHT_X
    jr z, .failBranch
    jr .done

.acceptModule:
    ld a, 1
    ld [wHasModule], a

    ; Once the selector is satisfied, erase variant-specific evidence and
    ; converge both worlds on a shared carried-module state.
    xor a
    ld hl, MODULE_LEFT_MAP
    ld [hl], a
    ld hl, MODULE_RIGHT_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld [hl], a
    ld hl, MODULE_BADGE_MAP
    ld a, 8
    ld [hl], a
    jr .done

.failBranch:
    ld a, 1
    ld [wFailed], a

    ; Wrong branch is irreversible for this run.
    xor a
    ld hl, MODULE_LEFT_MAP
    ld [hl], a
    ld hl, MODULE_RIGHT_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, 9
    ld [hl], a
    jr .done

.tryGenerator:
    ld a, [wPlayerX]
    cp GENERATOR_X
    jr nz, .done
    ld a, [wPlayerY]
    cp GENERATOR_Y
    jr nz, .done

    xor a
    ld [wHasModule], a
    ld hl, MODULE_BADGE_MAP
    ld [hl], a
    ld a, 1
    ld [wPowerOn], a
    ld hl, GENERATOR_MAP
    ld a, 6
    ld [hl], a
    jr .done

.tryGate:
    ld a, [wGateOpen]
    and a
    jr nz, .done
    ld a, [wPlayerX]
    cp GATE_CENTER_X
    jr nz, .done
    ld a, [wPlayerY]
    cp GATE_STOP_Y
    jr nz, .done

    ld a, 1
    ld [wGateOpen], a
    ld hl, GATE_ROW
    ld b, 20
.clearGate:
    xor a
    ld [hli], a
    dec b
    jr nz, .clearGate

.done:
    ld a, $30
    ldh [rP1], a
    ret

ReadMove:
    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    ld b, a

    bit 0, b
    jr z, .left

    ; Carried-module and failed states both clamp back to shared center.
    ld a, [wHasModule]
    and a
    jr nz, .rightClampCenter
    ld a, [wFailed]
    and a
    jr nz, .rightClampCenter
    ld a, [wPowerOn]
    and a
    jr nz, .rightBounds
    ld a, [wPlayerY]
    cp MODULE_Y
    jr nz, .rightBounds
    ld a, [wPlayerX]
    cp MODULE_RIGHT_X
    jr z, .left
    jr .rightBounds
.rightClampCenter:
    ld a, [wPlayerY]
    cp MODULE_Y
    jr nz, .rightBounds
    ld a, [wPlayerX]
    cp GENERATOR_X
    jr z, .left
.rightBounds:
    ld a, [wPlayerX]
    cp 160
    jr nc, .left
    add MOVE_STEP
    ld [wPlayerX], a

.left:
    bit 1, b
    jr z, .up

    ld a, [wHasModule]
    and a
    jr nz, .leftClampCenter
    ld a, [wFailed]
    and a
    jr nz, .leftClampCenter
    ld a, [wPowerOn]
    and a
    jr nz, .leftBounds
    ld a, [wPlayerY]
    cp MODULE_Y
    jr nz, .leftBounds
    ld a, [wPlayerX]
    cp MODULE_LEFT_X
    jr z, .up
    jr .leftBounds
.leftClampCenter:
    ld a, [wPlayerY]
    cp MODULE_Y
    jr nz, .leftBounds
    ld a, [wPlayerX]
    cp GENERATOR_X
    jr z, .up
.leftBounds:
    ld a, [wPlayerX]
    cp 10
    jr c, .up
    sub MOVE_STEP
    ld [wPlayerX], a

.up:
    bit 2, b
    jr z, .down
    ld a, [wGateOpen]
    and a
    jr nz, .upBounds
    ld a, [wPlayerY]
    cp GATE_STOP_Y
    jr z, .down
.upBounds:
    ld a, [wPlayerY]
    cp TARGET_Y
    jr z, .down
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
    ; 0 blank.
    REPT 8
        db %00000000, %00000000
    ENDR

    ; 1 player, unique solid 8x8 patch.
    REPT 8
        db %11111111, %11111111
    ENDR

    ; 2 X target.
    db %10000001, %10000001
    db %01000010, %01000010
    db %00100100, %00100100
    db %00011000, %00011000
    db %00011000, %00011000
    db %00100100, %00100100
    db %01000010, %01000010
    db %10000001, %10000001

    ; 3 TRIANGLE module / selector.
    db %00010000, %00010000
    db %00111000, %00111000
    db %00111000, %00111000
    db %01111100, %01111100
    db %01111100, %01111100
    db %11111110, %11111110
    db %11111110, %11111110
    db %00000000, %00000000

    ; 4 gate tile, 32 dark pixels.
    REPT 8
        db %10101010, %10101010
    ENDR

    ; 5 generator OFF.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10111101, %10111101
    db %10100101, %10100101
    db %10100101, %10100101
    db %10111101, %10111101
    db %10000001, %10000001
    db %11111111, %11111111

    ; 6 generator ON.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10100101, %10100101
    db %10011001, %10011001
    db %10011001, %10011001
    db %10100101, %10100101
    db %10000001, %10000001
    db %11111111, %11111111

    ; 7 SQUARE module / selector, hollow to avoid player-locator ambiguity.
    db %01111110, %01111110
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01111110, %01111110
    db %00000000, %00000000

    ; 8 shared carried-module badge.
    db %00111100, %00111100
    db %00100100, %00100100
    db %01111110, %01111110
    db %01011010, %01011010
    db %01011010, %01011010
    db %01111110, %01111110
    db %00100100, %00100100
    db %00111100, %00111100

    ; 9 irreversible FAIL marker.
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
wPlayerX:   ds 1
wPlayerY:   ds 1
wHasModule: ds 1
wFailed:    ds 1
wPowerOn:   ds 1
wGateOpen:  ds 1
