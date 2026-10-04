# Benchmark Suite v10

Suite v10 is PhiCade's 29-task frozen benchmark population.

It preserves the exact 21 tasks from Suite v9 and adds eight Compositional
Recall tasks. No prior task identity, source hash, ROM hash, oracle, prompt,
geometry, or suite origin is mutated.

## New 2×2×2 factorial

| Arrangement | Query | Operator | Correct |
| --- | --- | --- | --- |
| NORMAL | TRIANGLE | MATCH | LEFT |
| NORMAL | TRIANGLE | FLIP | RIGHT |
| NORMAL | SQUARE | MATCH | RIGHT |
| NORMAL | SQUARE | FLIP | LEFT |
| SWAPPED | TRIANGLE | MATCH | RIGHT |
| SWAPPED | TRIANGLE | FLIP | LEFT |
| SWAPPED | SQUARE | MATCH | LEFT |
| SWAPPED | SQUARE | FLIP | RIGHT |

## Frozen provenance

### NORMAL / TRIANGLE / MATCH
- source SHA-256: `d9c6f0e0462502a2e5906a9a8f07861526bdc65d2f99758db57b630e6233d6e7`
- ROM SHA-256: `a0dd1c4fd34a308c9951fee576f687a9337bef9c1dd05d38e751ef48e7949720`

### NORMAL / TRIANGLE / FLIP
- source SHA-256: `8b555998f193fd5cb40e61358da986119698e522938e213670c6e1d3c6a3dde4`
- ROM SHA-256: `3dcf0413ad3d62a49d45167a108a7483755925259ec8527ef2e9772d07b4f8d0`

### NORMAL / SQUARE / MATCH
- source SHA-256: `58659c6891709c5524644f40b0626ec0fca1df7ba5c36fada31f6a0611374eee`
- ROM SHA-256: `8aa58e9e982594462cce0b48aab2a9b6a060537e1a404b185511f9b6598f4afe`

### NORMAL / SQUARE / FLIP
- source SHA-256: `f77ebc811b75f15d47845246e702650d4f27129f5329eb767bd1c19b7d3c0123`
- ROM SHA-256: `aa10327cd7ef0d06b8506d062b72bcfe6d4199759def0f31fdd6e44da6beb83e`

### SWAPPED / TRIANGLE / MATCH
- source SHA-256: `b145f5b94fd0af906020257c07d8f4167c70b6e95fbc37677e92f427e1dc98d2`
- ROM SHA-256: `7732b4fc7220698bb20e3a40fd6c8bb85fdfa608bcc75f9135c6743e2453da57`

### SWAPPED / TRIANGLE / FLIP
- source SHA-256: `2bab8e7b334d0fbeeb6c1f3901fe51aec2fbddeb450ed6d710853e6058362108`
- ROM SHA-256: `5717f04ab690b1a887448205358f17aee64a590e3a0011b7fec039183ff6bea4`

### SWAPPED / SQUARE / MATCH
- source SHA-256: `886a3dab4a3814b8a9723e94353daff511f1c510976e7f3766a84dbda8ac75b9`
- ROM SHA-256: `1174a6b2b158b3712a9a71b880983259bfe4a8e26f166193f56e86e5b09886dd`

### SWAPPED / SQUARE / FLIP
- source SHA-256: `1e1f8380981896ddca16fa5f89ab7915175e94476e2c3cf2fdd425fb895f2613`
- ROM SHA-256: `c86b4bbacc6e9bedc9d731d2b396d5bef64889ecca2243031289d7f1020a996b`

These hashes were observed from the pinned RGBDS v1.0.3 CI build before registry
freeze.

## Coverage gate

Native qualification requires:

- Suite v9 remains READY at 21/21 using its exact prior population;
- Suite v10 is INCOMPLETE at 21/29;
- remains INCOMPLETE at every intermediate coverage from 22/29 through 28/29;
- becomes READY only at 29/29.

## Capability increment

Suite v9 requires relational recall:

`query + remembered binding -> side`

Suite v10 requires retrieval plus transformation:

`query + remembered binding + current operator -> transformed side`

MATCH preserves the remembered side. FLIP inverts it. The full factorial caps
always-left, always-right, ignore-operator, and always-flip shortcuts at 4/8.

See `COMPOSITIONAL_RECALL_BENCHMARK.md` for the behavioral qualification
contract.
