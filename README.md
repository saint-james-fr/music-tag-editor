# <img src="public/app-icon-64.png" width="48" height="48" alt="" align="left" /> Tag Editor

&nbsp;

Desktop batch music tag editor. Open a folder, edit tags in a spreadsheet grid, save, and rename from a pattern.

Built with **Tauri 2**, **Vue 3**, and [lofty](https://github.com/Serial-ATA/lofty-rs) (MP3, FLAC, M4A, WAV, AIFF, OGG, Opus, and more).

![Tag Editor screenshot](assets/screenshot.jpg)

## Run

```bash
yarn install
yarn tauri dev
```

## Usage

| Action | How |
|--------|-----|
| **Open** | Folder → all audio files recursively |
| **Import** | Add more files to the current list |
| **Edit** | Click cells; yellow bar = unsaved |
| **Bulk** | Check rows, or double-click a column header to copy the first visible value down |
| **Save** | Writes tags for dirty rows |
| **Rename** | Set pattern → **Preview** → **Rename** |

Rename tokens: `{{tracknumber}}`, `{{title}}`, `{{artist}}`, `{{album}}`, `{{genre}}`, `{{year}}`, `{{albumartist}}`, `{{filename}}`.

## Features

- Artwork column + popover (replace, delete, export) — **Save** writes embedded art
- Tabs — multiple lists per window; session restores paths
- Resizable columns that fill the window width
- Prefs — rename pattern, track padding, confirmations
