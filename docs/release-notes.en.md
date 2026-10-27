# Changes in 0.5.0

Added persistent RU/DE/EN selection; translated views, help, dialogs and validation messages. Localized calendar and a three-language installer. Database schema and attachment format remain compatible with 0.4.2.

## Install and start

Run Kartoteka-0.5.0-Setup.exe and keep the suggested user folder. Administrator rights and Rust are not required. Save your work and close the old application before updating. For a portable launch, extract the entire ZIP; keep the EXE and three DLL files together.

## Data safety and moving computers

Data folder: %LOCALAPPDATA%\Popovantondev\Kartoteka. It contains program_data.db, assets, backups and preferences.json. Data is not encrypted. To move computers, create a backup and copy the entire data folder while the application is closed. Restore uses backups from the application list and creates a safety copy first. Uninstalling keeps your data.

## Requirements and limitations

Tested on Windows 11 x 64. The EXE is unsigned. The table shows up to 2,000 rows and search up to 500 results. Free-form layout editing and Microsoft Access import are not implemented. Windows DPI 125/150% and two monitors were not separately tested; font smoothing depends on the egui renderer. No telemetry; core features work offline.

## SHA-256

[SHA256SUMS.txt](https://github.com/popovantondev/Kartoteka/releases/download/v0.5.0/SHA256SUMS.txt)
