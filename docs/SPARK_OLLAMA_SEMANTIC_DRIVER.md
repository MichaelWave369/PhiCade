# PhiCade Rung 48 — Local Ollama SPARK Semantic Driver

Rung 48 attaches a real local model provider to the governed SPARK semantic
control path created in Rung 47.

The architecture remains:

```text
local Ollama model
      ↓ structured semantic decision
PhiCade semantic turn contract
      ↓ ActionEnvelope
PhiCade AuthorityPolicy
      ↓ authorized gameplay action
PixelForge Runtime Bridge v1 / JSONL
      ↓
SPARK Threshold adapter
      ↓
real SPARK action-engine
```

No model receives a privileged SPARK object, save handle, process handle, or
runtime adapter handle.

## Why this is a separate semantic provider path

PhiCade's original Ollama path is framebuffer-oriented and requires a rendered
image observation. The current SPARK bridge deliberately does not claim
framebuffer streaming.

Rung 48 therefore adds a **text/semantic Ollama path** instead of forging a fake
framebuffer capability.

This also means a local model does not need to advertise Ollama vision support
to participate in this specific SPARK semantic playtest. Its exact installed
model digest is still inspected and recorded.

## Semantic turn contract

Request schema:

```text
phicade.spark-agent-turn-request.v1
```

Response schema:

```text
phicade.spark-agent-turn-response.v1
```

A request binds the model decision to:

- exact semantic observation tick;
- SPARK runtime hash at that observation;
- explicit agent ID and seat;
- explicit allowed-button grant;
- one-action maximum;
- bounded working memory plus its SHA-256;
- bounded replacement-memory budget.

A response is rejected if it is stale, names a different agent/seat, references
a different runtime hash, exceeds the memory budget, proposes more than one
action, delays the action, releases a button, or names an out-of-grant button.

## Ollama prompt boundary

The provider prompt contains only the already-qualified bounded semantic
projection:

- room, world, phase and vessel;
- player position, HP and dash readiness;
- vessel power and cooldown;
- enemy count and nearest enemy;
- visible fragment count;
- exits;
- allowed actions;
- dash and power counters;
- bounded model-authored working memory.

It explicitly tells the model that memory is not authoritative and that PhiCade
owns authority.

No full SPARK run, save state, inventory internals, D1 data, hidden rooms,
Memory Arcade storage, or unrelated campaign state is injected.

## Real local-model harness

Run from the PhiCade repository:

```bash
bash ./scripts/qualify-spark-ollama.sh \
  /path/to/parallax-pixelforge \
  /path/to/SparkTheSubstrate \
  qwen3.6:latest \
  3
```

Or invoke the Cargo example directly:

```bash
cargo run --manifest-path src-tauri/Cargo.toml \
  --example qualify_spark_ollama -- \
  --pixelforge-root /path/to/parallax-pixelforge \
  --spark-root /path/to/SparkTheSubstrate \
  --model qwen3.6:latest \
  --turns 3
```

The harness permits 1 through 8 turns.

The Ollama endpoint remains loopback-only. The default is:

```text
http://127.0.0.1:11434
```

## Exact source requirement

The local playtest currently requires the same already-qualified bridge pins:

PixelForge:

```text
8790c3b00b5184136fb8fa6a2fbdabe834eeca22
```

SPARK bridge source:

```text
fae7879820bef63a550fea486b2defbc3cee5304
```

The harness refuses a different repository HEAD rather than silently claiming
evidence for unqualified source.

## What one playtest turn does

For each turn:

1. observe real SPARK through PixelForge;
2. project it into `phicade.spark-semantic-observation.v1`;
3. capture the SPARK runtime hash;
4. send the bounded request to the selected local Ollama model;
5. parse Ollama structured output;
6. validate the response against the exact observation/hash/grant;
7. compile it to a Phi-Bot ActionEnvelope;
8. run PhiCade AuthorityPolicy;
9. translate only an accepted action to SPARK MOVE, DASH, or PULSE;
10. submit through PixelForge;
11. advance the real SPARK engine;
12. update bounded memory only from a validated model proposal.

An empty action array is treated as a governed wait. The SPARK runtime still
advances one externally stepped tick.

The playtest only passes if the model executes at least one authorized gameplay
action and the SPARK runtime hash changes.

## Receipt

Schema:

```text
phicade.spark-ollama-playtest.v1
```

Default artifact:

```text
artifacts/phicade-spark-ollama-playtest.json
```

The receipt records:

- PhiCade, PixelForge and SPARK revisions;
- provider name;
- exact installed model name and digest;
- advertised model capabilities;
- each semantic observation and its runtime hash;
- each model response;
- each PhiCade authority decision;
- each submitted SPARK intent;
- direct SPARK events;
- cumulative semantic events;
- bounded memory evolution;
- initial and final SPARK runtime hashes.

The model digest is inspected again after the run. If it changed during the
playtest, the run fails rather than binding evidence to an unstable identity.

## CI versus actual model evidence

GitHub CI can compile the harness and test the semantic Ollama protocol against a
loopback mock server. CI does **not** contain Mikey's local model weights and
therefore must not claim that a real local model completed a SPARK run.

A real `phicade.spark-ollama-playtest.v1` PASS receipt is created only when the
harness is executed against an installed local Ollama model.

That distinction is deliberate.

## Next rung

Once one or more local models have produced real PASS receipts, the next useful
step is to compare them under the exact same SPARK seed, turn budget, grant and
semantic observation policy.

That turns SPARK into a governed local-model evaluation environment instead of
just a game with an AI button attached to it.
