# EmulatorJS Core (Browser)

BBKEmu runs in the browser as a standard [EmulatorJS](https://emulatorjs.org)
core. The build follows EmulatorJS's own pipeline: the Rust libretro core is
compiled for `wasm32-unknown-emscripten`, linked against the
`EmulatorJS/RetroArch` fork, and packaged as the `<core>-wasm.data` bundles that
an EmulatorJS page loads at runtime.

The project site ships a ready-to-use player at
<https://AloysHF.github.io/BBKEmu/play.html>: visitors pick a `.gam` game file
and a firmware ZIP from their own machine; both are read through blob URLs, so
nothing is uploaded and the site does not host any game data.

## What you get

Every nightly publishes a single asset:

| File | Contents |
|------|----------|
| `bbkemu-emulatorjs.zip` | Both core variants, the build report `cores/reports/bbkemu.json`, `core.json`, `build.json`, `license.txt` and a README |

The two `.data` files stay separate inside the ZIP: EmulatorJS requests
`bbkemu-wasm.data` or `bbkemu-legacy-wasm.data` by name depending on WebGL2
support, so one bundle cannot serve both cases.

The bundle is laid out like this:

```
bbkemu-emulatorjs.zip
├── cores/
│   ├── bbkemu-wasm.data
│   ├── bbkemu-legacy-wasm.data
│   └── reports/bbkemu.json
├── core.json
├── build.json
├── license.txt
└── README.md
```

## Install into a site

EmulatorJS always fetches cores from `<EJS_pathtodata>cores/`, so host the
runtime yourself (or use the pinned CDN files) and drop the core beside it:

```
data/
├── loader.js
├── emulator.min.js
├── emulator.min.css
├── compression/
│   ├── extract7z.js       # extracts the core .data archive
│   └── extractzip.js      # extracts ZIP content
└── cores/
    ├── bbkemu-wasm.data
    ├── bbkemu-legacy-wasm.data
    └── reports/bbkemu.json
```

`emulator.min.zip` only contains `emulator.min.js` and `emulator.min.css`; the
`compression/` files are loaded on demand from the same EmulatorJS data
directory. Without `extract7z.js` the core download ends in a "Network Error".

To upgrade the runtime, replace the runtime files with a newer EmulatorJS
release (≥ 4.2.3) and keep this `cores/` directory.

## Use

```html
<div id="game"></div>
<script>
  EJS_player = "#game";
  EJS_core = "bbkemu";
  EJS_gameUrl = URL.createObjectURL(gameFile);
  EJS_gameName = gameFile.name;
  EJS_biosUrl = URL.createObjectURL(firmwareZip);
  EJS_pathtodata = "data/";
  EJS_startOnLoaded = false;
</script>
<script src="data/loader.js"></script>
```

`site/play.html` in this repository is a complete example.

### Firmware

BBKEmu needs the font ROM (`8.BIN`) and OS ROM (`E.BIN`) dumped from a physical
BBK dictionary. Desktop frontends read them from
`<system>/BBKEmu/A4980/`, but a browser page cannot write to a RetroArch system
directory. Set `EJS_biosUrl` to a ZIP that contains both files at its root:

```
bbk-firmware.zip
├── 8.BIN
└── E.BIN
```

EmulatorJS extracts the archive beside the content file inside the wasm file
system, which is where the core looks for the firmware on Emscripten. The
files are never uploaded; the browser reads them locally.

### Content

A bare `.gam` file loads on its own. If a game needs companion files, zip it
with them so the frontend sees a single file. The core receives the game data
in memory, so no path handling is needed.

## Build locally

```bash
# 1. Build the staticlib with a nightly toolchain (rebuild std with panic=abort
#    so Binaryen's Asyncify pass does not abort on wasm exception handling).
cargo build -p bbkemu-libretro --target wasm32-unknown-emscripten \
  --profile wasm-release -Zbuild-std=std,panic_abort

# 2. Link it into the EmulatorJS/RetroArch fork and package the bundles.
scripts/build-emulatorjs-core.sh --work-dir /tmp/emulatorjs-build
```

Requirements: an activated emsdk (`emmake`, `emcc`), `7z`, `zip` and `git`.
The output bundle is written to `out/bbkemu-emulatorjs.zip`.

## Notes and limitations

- The browser build renders the 159×96 LCD through libretro's software path.
  EmulatorJS scales it, so the LCD theme and swap options still apply.
- Core options (LCD color, orientation, CPU/timer rate, key repeat) are
  available through the EmulatorJS core-options menu.
- Frontend save states use `retro_serialize`, so they stay in browser storage
  and are not written back to a file.
