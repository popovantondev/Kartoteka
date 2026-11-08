<!-- ppk-readme:head:start -->
<a href="https://popovantondev.github.io/Kartoteka/index-en.html"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/header-en-dark.596af9a4.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/header-en-light.bf595f73.svg" width="880" height="188" alt="Kartoteka — Contacts, collections and lists on your own computer. Windows 11 · x64, Release 0.5.0"></picture></a>
<!-- ppk-readme:head:end -->

# Kartoteka

[Русский](../README.ru.md) · [Deutsch](../README.md) · **English**

<!-- ppk-readme:body:start -->
<p>
<a href="https://github.com/popovantondev/Kartoteka/releases/tag/v0.5.0"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-download-en-dark.e539ee75.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-download-en-light.0b21c6cc.svg" width="136" height="36" alt="Download"></picture></a>
<a href="https://popovantondev.github.io/Kartoteka/index-en.html"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-website-en-dark.0a4a827a.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-website-en-light.df215278.svg" width="120" height="36" alt="Website"></picture></a>
<a href="https://popovantondev.github.io/Kartoteka/Guide-en.html"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-guide-en-dark.aa8dc4fd.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-guide-en-light.66e4fc46.svg" width="141" height="36" alt="User guide"></picture></a>
<a href="https://popovantondev.github.io/Kartoteka/index-en.html#demo"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-demo-en-dark.8b547d19.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-demo-en-light.24c940ba.svg" width="171" height="36" alt="Try in browser"></picture></a>
<a href="https://github.com/popovantondev/Kartoteka/issues/new"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-bug-en-dark.fa7f7e14.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/btn-bug-en-light.dfda645f.svg" width="196" height="36" alt="Report a problem"></picture></a>
</p>

<p><a href="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/assets/app-en.png"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/preview-en-dark.3578c37e.webp"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/preview-en-light.d8009063.webp" width="880" height="516" alt="Kartoteka interface"></picture></a><br>
<sub><a href="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/assets/app-en.png">Full screenshot</a></sub></p>

<picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/divider-dark.2ce65351.svg"><img src="https://raw.githubusercontent.com/popovantondev/Kartoteka/main/docs/readme/divider-light.09467238.svg" width="880" height="6" alt="Divider"></picture>
<!-- ppk-readme:body:end -->

Keep contacts, collections and other lists on your own computer. Cards, tables, custom fields, photos, documents, search and backups in one application.

**Your local card index for Windows · 0.5.0 · Windows 11 x64**

Actual 0.5.0 interface. Sample records, no personal data.

## Features

- Record cards and a table with resizable columns.
- Custom tables and 11 field types: short and long text, integers and decimals, date, time, date/time, checkbox, URL, photo and document.
- Photo previews, managed local document copies and website links.
- Search across the selected table, loaded-row filtering and CSV export.
- Back up the database and attachments; a safety copy is made before restoring.
- Light theme, 80–150% scaling, collapsible menu and RU/DE/EN interface.

## Install and start

Run Kartoteka-0.5.0-Setup.exe and keep the suggested user folder. Administrator rights and Rust are not required. Save your work and close the old application before updating. For a portable launch, extract the entire ZIP; keep the EXE and three DLL files together.

## Choose a language

Open Settings → General or Appearance → Interface language and choose Русский, Deutsch or English. The choice survives restarts. Stored names and values are not rewritten; standard labels appear in the selected language. System file dialogs use the Windows language.

## Your first record

Click “New record”, fill in the required last and first names, attach a photo or document and click “Save”. The bottom buttons navigate between cards. Double-click a table row to open its card. Leaving an edited card prompts you to save it.

## Tables and fields

Create a table in Designer and add fields with a name, type, required flag and default value. Use arrows to reorder fields and drag header boundaries to resize columns. Forms controls visibility and shows a preview. Deleting a field requires confirmation and a safety copy.

## Photos and documents

The selected file is copied to the managed attachments folder. Save the record to attach it. Photos: JPG/JPEG, PNG, BMP, WEBP, up to 25 MB and 40 megapixels. Other documents: up to 250 MB. Documents open in the system application; HTTP/HTTPS links open in your browser.

## Search programs

Search

## Data safety and moving computers

Data folder: %LOCALAPPDATA%\Popovantondev\Kartoteka. It contains program_data.db, assets, backups and preferences.json. Data is not encrypted. To move computers, create a backup and copy the entire data folder while the application is closed. Restore uses backups from the application list and creates a safety copy first. Uninstalling keeps your data.

## Keyboard shortcuts

Ctrl+S saves; Ctrl+N creates a record; Ctrl+F opens search. Drag or double-click the menu divider to change its mode. The menu collapses automatically in narrow windows.

## Requirements and limitations

Tested on Windows 11 x64. The EXE is unsigned. The table shows up to 2,000 rows and search up to 500 results. Free-form layout editing and Microsoft Access import are not implemented. Windows DPI 125/150% and two monitors were not separately tested; font smoothing depends on the egui renderer. No telemetry; core features work offline.

## Build from source

Requires Rust stable-x86_64-pc-windows-gnullvm, rustfmt and LLVM-MinGW. scripts/build-noadmin.ps1 finds LLVM-MinGW under %LOCALAPPDATA%\Popovantondev\KartotekaTools, runs fmt/check/test/release and produces dist with the EXE and DLLs. Setup: scripts/build-setup.ps1 -NsisPath <path to makensis.exe>, using NSIS 3.11. Cargo.lock pins dependencies. Dependency licenses are unchanged; rights to the project code: see RIGHTS.md.

## Changes in 0.5.0

Added persistent RU/DE/EN selection; translated views, help, dialogs and validation messages. Localized calendar and a three-language installer. Database schema and attachment format remain compatible with 0.4.2.

## Verification

Build, language, data-preservation and installer checks are documented in RELEASE-CHECKS.md. Release file hashes are in SHA256SUMS.txt. Documentation checks do not replace running the app on your hardware.

## Rights and use

The source code is public for viewing; this is not an open-source licence. An unmodified release may be downloaded and run for personal use. Details: RIGHTS.md; rights to third-party components: third-party/README.md.

[Report an issue](https://github.com/popovantondev/Kartoteka/issues/new) · [Source code](https://github.com/popovantondev/Kartoteka)
