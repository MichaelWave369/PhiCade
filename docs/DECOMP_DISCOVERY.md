# PhiCade Decomp Discovery | Browser Rung 1

**Status:** read-only, source-credited discovery interface. **Not an installer, emulator, or runtime launcher.**

## Why this lives in PhiCade

The PhiCade Xbox Edition already has a static browser gateway and a GitHub Pages
deployment. The new `xbox/discovery/` route is a self-contained browser feature,
not a new privileged Tauri command or libretro core. It can open on ordinary PCs
and phones after Pages deployment. The existing PhiCade desktop/runtime and
Scarlett Lab remain unchanged.

## External source and evidence

Decomp Buddy (Tanooki Studios) documents its public `schema: 1` catalog here:
https://tanookistudios.com/decomp-buddy/catalog.json

Source: https://github.com/TanookiStudios/decomp-buddy
Upstream generator: `src/publiccatalog.js` (read to understand the schema;
not copied into PhiCade). The upstream app is GPL-3.0. This feature is an
independently written browser/metadata adapter and **does not vendor** its
application code, binaries or branding. Project metadata is credited and linked
to the source page; only a limited subset of public facts is presented.

## Publishing and failure states

The existing Xbox Pages workflow builds the `xbox/` site under the Pages root.
The discovery route is `/PhiCade/discovery/`.

On an authorized `main` push:
1. Download only the public JSON catalog (max 20 MiB; 18-second timeout).
2. Require `schema === 1`, `games[]`, and bounded source count.
3. Deterministically sample **at most 600 entries** across the source order.
4. Keep only title, console, project, platform labels, source link,
   source-reported completeness, version and safe forge URL.
5. Write a snapshot **only to the ephemeral Pages deployment artifact**,
   not into git. Display the upstream total and source generation date.
6. If the source is unavailable, publish the separately labeled
   five-project starter index (not a fabricated 'fresh' result).

This avoids browser CORS issues, auto-installation, republishing art or binaries,
and surprise actions on users' local machines. **The snapshot is not live or
complete.** Users can open Decomp Buddy for its full current catalog.

PR checks run only fixture-based tests and never download upstream content.
`workflow_dispatch` can rebuild the site manually, subject to repo permissions.
No new scheduled task, account, server, secret, or background job is installed.

## UI behavior

Search and filter by title, original console, and upstream-listed target
platform. Source details link to `https://decompbuddy.com/games/<slug>`; only
approved HTTPS source-forge origins are linked. Dynamic text is inserted with
`textContent`. External tabs are opened with `noopener noreferrer`.

Any upstream completeness field is shown **as a source-reported label**, never
as independently qualified game compatibility. Missing completeness is labeled.
No claims about safe installers or scan results are promoted to facts.

## Safety and source boundary

- No ROM, disc, BIOS, archive, executable, private SDK, or decryption material.
- No license/right-to-distribute assumptions.
- No download, installation, browser protocol handler, executable launch,
  AI action, session grant, native API, or privileged behavior.
- No storing cookies, user activity analytics, credentials, or personal data.
- No implicit PixelForge or Scarlett Lab authority.
- Treat all source data as untrusted. Missing links are simply omitted.

## CI

```bash
node --test xbox/tests/*.test.mjs
node xbox/discovery/build-catalog.mjs --fixture xbox/discovery/seed.json --output /tmp/discovery-test.json
```

Neither command is necessary for ordinary users. CI runs it for each PR.

## Future

Consider source-specific reviews, opt-in watched projects, public license audits,
and read-only PixelForge visualization after independent validation. Any future
download/install action must have its own threat model, legal review,
explicit operator approval, pinned hashes, and dedicated PR.
