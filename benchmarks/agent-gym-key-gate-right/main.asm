; PhiCade Φ-Agent Gym Key Gate RIGHT v1
; SPDX-License-Identifier: MIT
;
; Stateful object dependency probe:
; 1. Player begins below a locked horizontal gate.
; 2. A visible key sits on the RIGHT.
; 3. Press A while standing on the key to acquire it.
; 4. Return to the center gate and press A to unlock it.
; 5. Move through the opened gate to the visible X target.
;
; Pressing A on the empty side does nothing. Pressing A at the gate without
; the key does nothing. Scoring remains rendered-framebuffer-only.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF MOVE_STEP EQU 2

; OAM coordinates = screen coordinate + (8,16).
DEF PLAYER_START_X EQU 80   ; screen x 72
DEF PLAYER_START_Y EQU 128  ; screen y 112
DEF KEY_LEFT_X     EQU 32   ; screen x 24
DEF KEY_RIGHT_X    EQU 128  ; screen x 120
DEF KEY_Y          EQU 128  ; screen y 112
DEF GATE_CENTER_X  EQU 80   ; screen x 72
DEF GATE_STOP_Y    EQU 96   ; screen y 80, immediately below gate row
DEF TARGET_X       EQU 80   ; screen x 72
DEF TARGET_Y       EQU 40   ; screen y 24

DEF KEY_LEFT_MAP   EQU $9800 + 14 * 32 + 3
DEF KEY_RIGHT_MAP  EQU $9800 + 14 * 32 + 15
DEF KEY_BADGE_MAP  EQU $9800 + 1 * 32 + 1
DEF GATE_ROW       EQU $9800 + 9 * 32

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

    ; Closed horizontal gate across the visible playfield.
    ld hl, GATE_ROW
    ld b, 20
.drawGate:
    ld a, 4
    ld [hli], a
    dec b
    jr nz, .drawGate

    ; RIGHT variant key.
    ld hl, KEY_RIGHT_MAP
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

    ; Visible X target above the gate.
    ld a, TARGET_Y
    ld [$FE04], a
    ld a, TARGET_X
    ld [$FE05], a
    ld a, 2
    ld [$FE06], a
    xor a
    ld [$FE07], a

    xor a
    ld [wHasKey], a
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
    ; Select buttons. A is bit 0 after inversion.
    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    bit 0, a
    jr z, .done

    ld a, [wHasKey]
    and a
    jr nz, .tryGate

    ; Key pickup requires exact player overlap.
    ld a, [wPlayerY]
    cp KEY_Y
    jr nz, .done
    ld a, [wPlayerX]
    cp KEY_RIGHT_X
    jr nz, .done

    ld a, 1
    ld [wHasKey], a

    ; Remove the world key and show a shared acquired-key badge.
    ld hl, KEY_RIGHT_MAP
    xor a
    ld [hl], a
    ld hl, KEY_BADGE_MAP
    ld a, 3
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

    ; Remove the gate row after successful unlock.
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
    ld a, [wGateOpen]
    and a
    jr nz, .upBounds
    ld a, [wPlayerY]
    cp GATE_STOP_Y
    jr z, .down
.upBounds:
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
    ; 0 blank.
    REPT 8
        db %00000000, %00000000
    ENDR

    ; 1 player, unique solid 8x8 dark patch.
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

    ; 3 sparse key icon / acquired-key badge.
    db %00111000, %00111000
    db %01000100, %01000100
    db %01000100, %01000100
    db %00111000, %00111000
    db %00010000, %00010000
    db %00011100, %00011100
    db %00010000, %00010000
    db %00011100, %00011100

    ; 4 gate tile. 32 dark pixels, so it cannot beat player locator.
    REPT 8
        db %10101010, %10101010
    ENDR
TileDataEnd:

SECTION "State", WRAM0[$C000]
wPlayerX:  ds 1
wPlayerY:  ds 1
wHasKey:   ds 1
wGateOpen: ds 1
