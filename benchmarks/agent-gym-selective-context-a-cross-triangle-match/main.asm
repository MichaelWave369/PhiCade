; PhiCade Φ-Agent Gym Selective Context Routing LAYOUT A / CROSS / TRIANGLE / MATCH v1
; SPDX-License-Identifier: MIT
;
; Two-bank addressed relational-memory probe:
; 1. CIRCLE and CROSS banks both contain TRIANGLE/SQUARE bindings.
; 2. The banks use opposite left/right arrangements.
; 3. A erases all bank markers and position-bearing briefing evidence.
; 4. After a lockout, choice reveals bank selector + query + MATCH/FLIP.
; 5. The controller must retrieve from the selected bank, apply the operator,
;    choose between identical doors, and press A.
; 6. Wrong commitment is terminal.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF PLAYER_START_X EQU 80
DEF PLAYER_START_Y EQU 112
DEF MOVE_STEP      EQU 2
DEF WAIT_FRAMES    EQU 90

DEF STATE_BRIEF    EQU 0
DEF STATE_WAIT     EQU 1
DEF STATE_CHOICE   EQU 2
DEF STATE_DONE     EQU 3
DEF STATE_FAIL     EQU 4

DEF CIRCLE_MARK_MAP   EQU $9800 + 5 * 32 + 2
DEF CIRCLE_LEFT_MAP   EQU $9800 + 5 * 32 + 6
DEF CIRCLE_RIGHT_MAP  EQU $9800 + 5 * 32 + 14
DEF CROSS_MARK_MAP    EQU $9800 + 8 * 32 + 2
DEF CROSS_LEFT_MAP    EQU $9800 + 8 * 32 + 6
DEF CROSS_RIGHT_MAP   EQU $9800 + 8 * 32 + 14

DEF SELECTOR_MAP      EQU $9800 + 4 * 32 + 6
DEF QUERY_MAP         EQU $9800 + 4 * 32 + 9
DEF OPERATOR_MAP      EQU $9800 + 4 * 32 + 12
DEF LEFT_DOOR_MAP     EQU $9800 + 12 * 32 + 3
DEF RIGHT_DOOR_MAP    EQU $9800 + 12 * 32 + 15
DEF STATUS_MAP        EQU $9800 + 1 * 32 + 1

DEF CIRCLE_LEFT_TILE  EQU 2
DEF CIRCLE_RIGHT_TILE EQU 3
DEF CROSS_LEFT_TILE   EQU 3
DEF CROSS_RIGHT_TILE  EQU 2
DEF SELECTOR_TILE     EQU 10
DEF QUERY_TILE        EQU 2
DEF OPERATOR_TILE     EQU 7
DEF CORRECT_X         EQU 128
DEF WRONG_X           EQU 32
DEF CORRECT_MIN_X     EQU CORRECT_X - 3
DEF CORRECT_MAX_X     EQU CORRECT_X + 4
DEF WRONG_MIN_X       EQU WRONG_X - 3
DEF WRONG_MAX_X       EQU WRONG_X + 4

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

    ld hl, CIRCLE_MARK_MAP
    ld a, 9
    ld [hl], a
    ld hl, CIRCLE_LEFT_MAP
    ld a, CIRCLE_LEFT_TILE
    ld [hl], a
    ld hl, CIRCLE_RIGHT_MAP
    ld a, CIRCLE_RIGHT_TILE
    ld [hl], a

    ld hl, CROSS_MARK_MAP
    ld a, 10
    ld [hl], a
    ld hl, CROSS_LEFT_MAP
    ld a, CROSS_LEFT_TILE
    ld [hl], a
    ld hl, CROSS_RIGHT_MAP
    ld a, CROSS_RIGHT_TILE
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
    ld [wTimer], a
    ld [wArmed], a

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
    cp STATE_BRIEF
    jr z, ReadBrief
    cp STATE_CHOICE
    jp z, ReadChoice
    ret

ReadBrief:
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

    ; Erase both memory banks and every position-bearing briefing pixel.
    ld hl, CIRCLE_MARK_MAP
    ld [hl], a
    ld hl, CIRCLE_LEFT_MAP
    ld [hl], a
    ld hl, CIRCLE_RIGHT_MAP
    ld [hl], a
    ld hl, CROSS_MARK_MAP
    ld [hl], a
    ld hl, CROSS_LEFT_MAP
    ld [hl], a
    ld hl, CROSS_RIGHT_MAP
    ld [hl], a

    ; Choice scene reveals only selector + query + operator.
    ld hl, SELECTOR_MAP
    ld a, SELECTOR_TILE
    ld [hl], a
    ld hl, QUERY_MAP
    ld a, QUERY_TILE
    ld [hl], a
    ld hl, OPERATOR_MAP
    ld a, OPERATOR_TILE
    ld [hl], a
    ld hl, LEFT_DOOR_MAP
    ld a, 4
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

    and a
    jr nz, .movement
    ld a, 1
    ld [wArmed], a

.movement:
    ld a, [wArmed]
    and a
    jr z, .action

    bit 1, b
    jr z, .right
    ld a, [wPlayerX]
    cp 32
    jr z, .right
    cp 32
    jr c, .right
    sub MOVE_STEP
    ld [wPlayerX], a

.right:
    bit 0, b
    jr z, .action
    ld a, [wPlayerX]
    cp 128
    jr z, .action
    cp 128
    jr nc, .action
    add MOVE_STEP
    ld [wPlayerX], a

.action:
    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    cpl
    and $0F
    bit 0, a
    jr z, .done

    ld a, [wPlayerX]
    cp CORRECT_MIN_X
    jr c, .checkWrong
    cp CORRECT_MAX_X
    jr c, .accept
.checkWrong:
    ld a, [wPlayerX]
    cp WRONG_MIN_X
    jr c, .done
    cp WRONG_MAX_X
    jr c, .fail
    jr .done

.accept:
    ld a, STATE_DONE
    ld [wState], a
    ld a, CORRECT_X
    ld [wPlayerX], a
    ld hl, STATUS_MAP
    ld a, 5
    ld [hl], a
    jr .done

.fail:
    ld a, STATE_FAIL
    ld [wState], a
    ld a, PLAYER_START_X
    ld [wPlayerX], a
    ld hl, STATUS_MAP
    ld a, 6
    ld [hl], a

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

    ; 1 solid player.
    REPT 8
        db %11111111, %11111111
    ENDR

    ; 2 TRIANGLE.
    db %00011000, %00011000
    db %00111100, %00111100
    db %01100110, %01100110
    db %11000011, %11000011
    db %11000011, %11000011
    db %11111111, %11111111
    db %00000000, %00000000
    db %00000000, %00000000

    ; 3 SQUARE.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %11111111, %11111111

    ; 4 identical door.
    db %11111111, %11111111
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10000001, %10000001
    db %10100101, %10100101

    ; 5 PASS.
    db %11110000, %11110000
    db %10010000, %10010000
    db %11110000, %11110000
    db %10000000, %10000000
    db %10000000, %10000000
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000

    ; 6 FAIL.
    db %11110000, %11110000
    db %10000000, %10000000
    db %11100000, %11100000
    db %10000000, %10000000
    db %10000000, %10000000
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000

    ; 7 MATCH (=).
    db %00000000, %00000000
    db %00000000, %00000000
    db %01111110, %01111110
    db %00000000, %00000000
    db %01111110, %01111110
    db %00000000, %00000000
    db %00000000, %00000000
    db %00000000, %00000000

    ; 8 FLIP (X).
    db %11000011, %11000011
    db %01100110, %01100110
    db %00111100, %00111100
    db %00011000, %00011000
    db %00011000, %00011000
    db %00111100, %00111100
    db %01100110, %01100110
    db %11000011, %11000011

    ; 9 CIRCLE bank marker.
    db %00111100, %00111100
    db %01100110, %01100110
    db %11000011, %11000011
    db %11000011, %11000011
    db %11000011, %11000011
    db %11000011, %11000011
    db %01100110, %01100110
    db %00111100, %00111100

    ; 10 CROSS bank marker.
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
wState:   ds 1
wTimer:   ds 1
wArmed:   ds 1
