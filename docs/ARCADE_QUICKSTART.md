# PhiCade Arcade Setup: cores, BIOS and one-click Play

PhiCade does not distribute game ROMs, console firmware, keys or third-party emulator binaries. Keep user-selected games and local core files on your PC; do not commit them to GitHub.

## Quick Play

1. Open the native Tauri PhiCade desktop application.
2. Select your ROM folder using **SELECT ROM DIRECTORY** (currently indexes the chosen folder, not recursive folders).
3. For each console family, download a suitable Windows x64 libretro core from the **official** buildbot, extract its `.dll`, and select **REGISTER LIBRETRO CORE** one time for each binary.
4. Or unzip several official Libretro core DLLs into a single folder and click **IMPORT ALL CORES FROM FOLDER**. This explicitly selected, nonrecursive import accepts at most 64 files named `*_libretro.dll` (and equivalent shared-library extensions), verifies each core independently and reports failures. No automatic downloads or permanently granted runtime authority.
5. Select a game in the list and press **PLAY GAME**. Or double-click the game row.
6. PhiCade chooses a registered core advertising that game's file extension, honors a manually selected matching core, and starts a single operator-approved session. The native backend verifies the core's SHA-256 and identity at every launch.
7. Press **F11** or **Alt+Enter** during gameplay for fullscreen arcade mode; press either again or **Esc** to return.

**If a matching core isn't registered**, the Play button shows a notice. GB/GBC can use the legacy SameBoy picker if no compatible registered core exists. The Play action does not download, install or authorize a core for future sessions.

## Recommended starting cores (Windows x64)

Official core downloads: https://buildbot.libretro.com/nightly/windows/x86_64/latest/

| PhiCade library category | Core to try | Archive to find | Firmware notes |
| --- | --- | --- | --- |
| NES | FCEUmm | `fceumm_libretro.dll.zip` | Ordinary NES cartridges generally do not need BIOS; Famicom Disk System does |
| SNES | Snes9x | `snes9x_libretro.dll.zip` | Ordinary SNES cartridges generally do not need external BIOS; special add-ons can |
| GB/GBC | SameBoy (qualified legacy path) or mGBA | `mgba_libretro.dll.zip` | Standard cartridge games generally do not need external BIOS |
| GBA | mGBA | `mgba_libretro.dll.zip` | External BIOS is generally optional with mGBA |
| Genesis / Mega Drive | Genesis Plus GX | `genesis_plus_gx_libretro.dll.zip` | Ordinary Genesis cartridge games generally do not need BIOS; Sega CD does |
| PlayStation 1 | Beetle PSX (mednafen) | `mednafen_psx_libretro.dll.zip` | Region-appropriate PS1 BIOS is typically required |

These are suggested candidates, **not** a guarantee that every libretro core works with PhiCade's current generic host. Some cores need hardware rendering or firmware, interfaces that PhiCade has not yet qualified. BIOS filenames and hashes must match the **specific** core's documentation.

## Where firmware belongs

PhiCade's native libretro host receives its Tauri application data **system** directory, normally on Windows:

`%APPDATA%\com.phicade.desktop\system`

Create it if needed; place only firmware you are authorized to use, with the exact filenames/folder structure required by the core.

Official BIOS documentation:
- https://docs.libretro.com/guides/bios/
- https://docs.libretro.com/library/bios/
- https://docs.libretro.com/guides/core-list/

**There is no legitimate universal commercial BIOS download.** Obtain permitted firmware from hardware you own where lawful, official provider downloads, or available open-source firmware alternatives when supported. Do not use random "all BIOS" bundles.

## Scope

PhiCade currently indexes `.nes`, `.sfc`, `.smc`, `.gb`, `.gbc`, `.gba`, `.md`, `.gen`, `.32x`, `.cue`, `.chd`, and `.pbp` in its library. Other classic systems like Atari, Amiga, Commodore 64, CoCo 3, arcade MAME, N64 and PS2 need additional catalog and/or emulator adapter work. Installing a libretro DLL does not by itself enable a new library category.

For PS1 cue/bin images, keep all referenced track files together; launch the `.cue` file (rather than a raw track `.bin`). For huge libraries, the current scan is limited to the chosen folder and up to 4096 entries.


## Old SNES collections without file extensions

Some classic SNES ROM collections (including old Snes9x libraries) store
cartridges as files named `Secret of Mana`, `Samurai Shodown` or `Super
Metroid` with **no extension**. Windows Explorer calls them simply **File**.
PhiCade now examines standard SNES internal cartridge headers (LoROM, HiROM,
including old 512-byte copier headers) to recognize these without renaming them.

- Select the directory containing the original SNES files, then click
  **RESCAN GAME LIBRARY**.
- Choose the **SNES** system tab to browse the detected cartridge titles.
- If nonstandard or hacked extensionless dumps are still missing, switch
  **SNES NO-EXT FILES** from **AUTO-DETECT** to **INCLUDE LEGACY DUMPS**.
  This explicit fallback includes files with plausible SNES cartridge
  sizes even if the SNES internal header cannot be verified.
- PhiCade never edits or renames these games. A detected legacy ROM gets an
  internal `.sfc` *routing hint* so the registered Snes9x Libretro core can
  be automatically selected; the core receives the actual original path.
- Files with other extensions are **not** treated as ROMs merely because they
  appear in this folder. `.srm` saves, `.txt` lists, and unrelated files
  stay outside the library. The fallback is disabled by default, and should
  only be used when scanning an SNES ROM directory.

This is a content-based best-effort scanner, not a ROM identity database.
Some badly corrupted images may still fail to start; the source bytes are
preserved for troubleshooting.

## Nintendo 64: Star Fox 64 and other 3D games

**Current status:** The PhiCade native Libretro host does not implement the
OpenGL/Vulkan hardware-rendering context that most modern N64 cores require.
Simply registering `mupen64plus_next_libretro.dll` in the normal runtime
registry is **not** sufficient for native N64 playback.

To get N64 games launched **from PhiCade** today, the desktop uses an explicit
RetroArch external-window bridge:

1. Install the Windows x64 version of RetroArch from
   https://www.retroarch.com/platforms.html
2. Get the Windows x64 Mupen64Plus-Next Libretro core from
   https://buildbot.libretro.com/nightly/windows/x86_64/latest/mupen64plus_next_libretro.dll.zip
   and extract the `mupen64plus_next_libretro.dll` file somewhere permanent.
   The alternative `parallel_n64_libretro.dll` is also supported as a user-
   selected bridge core, but it is not the default recommendation.
3. Put legally held N64 ROM files (`.z64`, `.n64`, `.v64`) in the folder you
   scan with PhiCade. Zip archives are not currently included in the N64 index.
4. In PhiCade under **N64 // 3D EXTERNAL BRIDGE**, select your
   `retroarch.exe` and the extracted N64 core DLL. These paths are saved.
5. Click the **N64** tab, select Star Fox 64 or another game, and press
   **PLAY GAME** (or double-click). The user gesture invokes:
   `retroarch.exe -L <selected-N64-core.dll> <selected-game.z64>`
   using argument-array execution **without shell expansion**.
6. RetroArch opens its own game window. Controller binding, fullscreen,
   graphics settings and saves are handled by RetroArch, not PhiCade's
   in-app CRT / F11 mode. RetroArch can be configured with an Xbox controller.

**Limitations:** This is a bridge, **not yet** 3D video embedded in PhiCade.
The selected RetroArch executable must be named `retroarch.exe`; the
core DLL must be the N64 Mupen64Plus-Next or ParaLLEl core. The Rust host
canonicalizes these paths, only allows N64 file extensions, hashes the core
and ROM, uses an argv-based process launch, and refuses a second concurrent
N64 bridge session. It never bundles a third-party executable, firmware,
or a game ROM.

**N64 games generally do not require a BIOS.** Do not download random BIOS
bundles. Most cartridge games, including Star Fox 64, can start without
proprietary console firmware. For controller configuration use RetroArch's
official documentation: https://docs.libretro.com/guides/input-joypad-drivers/

**Next engineering milestone:** Real in-PhiCade 3D rendering would require a
supported GL/Vulkan context and N64-specific controls plus a rendering test
suite; this launch bridge is a usable incremental step.


## N64 CRT overlay fix (Windows / Vulkan)

The first experimental N64 docking release used Win32 `SetParent` to make
RetroArch a child of PhiCade's native top-level window. On Windows with
WebView2, this could cause the game to vanish behind the WebView (leaving
only an empty CRT) even though RetroArch was still running and Undock
restored gameplay.

The revised **DOCK N64 TO CRT** action keeps RetroArch's Vulkan game window
**top-level**. It assigns the PhiCade window as its Win32 **owner** (not its
parent), removes its borders, and positions the real game surface over the
CRT with `ClientToScreen` and `SetWindowPos`. An owned window stays above
PhiCade without global always-on-top behavior. It follows PhiCade window
moves/resizes and the 4:3 fullscreen layout. **UNDOCK N64** restores the
original owner, styles, z-order and location without reloading the game.

This approach is **visually integrated**, not pixel capture, streaming or
a reparented native child. HTML overlays cannot paint over the N64 image.
As before, some Vulkan configurations may resist being resized or styled
while running; the fallback is always **UNDOCK N64**. Undock remains visible
in PhiCade fullscreen mode as an escape hatch. Windows compile/unit tests
do not by themselves prove that a real RetroArch Vulkan swapchain displays
correctly; this still needs a Star Fox 64 test.

### First recovery test

1. Exit any running RetroArch game normally; close PhiCade.
2. Restart the feature branch with `npm run desktop`.
3. Set RetroArch to windowed mode, launch Star Fox 64 from PhiCade and
   let auto-docking attempt the overlay.
4. If the game is still absent, click **UNDOCK N64**, verify external game
   video remains playable, and report the precise PhiCade notice bar.
   Do not force-close RetroArch or delete its configuration.

## Experimental N64 docking inside PhiCade (Windows)

**New feature:** After the N64 external bridge launches an N64 ROM, PhiCade
attempts to dock that *existing* RetroArch game window over its CRT panel.
RetroArch still does the Vulkan/OpenGL drawing, audio, saves and controller
handling; PhiCade supplies the launcher, frame, library and surrounding UI.
This is **Win32 window reparenting**, not a new N64 renderer or a streaming
codec.

- Start Star Fox 64 using the same N64 **PLAY GAME** button. PhiCade
  automatically tries to find the RetroArch game window and attach it.
- If successful, the game appears in the CRT area. Resizing the app,
  scrolling, and clicking PhiCade's **FULLSCREEN (F11)** button resize the
  docked game area (4:3 layout).
- **UNDOCK N64** restores the external RetroArch window without stopping it.
  **DOCK N64 TO CRT** re-attaches it if you want to try again.
- **CLOSE N64** sends RetroArch a normal Win32 WM_CLOSE message rather than
  forcibly killing it, allowing ordinary save shutdown.
- If Windows denies reparenting or GPU rendering fails, **undock** and keep
  using the normal external N64 window. Closing/reopening PhiCade also
  restores the game to an independent window when possible.

**First test tip:** Start RetroArch in its normal **windowed** mode, not
exclusive fullscreen, before launching from PhiCade. Windows needs to resize
the RetroArch client window to the CRT's rectangle. If the window remains
fullscreen or Vulkan refuses resizing, use **UNDOCK N64** and continue playing
externally.

**Known limitations:**
1. Windows 11 only for now. Win32 SetParent can affect DPI-awareness
   compatibility between the two processes, especially when display
   scaling differs; consult
   https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent
2. Some Vulkan/GL swapchains do not react well to reparenting. This is a
   prototype that requires hands-on Star Fox 64 testing on the target PC.
3. When RetroArch has keyboard focus, PhiCade's JavaScript F11 shortcut may
   not receive key events. Use the visible **FULLSCREEN (F11)** button in the
   PhiCade UI (click outside the game first), or undock and use RetroArch's
   own fullscreen controls. Gamepad input remains owned by RetroArch.
4. HTML scanlines and other overlays cannot render on top of the native child
   window; the CRT surface itself can still be framed by HTML/CSS.
5. N64 rendering and video/audio are **not** available through PhiCade's
   internal FramePacket or agent observation APIs with this bridge.
