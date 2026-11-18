# PhiCade Arcade Setup: cores, BIOS and one-click Play

PhiCade does not distribute game ROMs, console firmware, keys or third-party emulator binaries. Keep user-selected games and local core files on your PC; do not commit them to GitHub.

## Quick Play

1. Open the native Tauri PhiCade desktop application.
2. Select your ROM folder using **SELECT ROM DIRECTORY** (currently indexes the chosen folder, not recursive folders).
3. For each console family, download a suitable Windows x64 libretro core from the **official** buildbot, extract its `.dll`, and select **REGISTER LIBRETRO CORE** one time for each binary.
4. Select a game in the list and press **PLAY GAME**. Or double-click the game row.
5. PhiCade chooses a registered core advertising that game's file extension, honors a manually selected matching core, and starts a single operator-approved session. The native backend verifies the core's SHA-256 and identity at every launch.
6. Press **F11** or **Alt+Enter** during gameplay for fullscreen arcade mode; press either again or **Esc** to return.

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
