; PhiCade Φ-Agent Gym Nested Branch Graph SQUARE → CROSS v1
; SPDX-License-Identifier: MIT
;
; Two-level conditional branch probe:
; 1. Stage 1 renders TRIANGLE left and SQUARE right plus one selector.
; 2. Correct Stage 1 commitment reveals Stage 2 only after the first decision.
; 3. Stage 2 renders CIRCLE left and CROSS right plus a new selector.
; 4. Wrong commitment at either stage enters an irreversible FAIL state.
; 5. Correct Stage 2 commitment converges onto one shared generator/gate/target chain.

DEF rP1   EQU $FF00
DEF rLCDC EQU $FF40
DEF rBGP  EQU $FF47
DEF rOBP0 EQU $FF48
DEF rLY   EQU $FF44

DEF MOVE_STEP EQU 2

; OAM coordinates = screen coordinate + (8,16).
DEF CENTER_X          EQU 80
DEF PLAYER_START_Y    EQU 128
DEF STAGE1_LEFT_X     EQU 32
DEF STAGE1_RIGHT_X    EQU 128
DEF STAGE1_Y          EQU 128
DEF STAGE2_LEFT_X     EQU 32
DEF STAGE2_RIGHT_X    EQU 128
DEF STAGE2_Y          EQU 96
DEF STAGE2_MIN_Y      EQU STAGE2_Y - 2
DEF STAGE2_MAX_Y      EQU STAGE2_Y + 3
DEF GENERATOR_X       EQU 80
DEF GENERATOR_Y       EQU 80
DEF GENERATOR_MIN_Y   EQU GENERATOR_Y - 2
DEF GENERATOR_MAX_Y   EQU GENERATOR_Y + 3
DEF GATE_CENTER_X     EQU 80
DEF GATE_STOP_Y       EQU 72
DEF GATE_MIN_Y        EQU GATE_STOP_Y - 2
DEF GATE_MAX_Y        EQU GATE_STOP_Y + 3
DEF TARGET_X          EQU 80
DEF TARGET_Y          EQU 40

DEF STAGE1_LEFT_MAP   EQU $9800 + 14 * 32 + 3
DEF STAGE1_RIGHT_MAP  EQU $9800 + 14 * 32 + 15
DEF STAGE2_LEFT_MAP   EQU $9800 + 10 * 32 + 3
DEF STAGE2_RIGHT_MAP  EQU $9800 + 10 * 32 + 15
DEF SELECTOR_MAP      EQU $9800 + 4 * 32 + 9
DEF BADGE_MAP         EQU $9800 + 1 * 32 + 1
DEF GENERATOR_MAP     EQU $9800 + 9 * 32 + 11
DEF GATE_ROW          EQU $9800 + 7 * 32
DEF STAGE1_SELECTOR_TILE EQU 7
DEF STAGE2_SELECTOR_TILE EQU 11

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

    ; Shared closed gate.
    ld hl, GATE_ROW
    ld b, 20
.drawGate:
    ld a, 4
    ld [hli], a
    dec b
    jr nz, .drawGate

    ; Shared generator starts OFF.
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a

    ; Stage 1 always exposes the same two candidate families.
    ld hl, STAGE1_LEFT_MAP
    ld a, 3
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld a, 7
    ld [hl], a

    ; Only the Stage 1 selector differs across family variants.
    ld hl, SELECTOR_MAP
    ld a, 7
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
    ld a, CENTER_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, 1
    ld [$FE02], a
    xor a
    ld [$FE03], a

    ; Shared visible X target.
    ld a, TARGET_Y
    ld [$FE04], a
    ld a, TARGET_X
    ld [$FE05], a
    ld a, 2
    ld [$FE06], a
    xor a
    ld [$FE07], a

    xor a
    ld [wStage1Done], a
    ld [wStage2Done], a
    ld [wFailed], a
    ld [wPowerOn], a
    ld [wGateOpen], a
    ld [wMoveLock], a

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
    call RenderState

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
    jp z, .done

    ld a, [wFailed]
    and a
    jp nz, .done

    ld a, [wPowerOn]
    and a
    jp nz, .tryGate

    ld a, [wStage2Done]
    and a
    jp nz, .tryGenerator

    ld a, [wStage1Done]
    and a
    jp nz, .tryStage2

.tryStage1:
    ld a, [wPlayerY]
    cp STAGE1_Y
    jp nz, .done
    ld a, [wPlayerX]
    cp STAGE1_RIGHT_X
    jr z, .acceptStage1
    cp STAGE1_LEFT_X
    jr z, .fail
    jp .done

.acceptStage1:
    ld a, 1
    ld [wStage1Done], a

    ; Erase first-level evidence. The second selector is revealed only now.
    xor a
    ld hl, STAGE1_LEFT_MAP
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld [hl], a

    ld hl, STAGE2_LEFT_MAP
    ld a, 10
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld a, 11
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, 11
    ld [hl], a
    jp .done

.tryStage2:
    ; Use a five-pixel rendered overlap band instead of one sacred y pixel.
    ; The branch choice remains exact in x and selector identity.
    ld a, [wPlayerY]
    cp STAGE2_MIN_Y
    jp c, .done
    cp STAGE2_MAX_Y
    jp nc, .done
    ld a, [wPlayerX]
    cp STAGE2_RIGHT_X
    jr z, .acceptStage2
    cp STAGE2_LEFT_X
    jr z, .fail
    jp .done

.acceptStage2:
    ld a, 1
    ld [wStage2Done], a

    ; Stage 2 is the branch-graph convergence boundary. Normalize the actor
    ; onto the shared center before erasing variant-specific evidence.
    ld a, CENTER_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, STAGE2_Y
    ld [wPlayerY], a
    ld [$FE00], a
    ld a, 1
    ld [wMoveLock], a

    ; Erase second-level evidence and converge every variant.
    xor a
    ld hl, STAGE2_LEFT_MAP
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld [hl], a
    ld hl, BADGE_MAP
    ld a, 8
    ld [hl], a
    jp .done

.fail:
    ld a, 1
    ld [wFailed], a

    ; Failure is also a convergence boundary. A Stage 2 failure normalizes
    ; both axes onto the Stage 2 seam; Stage 1 failure preserves its row.
    ld a, [wStage1Done]
    and a
    jr z, .failNormalizeX
    ld a, STAGE2_Y
    ld [wPlayerY], a
.failNormalizeX:
    ld a, CENTER_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, [wPlayerY]
    ld [$FE00], a
    ld a, 1
    ld [wMoveLock], a

    ; Any wrong commitment collapses onto one terminal FAIL world.
    xor a
    ld hl, STAGE1_LEFT_MAP
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld [hl], a
    ld hl, STAGE2_LEFT_MAP
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld [hl], a
    ld hl, BADGE_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, 9
    ld [hl], a
    jp .done

.tryGenerator:
    ld a, [wPlayerX]
    cp GENERATOR_X
    jp nz, .done
    ld a, [wPlayerY]
    cp GENERATOR_MIN_Y
    jp c, .done
    cp GENERATOR_MAX_Y
    jp nc, .done

    ; A successful generator interaction owns the boundary. Normalize the
    ; actor and lock movement for this frame before mutating power state.
    ld a, GENERATOR_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, GENERATOR_Y
    ld [wPlayerY], a
    ld [$FE00], a
    ld a, 1
    ld [wPowerOn], a
    ld [wMoveLock], a
    jp .done

.tryGate:
    ld a, [wGateOpen]
    and a
    jp nz, .done
    ld a, [wPlayerX]
    cp GATE_CENTER_X
    jp nz, .done
    ld a, [wPlayerY]
    cp GATE_MIN_Y
    jp c, .done
    cp GATE_MAX_Y
    jp nc, .done

    ; Gate activation is another authoritative convergence boundary.
    ld a, GATE_CENTER_X
    ld [wPlayerX], a
    ld [$FE01], a
    ld a, GATE_STOP_Y
    ld [wPlayerY], a
    ld [$FE00], a
    ld a, 1
    ld [wGateOpen], a
    ld [wMoveLock], a

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

    ; Branch commitment and failure transitions own this frame. Suppress any
    ; stale directional state that the host may still expose during the same
    ; emulated frame, then normalize converged worlds onto the shared x seam.
    ld a, [wMoveLock]
    and a
    jr z, .checkConverged
    xor a
    ld [wMoveLock], a
    ld a, $30
    ldh [rP1], a
    ret

.checkConverged:
    ld a, [wFailed]
    and a
    jr nz, .forceCenter
    ld a, [wStage2Done]
    and a
    jr z, .horizontal

.forceCenter:
    ld a, CENTER_X
    ld [wPlayerX], a
    jp .up

.horizontal:
    bit 0, b
    jr z, .left

    ld a, [wFailed]
    and a
    jr nz, .rightClampCenter
    ld a, [wStage2Done]
    and a
    jr nz, .rightClampCenter

    ld a, [wStage1Done]
    and a
    jr z, .rightStage1

    ld a, [wPlayerY]
    cp STAGE1_Y
    jr z, .rightClampCenter
    cp STAGE2_Y
    jr nz, .rightBounds
    ld a, [wPlayerX]
    cp STAGE2_RIGHT_X
    jr z, .left
    jr .rightBounds

.rightStage1:
    ld a, [wPlayerY]
    cp STAGE1_Y
    jr nz, .rightBounds
    ld a, [wPlayerX]
    cp STAGE1_RIGHT_X
    jr z, .left
    jr .rightBounds

.rightClampCenter:
    ld a, [wPlayerX]
    cp CENTER_X
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

    ld a, [wFailed]
    and a
    jr nz, .leftClampCenter
    ld a, [wStage2Done]
    and a
    jr nz, .leftClampCenter

    ld a, [wStage1Done]
    and a
    jr z, .leftStage1

    ld a, [wPlayerY]
    cp STAGE1_Y
    jr z, .leftClampCenter
    cp STAGE2_Y
    jr nz, .leftBounds
    ld a, [wPlayerX]
    cp STAGE2_LEFT_X
    jr z, .up
    jr .leftBounds

.leftStage1:
    ld a, [wPlayerY]
    cp STAGE1_Y
    jr nz, .leftBounds
    ld a, [wPlayerX]
    cp STAGE1_LEFT_X
    jr z, .up
    jr .leftBounds

.leftClampCenter:
    ld a, [wPlayerX]
    cp CENTER_X
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

    ; Stage 2 cannot be reached until Stage 1 is accepted.
    ld a, [wStage1Done]
    and a
    jr nz, .upGateCheck
    ld a, [wFailed]
    and a
    jr nz, .upGateCheck
    ld a, [wPlayerY]
    cp PLAYER_START_Y
    jr z, .down

.upGateCheck:
    ; Stage 2 acceptance creates a hard generator stop. Approach from below
    ; snaps the final <=2 px onto the exact interaction coordinate.
    ld a, [wStage2Done]
    and a
    jr z, .upGateBarrier
    ld a, [wPowerOn]
    and a
    jr nz, .upGateBarrier
    ld a, [wPlayerY]
    cp GENERATOR_Y
    jr z, .down
    cp GENERATOR_Y + 3
    jr nc, .upGateBarrier
    ld a, GENERATOR_Y
    ld [wPlayerY], a
    jr .down

.upGateBarrier:
    ld a, [wGateOpen]
    and a
    jr nz, .upTarget
    ld a, [wPlayerY]
    cp GATE_STOP_Y
    jr z, .down
    cp GATE_STOP_Y + 3
    jr nc, .upBounds
    ld a, GATE_STOP_Y
    ld [wPlayerY], a
    jr .down

.upTarget:
    ld a, [wPlayerY]
    cp TARGET_Y
    jr z, .down
    cp TARGET_Y + 3
    jr nc, .upBounds
    ld a, TARGET_Y
    ld [wPlayerY], a
    jr .down

.upBounds:
    ld a, [wPlayerY]
    cp 18
    jr c, .down
    sub MOVE_STEP
    ld [wPlayerY], a

.down:
    bit 3, b
    jr z, .doneMove
    ld a, [wPlayerY]
    cp 144
    jr nc, .doneMove
    add MOVE_STEP
    ld [wPlayerY], a

.doneMove:
    ld a, $30
    ldh [rP1], a
    ret

RenderState:
    ; Rendering is a deterministic projection of WRAM state. This repairs any
    ; transient tile-map write that missed its one-shot transition frame and
    ; makes convergence evidence depend on state, not timing.

    ld a, [wFailed]
    and a
    jp nz, .failed

    ld a, [wStage2Done]
    and a
    jp nz, .accepted

    ld a, [wStage1Done]
    and a
    jp nz, .stage2

.initial:
    ld hl, STAGE1_LEFT_MAP
    ld a, 3
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld a, 7
    ld [hl], a
    ld hl, STAGE2_LEFT_MAP
    xor a
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, STAGE1_SELECTOR_TILE
    ld [hl], a
    ld hl, BADGE_MAP
    xor a
    ld [hl], a
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a
    jp .gate

.stage2:
    ld hl, STAGE1_LEFT_MAP
    xor a
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld [hl], a
    ld hl, STAGE2_LEFT_MAP
    ld a, 10
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld a, 11
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, STAGE2_SELECTOR_TILE
    ld [hl], a
    ld hl, BADGE_MAP
    xor a
    ld [hl], a
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a
    jp .gate

.accepted:
    ld hl, STAGE1_LEFT_MAP
    xor a
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld [hl], a
    ld hl, STAGE2_LEFT_MAP
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld [hl], a

    ld a, [wPowerOn]
    and a
    jr nz, .powered

    ld hl, BADGE_MAP
    ld a, 8
    ld [hl], a
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a
    jr .gate

.powered:
    ld hl, BADGE_MAP
    xor a
    ld [hl], a
    ld hl, GENERATOR_MAP
    ld a, 6
    ld [hl], a
    jr .gate

.failed:
    ld hl, STAGE1_LEFT_MAP
    xor a
    ld [hl], a
    ld hl, STAGE1_RIGHT_MAP
    ld [hl], a
    ld hl, STAGE2_LEFT_MAP
    ld [hl], a
    ld hl, STAGE2_RIGHT_MAP
    ld [hl], a
    ld hl, BADGE_MAP
    ld [hl], a
    ld hl, SELECTOR_MAP
    ld a, 9
    ld [hl], a
    ld hl, GENERATOR_MAP
    ld a, 5
    ld [hl], a

.gate:
    ld a, [wGateOpen]
    and a
    jr nz, .gateOpen
    ld a, 4
    jr .writeGate

.gateOpen:
    xor a

.writeGate:
    ld hl, GATE_ROW
    ld b, 20
.writeGateLoop:
    ld [hli], a
    dec b
    jr nz, .writeGateLoop
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

    ; 1 player.
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

    ; 3 TRIANGLE.
    db %00010000, %00010000
    db %00111000, %00111000
    db %00111000, %00111000
    db %01111100, %01111100
    db %01111100, %01111100
    db %11111110, %11111110
    db %11111110, %11111110
    db %00000000, %00000000

    ; 4 gate.
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

    ; 7 SQUARE.
    db %01111110, %01111110
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01000010, %01000010
    db %01111110, %01111110
    db %00000000, %00000000

    ; 8 shared accepted badge.
    db %00111100, %00111100
    db %00100100, %00100100
    db %01111110, %01111110
    db %01011010, %01011010
    db %01011010, %01011010
    db %01111110, %01111110
    db %00100100, %00100100
    db %00111100, %00111100

    ; 9 FAIL marker.
    db %10000001, %10000001
    db %01000010, %01000010
    db %00100100, %00100100
    db %00011000, %00011000
    db %00011000, %00011000
    db %00100100, %00100100
    db %01000010, %01000010
    db %10000001, %10000001

    ; 10 CIRCLE.
    db %00111100, %00111100
    db %01100110, %01100110
    db %11000011, %11000011
    db %10000001, %10000001
    db %10000001, %10000001
    db %11000011, %11000011
    db %01100110, %01100110
    db %00111100, %00111100

    ; 11 CROSS/PLUS.
    db %00011000, %00011000
    db %00011000, %00011000
    db %00011000, %00011000
    db %11111111, %11111111
    db %11111111, %11111111
    db %00011000, %00011000
    db %00011000, %00011000
    db %00011000, %00011000
TileDataEnd:

SECTION "State", WRAM0[$C000]
wPlayerX:    ds 1
wPlayerY:    ds 1
wStage1Done: ds 1
wStage2Done: ds 1
wFailed:     ds 1
wPowerOn:    ds 1
wGateOpen:   ds 1
wMoveLock:   ds 1
