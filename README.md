<p align="center">
  <img src="assets/app-icon.png" width="160" height="160" alt="Music Tag Editor">
</p>

<h1 align="center">Music Tag Editor</h1>

<p align="center">
  Fix messy music tags without the headache.<br>
  Drop in a folder, edit like a spreadsheet, save, rename — done.
</p>

<p align="center">
  <a href="https://github.com/saint-james-fr/music-tag-editor/releases/latest"><img src="https://img.shields.io/github/v/release/saint-james-fr/music-tag-editor?style=flat-square&label=download&color=2ea44f" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/saint-james-fr/music-tag-editor?style=flat-square" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platforms">
  <img src="https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
</p>

<p align="center">
  <a href="https://github.com/saint-james-fr/music-tag-editor/releases/latest"><strong>Download</strong></a>
  · macOS · Windows · Linux
</p>

<p align="center">
  <img src="assets/screenshot.jpg" width="900" alt="Music Tag Editor screenshot">
</p>

Artwork, bulk edits, rename patterns, multiple tabs.  
Works with MP3, FLAC, M4A, WAV, and friends ([lofty](https://github.com/Serial-ATA/lofty-rs) · Tauri · Vue).

## Download

Grab the latest build from **[Releases](https://github.com/saint-james-fr/music-tag-editor/releases/latest)**.

Builds aren’t code-signed yet, so the OS may complain on first open:

- **macOS** — right-click → **Open**, or Privacy & Security → **Open Anyway**. Still stuck? `xattr -cr "/Applications/Music Tag Editor.app"`
- **Windows** — SmartScreen → **More info** → **Run anyway**
- **Linux** — `chmod +x` the AppImage, then run it

## Quick tips

- Yellow bar = unsaved changes
- Double-click a column header to fill down
- Rename with `{{tracknumber}}`, `{{title}}`, `{{artist}}`, `{{album}}`, …

## Develop

```bash
yarn install
yarn tauri dev
```

Build: `yarn tauri build` → installers in `src-tauri/target/release/bundle/`.  
Needs [Node](https://nodejs.org/) + yarn, [Rust](https://rustup.rs/), and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Ship a release: GitHub → **Actions** → **Release** → pick patch / minor / major.

## Contributing

Issues and PRs welcome — keep them small and focused. Bug reports with a short “what I did / what I expected” go a long way.

## License

[MIT](LICENSE) © 2026 Maxence Robinet
