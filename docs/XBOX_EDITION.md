# PhiCade Xbox Edition — Rung 59 / Browser Gateway v0.1

This milestone adds a **separate, static, controller-oriented browser launcher** under
`xbox/` in the existing PhiCade repository. It does **not** install PhiOS as the
Xbox operating system, port Tauri/Rust to Xbox, package a UWP application,
or create an emulation/AI streaming server.

## Why the separate entry point?

The existing desktop UI is a Tauri application with native Rust/libretro APIs,
filesystem access, and local Ollama integration. None of those native capabilities
are available to an ordinary Xbox Microsoft Edge page. The browser client only
navigates to an audited, hardcoded set of HTTPS destinations.

This preserves PhiCade's governing principle:

```
SELECT PORTAL (human) -> open HTTPS URL (browser)
not
GAMEPAD -> privileged emulator/system/agent action
```

There is no backend, secret, auth token, remote-desktop service, file access,
session authority, or use of the PhiCade Action Bus in this preview.

## First launch (Xbox One X, no Developer Mode required)

1. On Xbox in **Retail Mode**, launch **Microsoft Edge**.
2. Once the Pages workflow has succeeded and the repository's
   **Settings → Pages → Build and deployment → Source** is set to
   **GitHub Actions**, browse to
   **https://michaelwave369.github.io/PhiCade/**.
3. Try D-pad / left stick to select a portal, **A** to open it,
   **X** to show controls, **B** to close the controls panel.
4. If the browser captures controller buttons or does not expose the
   **Gamepad API**, use whatever pointer/keyboard navigation Edge makes
   available. Gamepad support is browser/hardware-dependent, not assumed.
5. Once a portal is opened, use the browser's back/navigation controls
   to return. Controller support on **destination sites** is independent.

On desktop, use arrow keys, Enter, Escape, Tab, touch, or mouse. Links still
work even if Gamepad API access is unavailable.

## Local test (no install)

From the repository root:

```bash
node --test xbox/tests/*.test.mjs
node --check xbox/app.mjs
node --check xbox/navigation.mjs
python -m http.server 8000 --directory xbox
```

Visit **http://localhost:8000/** from a desktop browser for local UI
testing. Use a server, not `file://`, because the launcher imports an ES module.

The dedicated workflow verifies navigation logic and deploys **only**
`index.html`, `styles.css`, `app.mjs`, and `navigation.mjs`.
A PR will run tests without deploying; a merge into main will attempt Pages
deployment. Pages requires its initial repository configuration.

## Acceptance checklist

- [ ] CI navigation tests and JavaScript syntax pass
- [ ] GitHub Pages build and deployment succeed
- [ ] Xbox One X Edge loads the actual page
- [ ] Card text and status panel are legible from couch distance
- [ ] Browser reports controller through Gamepad API (or records unsupported)
- [ ] D-pad navigation and thumbstick navigation work without unintended repeats
- [ ] A selects a link; X shows help; B closes help
- [ ] Mouse/keyboard fallback works on desktop
- [ ] External browser navigation and Back behavior are documented on-console
- [ ] No native runtime, ROM access, Ollama, or PhiOS authority is represented as working

**Hardware qualification status: NOT TESTED.** Passing CI is not evidence
that Xbox Edge emits Gamepad API events, handles A/B button actions correctly,
or retains focus after a cross-site navigation.

## Roadmap

**R59 / now:** browser launcher, curated HTTPS portals, keyboard/gamepad
navigation, honest status panel, tests, opt-in Pages deployment.

**R60 candidate:** console-tested focus/input acceptance; orientation,
performance, virtual pointer or remote-compatible input fallback if required.
Do not advertise controller support until measured on Xbox One X.

**Future candidate (separate security review):** authenticated **read-only**
PhiOS/PhiCade service status, then explicit least-privilege remote session
proposals with origin validation, secure transport, user consent, rate limiting,
and receipts. Never expose local Ollama or desktop control ports directly to
the public web. Browser access to a private LAN endpoint may be limited by
HTTPS mixed-content and CORS rules.

**Separate potential work:** UWP packaging in Xbox Developer Mode. This is
not part of R59, and Developer Mode is not required for the browser route.

## External platform reference

- Microsoft UWP Xbox FAQ:
  https://learn.microsoft.com/en-us/windows/uwp/xbox-apps/frequently-asked-questions
- Microsoft Xbox One Developer Mode:
  https://learn.microsoft.com/en-us/windows/uwp/xbox-apps/devkit-activation
