# Portable Evidence Bundle v1

Rung 43 packages one fully closed PhiCade Suite Report evidence graph into a
single deterministic ZIP32 archive.

The bundle is intended for review, publication, archival, or transfer to another
machine without requiring access to the original PhiCade app-data tree.

## Export flow

From the desktop **PUBLIC RESULT** lane:

1. Select one existing Suite Report.
2. Press **EXPORT EVIDENCE ZIP**.
3. PhiCade re-runs full Evidence Closure v1.
4. The exact verified evidence bytes are collected.
5. Public Result JSON and Markdown are regenerated deterministically from the
   same Suite Report.
6. A manifest is built with relative path, SHA-256, and byte length for each
   included file.
7. PhiCade writes one deterministic ZIP archive.

The bundle is refused if Evidence Closure fails.

## Bundle schema

The manifest uses:

`phicade.portable-evidence-bundle.v1`

with:

`recordStatus = VERIFIED_EXPORT`

The manifest records:

- suite ID;
- Suite Report ID;
- Suite Report SHA-256;
- exact model digest;
- every bundled relative path;
- SHA-256 of every bundled evidence/public-result file;
- byte size of every bundled evidence/public-result file.

## Layout

A typical archive has this shape:

```text
manifest.json
suite-report.json
public-result/
  result.json
  result.md
model-qualification/
  receipt.json
campaigns/
  <task-id>/
    campaign-000001.json
trials/
  <task-id>/
    run-000001.json
    ...
autodrive/
  <task-id>/
    run-000001.json
    ...
```

Task subdirectories prevent campaign/run-number collisions across benchmark
ROM namespaces.

## Included evidence

The archive contains the exact bytes already validated by Evidence Closure v1:

- the selected Suite Report;
- every referenced campaign receipt;
- every campaign trial's model-gameplay receipt;
- every trial's referenced Autodrive receipt;
- the exact SHA-pinned model qualification receipt.

It also includes freshly regenerated deterministic Public Suite Result JSON and
Markdown for the same report.

## Deliberate exclusions

Portable Evidence Bundle v1 does **not** include:

- game ROMs;
- benchmark ROM binaries;
- emulator/core binaries;
- model weights;
- screenshots or framebuffer captures;
- save RAM;
- save states;
- replay state blobs;
- commercial game data;
- ScummVM binaries or game assets.

The archive is an evidence package, not a content or executable distribution
format.

Hashes that identify excluded runtime/model/game artifacts remain present in the
receipts where the evidence model already records them.

## Deterministic ZIP

The packer is dependency-free ZIP32 using STORED entries.

Determinism comes from:

- lexicographically sorted entry names;
- fixed DOS timestamp fields;
- no host-specific file attributes;
- no current timestamp in the bundle manifest;
- exact source receipt bytes;
- deterministic Public Result rendering.

For the same closed evidence graph and same PhiCade bundle format version, the
resulting ZIP bytes are stable.

The desktop returns the final ZIP SHA-256.

## Safety bounds

The packer rejects:

- absolute paths;
- backslash paths;
- empty path segments;
- `.` / `..` traversal segments;
- non-printable filenames;
- forged Suite Report bytes that do not match the pinned report SHA;
- evidence payloads beyond the configured 64 MiB safety bound;
- ZIP32 overflows.

## Verification model

A recipient can first hash the ZIP, then inspect `manifest.json` and hash each
listed file.

The manifest is not a replacement for semantic verification. Full semantic
verification still means applying the PhiCade Evidence Closure rules to the
receipts inside the archive.

A future rung may add an offline bundle verifier that performs that check
without requiring the original app-data directory.
