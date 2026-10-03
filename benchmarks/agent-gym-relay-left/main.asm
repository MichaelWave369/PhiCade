; PhiCade Φ-Agent Gym Relay Rooms LEFT v1
; SPDX-License-Identifier: MIT
;
; Three-room objective:
; 1. Briefing room shows a LEFT cue.
; 2. A accepts the mission and removes the cue.
; 3. Corridor room requires a visible DOWN -> RIGHT -> UP wall detour.
; 4. Terminal room is visually identical to the RIGHT variant.
; 5. A later LEFT/RIGHT choice must use the earlier briefing evidence.
;
; The corridor cannot satisfy the final score because its player Y range stays
; at least 16 pixels away from the terminal target Y.
; Scoring is rendered-framebuffer-only.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF MOVE_STEP EQU 2

DEF STATE_BRIEFING EQU 0
DEF STATE_CORRIDOR EQU 1
DEF STATE_TERMINAL EQU 2

; OAM coordinates = screen coordinate + (8,16).
DEF BRIEF_X EQU 80
DEF BRIEF_Y EQU 40
DEF CORRIDOR_START_X EQU 24
DEF CORRIDOR_START_Y EQU 40
DEF TERMINAL_START_X EQU 80
DEF TERMINAL_START_Y EQU 128

; Corridor wall: screen x 72..79, y 0..95. Gap starts at screen y 96.
DEF WALL_LEFT_STOP_X  EQU 72
DEF WALL_RIGHT_STOP_X EQU 88
DEF WALL_GAP_Y        EQU 112
DEF CORRIDOR_EXIT_X   EQU 144
DEF CORRIDOR_EXIT_Y_MAX EQU 40

DEF CUE_MAP        EQU $9800 + 3 * 32 + 8
DEF EXIT_MAP       EQU $9800 + 3 * 32 + 17
DEF LEFT_DOOR_MAP  EQU $9800 + 14 * 32 + 3
DEF RIGHT_DOOR_MAP EQU $9800 + 14 * 32 + 15

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

    ; Briefing LEFT arrow.
    ld hl, CUE_MAP
    ld a, 3
    ld [hli], a
    ld a, 5
    ld [hli], a
    ld [hl], a

    xor a
    ld hl, $FE00
    ld b, 160
.clearOam:
    ld [hli], a
    dec b
    jr nz, .clearOam

    ld a, BRIEF_Y
    ld [wPlayerY], a
    ld a, BRIEF_X
    ld [wPlayerX], a
    xor a
    ld [wState], a
    ld [wArmed], a

    ld a, 1
    ld [$FE02], a
    xor a
    ld [$FE03], a

    ld a, %10010011
    ldh [rLCDC], a

MainLoop:
.waitVBlank:
    ldh a, [rLY]
    cp 144
    jr c, .waitVBlank

    call ReadInput
    call WritePlayerOam

.waitVisible:
    ldh a, [rLY]
    cp 144
    jr nc, .waitVisible
    jr MainLoop

ReadInput:
    ld a, [wState]
    cp STATE_BRIEFING
    jr z, ReadBriefing
    cp STATE_CORRIDOR
    jr z, ReadCorridor
    jp ReadTerminal

ReadBriefing:
    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    bit 0, a
    jr z, .done
    call EnterCorridor
.done:
    ld a, $30
    ldh [rP1], a
    ret

EnterCorridor:
    ld a, STATE_CORRIDOR
    ld [wState], a
    xor a
    ld [wArmed], a

    ; Remove all briefing evidence.
    ld hl, CUE_MAP
    xor a
    ld [hli], a
    ld [hli], a
    ld [hl], a

    ; Visible wall forces the same detour in both variants.
    ld hl, $9800 + 9
    ld b, 12
.drawWall:
    ld a, 6
    ld [hl], a
    ld de, 32
    add hl, de
    dec b
    jr nz, .drawWall

    ; Identical exit marker.
    ld hl, EXIT_MAP
    ld a, 2
    ld [hl], a

    ld a, CORRIDOR_START_Y
    ld [wPlayerY], a
    ld a, CORRIDOR_START_X
    ld [wPlayerX], a
    ret

ReadCorridor:
    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    ld b, a

    ; Right. Above the gap, stop immediately left of the wall.
    bit 0, b
    jr z, .left
    ld a, [wPlayerY]
    cp WALL_GAP_Y
    jr nc, .rightBounds
    ld a, [wPlayerX]
    cp WALL_LEFT_STOP_X
    jr z, .left
.rightBounds:
    ld a, [wPlayerX]
    cp 160
    jr nc, .left
    add MOVE_STEP
    ld [wPlayerX], a

.left:
    ; Left. Above the gap, stop immediately right of the wall.
    bit 1, b
    jr z, .up
    ld a, [wPlayerY]
    cp WALL_GAP_Y
    jr nc, .leftBounds
    ld a, [wPlayerX]
    cp WALL_RIGHT_STOP_X
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
    ld a, [wPlayerY]
    cp WALL_GAP_Y
    jr nz, .upBounds
    ld a, [wPlayerX]
    cp WALL_LEFT_STOP_X + 1
    jr c, .upBounds
    cp WALL_RIGHT_STOP_X
    jr c, .down
.upBounds:
    ld a, [wPlayerY]
    cp 18
    jr c, .down
    sub MOVE_STEP
    ld [wPlayerY], a

.down:
    bit 3, b
    jr z, .checkExit
    ld a, [wPlayerY]
    cp 120 ; screen y 104 max, safely away from terminal target y 112
    jr nc, .checkExit
    add MOVE_STEP
    ld [wPlayerY], a

.checkExit:
    call CheckCorridorExit
    ld a, $30
    ldh [rP1], a
    ret

CheckCorridorExit:
    ld a, [wPlayerX]
    cp CORRIDOR_EXIT_X
    ret c
    ld a, [wPlayerY]
    cp CORRIDOR_EXIT_Y_MAX + 1
    ret nc
    jp EnterTerminal

EnterTerminal:
    ld a, STATE_TERMINAL
    ld [wState], a
    xor a
    ld [wArmed], a

    ; Remove corridor wall.
    ld hl, $9800 + 9
    ld b, 12
.clearWall:
    xor a
    ld [hl], a
    ld de, 32
    add hl, de
    dec b
    jr nz, .clearWall

    ld hl, EXIT_MAP
    xor a
    ld [hl], a

    ; Two visually identical terminals.
    ld hl, LEFT_DOOR_MAP
    ld a, 2
    ld [hl], a
    ld hl, RIGHT_DOOR_MAP
    ld [hl], a

    ld a, TERMINAL_START_Y
    ld [wPlayerY], a
    ld a, TERMINAL_START_X
    ld [wPlayerX], a
    ret

ReadTerminal:
    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    ld b, a

    ; Require a neutral D-pad frame after entering the terminal room.
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

    ; 1 solid player, unique 64-dark-pixel locator.
    REPT 8
        db %11111111, %11111111
    ENDR

    ; 2 identical door / terminal outline.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10100101, %10100101

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

    ; 5 arrow shaft.
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000
    db %11111111, %11111111
    db %11111111, %11111111
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000

    ; 6 striped corridor wall, 32 dark pixels.
    REPT 8
        db %10101010, %10101010
    ENDR
TileDataEnd:

SECTION "State", WRAM0[$C000]
wPlayerX: ds 1
wPlayerY: ds 1
wState:   ds 1
wArmed:   ds 1
