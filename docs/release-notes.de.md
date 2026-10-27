# Änderungen in 0.5.0

RU/DE/EN mit gespeicherter Sprachwahl; übersetzte Ansichten, Hilfe, Dialoge und Validierungsmeldungen. Lokalisierter Kalender und dreisprachiges Setup. Datenbankschema und Anhangformat bleiben mit 0.4.2 kompatibel.

## Installation und Start

Starten Sie Kartoteka-0.5.0-Setup.exe und behalten Sie den vorgeschlagenen Benutzerordner. Administratorrechte und Rust sind nicht erforderlich. Speichern und schließen Sie die alte Anwendung vor einem Update. Ohne Installation: das gesamte portable ZIP entpacken; EXE und drei DLL-Dateien müssen zusammenbleiben.

## Datensicherung und Umzug

Datenordner: %LOCALAPPDATA%\Popovantondev\Kartoteka. Er enthält program_data.db, assets, backups und preferences.json. Daten sind unverschlüsselt. Zum Umzug zuerst sichern und bei geschlossener Anwendung den gesamten Datenordner auf den anderen Computer kopieren. Wiederherstellung verwendet Sicherungen aus der Programmliste und erstellt vorher eine Schutzkopie. Die Deinstallation erhält Ihre Daten.

## Voraussetzungen und Grenzen

Auf Windows 11 x 64 geprüft. Die EXE ist nicht signiert. Die Tabelle zeigt bis zu 2000 Zeilen, die Suche bis zu 500 Treffer. Freie Formularanordnung und Microsoft-Access-Import sind nicht implementiert. Windows-DPI 125/150% und zwei Monitore sind nicht separat geprüft; die Schriftglättung hängt vom egui-Renderer ab. Keine Telemetrie; Kernfunktionen arbeiten offline.

## SHA-256

[SHA256SUMS.txt](https://github.com/popovantondev/Kartoteka/releases/download/v0.5.0/SHA256SUMS.txt)
