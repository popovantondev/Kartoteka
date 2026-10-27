//! Локальные фотографии, документы и резервные копии.
//! Привязанные файлы копируются внутрь каталога данных программы.

use crate::db::{Database, DbResult, FieldKind};
use chrono::Local;
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

pub fn default_data_dir() -> PathBuf {
    if let Some(base) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(base).join("Popovantondev").join("Kartoteka");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("kartoteka_data")
}

pub fn import_file(db: &Database, source: &Path, kind: FieldKind) -> DbResult<String> {
    if !source.is_file() {
        return Err(crate::i18n::tr("Выбранный файл не существует").into());
    }
    let metadata = fs::metadata(source).map_err(|e| e.to_string())?;
    if metadata.len() > 250 * 1024 * 1024 {
        return Err(crate::i18n::tr("Максимальный размер файла — 250 МБ").into());
    }
    let extension = source
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if kind == FieldKind::Image {
        if !["jpg", "jpeg", "png", "bmp", "webp"].contains(&extension.as_str()) {
            return Err(crate::i18n::tr("Для фото выберите JPG, PNG, BMP или WEBP").into());
        }
        if metadata.len() > 25 * 1024 * 1024 {
            return Err(crate::i18n::tr("Фото не должно превышать 25 МБ").into());
        }
        // Маленький сжатый файл способен распаковаться в сотни мегапикселей.
        // Проверяем размеры ДО декодирования, чтобы не исчерпать память.
        let (width, height) = image::image_dimensions(source)
            .map_err(|_| crate::i18n::tr("Не удалось прочитать размеры фотографии").to_string())?;
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 40_000_000 {
            return Err(
                crate::i18n::tr("Слишком большое изображение: максимум 40 мегапикселей").into(),
            );
        }
        image::ImageReader::open(source)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|_| crate::i18n::tr("Не удалось прочитать изображение").to_string())?;
    }
    let folder = if kind == FieldKind::Image {
        "images"
    } else {
        "documents"
    };
    let relative = format!(
        "assets/{folder}/{}.{}",
        Uuid::new_v4(),
        if extension.is_empty() {
            "bin"
        } else {
            &extension
        }
    );
    let target = db.root.join(&relative);
    fs::create_dir_all(
        target
            .parent()
            .ok_or(crate::i18n::tr("Некорректная папка"))?,
    )
    .map_err(|e| e.to_string())?;
    fs::copy(source, &target).map_err(|e| e.to_string())?;
    Ok(relative)
}

pub fn resolve_attachment(db: &Database, relative: &str) -> DbResult<PathBuf> {
    let path = Path::new(relative);
    if !relative.starts_with("assets/")
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(crate::i18n::tr("Некорректная ссылка на локальный файл").into());
    }
    let full = db.root.join(path);
    let canonical_root = db.root.canonicalize().map_err(|e| e.to_string())?;
    let canonical_file = full
        .canonicalize()
        .map_err(|_| crate::i18n::tr("Файл не найден. Возможно, он удалён.").to_string())?;
    if !canonical_file.starts_with(canonical_root) {
        return Err(crate::i18n::tr("Файл находится вне папки программы").into());
    }
    Ok(canonical_file)
}

pub fn photo_texture(
    ctx: &eframe::egui::Context,
    db: &Database,
    rel: &str,
) -> DbResult<eframe::egui::TextureHandle> {
    let path = resolve_attachment(db, rel)?;
    let original = image::open(path).map_err(|e| e.to_string())?;
    let thumb = original.thumbnail(900, 1100).to_rgba8();
    let dims = [thumb.width() as usize, thumb.height() as usize];
    let image = eframe::egui::ColorImage::from_rgba_unmultiplied(dims, thumb.as_raw());
    Ok(ctx.load_texture(
        format!("photo-{rel}"),
        image,
        eframe::egui::TextureOptions::LINEAR,
    ))
}

fn copy_tree(from: &Path, to: &Path) -> DbResult<()> {
    if !from.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for item in fs::read_dir(from).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let kind = item.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(crate::i18n::tr("Символические ссылки в вложениях не разрешены").into());
        }
        let target = to.join(item.file_name());
        if kind.is_dir() {
            copy_tree(&item.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(item.path(), target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Проверяет, что все ссылки на фото и документы в выбранном снимке
/// указывают на реальные файлы ВНУТРИ этого же снимка.
/// Иначе копия БД существует, но восстановить её без потери вложений нельзя.
fn verify_snapshot(snapshot: &Path) -> DbResult<()> {
    let source = snapshot.join("program_data.db");
    Database::quick_check(&source)?;
    let old =
        rusqlite::Connection::open_with_flags(&source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
    let mut stmt=old.prepare("SELECT v.value FROM ui_values v JOIN ui_fields f ON f.id=v.field_id WHERE f.kind IN ('image','file') AND v.value<>''")
        .map_err(|e|e.to_string())?;
    let values = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let safe_base = snapshot.canonicalize().map_err(|e| e.to_string())?;
    for relative in values {
        let rel = Path::new(&relative);
        if !relative.starts_with("assets/")
            || rel.components().any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(match crate::i18n::current() {
                crate::i18n::Language::Ru => {
                    format!("В копии указан недопустимый путь: {relative}")
                }
                crate::i18n::Language::De => {
                    format!("Ungültiger Pfad in der Sicherung: {relative}")
                }
                crate::i18n::Language::En => format!("Invalid path in backup: {relative}"),
            });
        }
        let file = snapshot
            .join(rel)
            .canonicalize()
            .map_err(|_| match crate::i18n::current() {
                crate::i18n::Language::Ru => format!("В копии отсутствует файл: {relative}"),
                crate::i18n::Language::De => format!("Datei fehlt in der Sicherung: {relative}"),
                crate::i18n::Language::En => format!("File missing from backup: {relative}"),
            })?;
        if !file.starts_with(&safe_base) || !file.is_file() {
            return Err(match crate::i18n::current() {
                crate::i18n::Language::Ru => {
                    format!("Файл копии находится вне папки данных: {relative}")
                }
                crate::i18n::Language::De => {
                    format!("Sicherungsdatei liegt außerhalb des Datenordners: {relative}")
                }
                crate::i18n::Language::En => {
                    format!("Backup file is outside the data folder: {relative}")
                }
            });
        }
    }
    Ok(())
}

/// Создание снимка. Пользовательская копия должна быть полностью проверена;
/// защитный снимок перед восстановлением может быть неполным (например,
/// некоторые текущие фото уже удалены). В этом случае пишем WARNING.txt,
/// иначе восстановить здоровую копию поверх повреждённого состояния нельзя.
fn create_snapshot(db: &Database, name: &str, strict: bool) -> DbResult<PathBuf> {
    let backups = db.root.join("backups");
    fs::create_dir_all(&backups).map_err(|e| e.to_string())?;
    let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut target = backups.join(format!("{name}_{stamp}"));
    let mut num = 2;
    while target.exists() {
        target = backups.join(format!("{name}_{stamp}_{num}"));
        num += 1;
    }
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    if let Err(e) = db
        .backup_database(&target.join("program_data.db"))
        .and_then(|_| copy_tree(&db.root.join("assets"), &target.join("assets")))
    {
        let _ = fs::remove_dir_all(&target);
        return Err(e);
    }
    if let Err(e) = verify_snapshot(&target) {
        if strict {
            let _ = fs::remove_dir_all(&target);
            return Err(e);
        }
        let warning=match crate::i18n::current(){crate::i18n::Language::Ru=>format!("ВНИМАНИЕ: защитная копия сделана перед восстановлением, но она НЕПОЛНАЯ.\nПроблема: {e}\nЭту копию нельзя считать проверенной.\n"),crate::i18n::Language::De=>format!("WARNUNG: Die Schutzkopie vor der Wiederherstellung ist UNVOLLSTÄNDIG.\nProblem: {e}\nDiese Kopie ist nicht als geprüft anzusehen.\n"),crate::i18n::Language::En=>format!("WARNING: The safety copy made before restoring is INCOMPLETE.\nProblem: {e}\nThis copy must not be considered verified.\n")};
        if let Err(write_error) = fs::write(target.join("WARNING.txt"), warning) {
            let _ = fs::remove_dir_all(&target);
            return Err(write_error.to_string());
        }
    }
    Ok(target)
}

pub fn create_backup(db: &Database) -> DbResult<PathBuf> {
    create_snapshot(db, crate::i18n::tr("Копия"), true)
}

pub fn list_backups(db: &Database) -> DbResult<Vec<PathBuf>> {
    let backups = db.root.join("backups");
    if !backups.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = fs::read_dir(backups)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.is_dir() && p.join("program_data.db").is_file())
        .collect::<Vec<_>>();
    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    Ok(dirs)
}

pub fn restore_backup(db: &Database, selected: &Path) -> DbResult<()> {
    let canonical = selected.canonicalize().map_err(|e| e.to_string())?;
    let base = db
        .root
        .join("backups")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !canonical.starts_with(&base) || canonical == base {
        return Err(crate::i18n::tr("Копия должна быть из списка приложения").into());
    }
    verify_snapshot(&canonical)?;
    let source = canonical.join("program_data.db");
    // Перед восстановлением сохраняем ВСЁ, что ещё доступно. Если в текущей
    // базе уже отсутствует вложение, защитная копия отмечается WARNING.txt,
    // но не блокирует восстановление из исправной копии.
    create_snapshot(db, crate::i18n::tr("Перед_восстановлением"), false)?;
    let stage = db.root.join("program_data.restore.tmp");
    fs::copy(&source, &stage).map_err(|e| e.to_string())?;
    Database::quick_check(&stage)?;
    // Сначала возвращаем все вложения. Старые файлы не удаляем: так не теряются
    // пользовательские данные, не попавшие в восстанавливаемую копию.
    if let Err(e) = copy_tree(&canonical.join("assets"), &db.root.join("assets")) {
        let _ = fs::remove_file(&stage);
        return Err(e);
    }
    // Windows разрешает замену файла только когда нет открытых соединений SQLite.
    let old = db
        .root
        .join(format!("program_data.before_restore.{}.db", Uuid::new_v4()));
    fs::rename(&db.path, &old).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&stage, &db.path) {
        let _ = fs::rename(&old, &db.path);
        return Err(match crate::i18n::current() {
            crate::i18n::Language::Ru => format!("Восстановление отменено: {e}"),
            crate::i18n::Language::De => format!("Wiederherstellung abgebrochen: {e}"),
            crate::i18n::Language::En => format!("Restore cancelled: {e}"),
        });
    }
    let _ = fs::remove_file(&old);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backup_checks_attached_documents_and_restores_values() {
        use crate::db::Record;
        let root = std::env::temp_dir().join(format!("kartoteka-backup-test-{}", Uuid::new_v4()));
        let db = Database::new(root.clone()).unwrap();
        let tid = db.tables().unwrap()[0].id;
        let fields = db.fields(tid).unwrap();
        let mut record = Record::new(tid, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                record.values.insert(f.id, "Тест".into());
            }
            if f.name == "Имя" {
                record.values.insert(f.id, "Ира".into());
            }
        }
        let id = db.save_record(&record).unwrap();
        let source = root.join("sample.txt");
        fs::write(&source, b"Example document").unwrap();
        let rel = import_file(&db, &source, FieldKind::File).unwrap();
        let doc_id = fields
            .iter()
            .find(|f| f.kind == FieldKind::File)
            .unwrap()
            .id;
        let mut record = db.record(tid, id).unwrap();
        record.values.insert(doc_id, rel.clone());
        db.save_record(&record).unwrap();
        let backup = create_backup(&db).unwrap();
        assert!(backup.join(&rel).is_file());
        assert!(verify_snapshot(&backup).is_ok());
        // Потерянное вложение делает архив непригодным для восстановления.
        fs::remove_file(backup.join(&rel)).unwrap();
        assert!(verify_snapshot(&backup).is_err());
        assert!(restore_backup(&db, &backup).is_err());
        assert_eq!(db.record(tid, id).unwrap().value(doc_id), rel);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_external_path() {
        let root = std::env::temp_dir().join(format!("kartoteka-path-test-{}", Uuid::new_v4()));
        let db = Database::new(root.clone()).unwrap();
        assert!(resolve_attachment(&db, "assets/../secret.doc").is_err());
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn restores_previous_database_and_attached_document() {
        use crate::db::Record;
        let root = std::env::temp_dir().join(format!("kartoteka-restore-test-{}", Uuid::new_v4()));
        let db = Database::new(root.clone()).unwrap();
        let table_id = db.tables().unwrap()[0].id;
        let fields = db.fields(table_id).unwrap();
        let last_name = fields.iter().find(|f| f.name == "Фамилия").unwrap().id;
        let first_name = fields.iter().find(|f| f.name == "Имя").unwrap().id;
        let document = fields
            .iter()
            .find(|f| f.kind == FieldKind::File)
            .unwrap()
            .id;
        let source = root.join("class-note.txt");
        fs::write(&source, b"File stored in the backup").unwrap();
        let attachment = import_file(&db, &source, FieldKind::File).unwrap();
        let mut record = Record::new(table_id, &fields);
        record.values.insert(last_name, "Зорина".into());
        record.values.insert(first_name, "Катя".into());
        record.values.insert(document, attachment.clone());
        let record_id = db.save_record(&record).unwrap();
        let backup = create_backup(&db).unwrap();

        let mut changed = db.record(table_id, record_id).unwrap();
        changed.values.insert(last_name, "Исправленная".into());
        db.save_record(&changed).unwrap();
        fs::remove_file(root.join(&attachment)).unwrap();
        // Даже если текущее вложение уже удалено, полную здоровую копию
        // всё равно можно восстановить. Текущий неполный снимок маркируется.
        restore_backup(&db, &backup).unwrap();
        let incomplete = list_backups(&db)
            .unwrap()
            .into_iter()
            .find(|path| path.join("WARNING.txt").is_file())
            .unwrap();
        assert!(fs::read_to_string(incomplete.join("WARNING.txt"))
            .unwrap()
            .contains("НЕПОЛНАЯ"));
        assert_eq!(
            db.record(table_id, record_id).unwrap().value(last_name),
            "Зорина"
        );
        assert_eq!(
            db.record(table_id, record_id).unwrap().value(document),
            attachment
        );
        assert_eq!(
            fs::read(root.join(&attachment)).unwrap(),
            b"File stored in the backup"
        );
        assert!(list_backups(&db).unwrap().len() >= 2);
        let _ = fs::remove_dir_all(root);
    }
}
