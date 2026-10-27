//! Хранение данных в локальной SQLite. Запросы параметризованы; структура
//! пользовательских таблиц описывается метаданными. Никакой сети здесь нет.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub type DbResult<T> = Result<T, String>;
fn err<E: std::fmt::Display>(error: E) -> String {
    error.to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    ShortText,
    LongText,
    Integer,
    Decimal,
    Date,
    Time,
    DateTime,
    Boolean,
    Url,
    File,
    Image,
}
impl FieldKind {
    pub const ALL: [Self; 11] = [
        Self::ShortText,
        Self::LongText,
        Self::Integer,
        Self::Decimal,
        Self::Date,
        Self::Time,
        Self::DateTime,
        Self::Boolean,
        Self::Url,
        Self::File,
        Self::Image,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::ShortText => crate::i18n::tr("Короткий текст"),
            Self::LongText => crate::i18n::tr("Длинный текст"),
            Self::Integer => crate::i18n::tr("Целое число"),
            Self::Decimal => crate::i18n::tr("Десятичное число"),
            Self::Date => crate::i18n::tr("Дата"),
            Self::Time => crate::i18n::tr("Время"),
            Self::DateTime => crate::i18n::tr("Дата и время"),
            Self::Boolean => crate::i18n::tr("Да / Нет"),
            Self::Url => crate::i18n::tr("Ссылка на сайт"),
            Self::File => crate::i18n::tr("Документ"),
            Self::Image => crate::i18n::tr("Фотография"),
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::ShortText => "short",
            Self::LongText => "long",
            Self::Integer => "int",
            Self::Decimal => "decimal",
            Self::Date => "date",
            Self::Time => "time",
            Self::DateTime => "datetime",
            Self::Boolean => "bool",
            Self::Url => "url",
            Self::File => "file",
            Self::Image => "image",
        }
    }
    fn parse(s: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|k| k.key() == s)
            .unwrap_or(Self::ShortText)
    }
    pub fn validate(self, value: &str) -> DbResult<()> {
        let s = value.trim();
        if s.is_empty() {
            return Ok(());
        }
        match self {
            Self::ShortText if s.chars().count() > 255 => {
                Err(crate::i18n::tr("не более 255 символов").into())
            }
            Self::Integer if s.parse::<i64>().is_err() => {
                Err(crate::i18n::tr("укажите целое число").into())
            }
            Self::Decimal
                if s.replace(',', ".")
                    .parse::<f64>()
                    .map(|v| !v.is_finite())
                    .unwrap_or(true) =>
            {
                Err(crate::i18n::tr("укажите число").into())
            }
            Self::Date if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_err() => {
                Err(crate::i18n::tr("дата: ГГГГ-ММ-ДД").into())
            }
            Self::Time if NaiveTime::parse_from_str(s, "%H:%M").is_err() => {
                Err(crate::i18n::tr("время: ЧЧ:ММ").into())
            }
            Self::DateTime if NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").is_err() => {
                Err(crate::i18n::tr("дата и время: ГГГГ-ММ-ДД ЧЧ:ММ").into())
            }
            Self::Boolean if s != "0" && s != "1" => {
                Err(crate::i18n::tr("допустимо только Да или Нет").into())
            }
            Self::Url if !valid_web_url(s) => Err(crate::i18n::tr(
                "укажите полный адрес сайта, например https://example.com",
            )
            .into()),
            Self::File if !valid_asset_path(s, "assets/documents/") => {
                Err(crate::i18n::tr("недопустимый путь к документу").into())
            }
            Self::Image if !valid_asset_path(s, "assets/images/") => {
                Err(crate::i18n::tr("недопустимый путь к фотографии").into())
            }
            _ => Ok(()),
        }
    }
}

/// Проверка пользовательского URL без отправки запросов в сеть.
fn valid_web_url(value: &str) -> bool {
    if value.chars().any(char::is_whitespace) || value.contains('\\') {
        return false;
    }
    let address = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"));
    match address {
        None => false,
        Some(rest) => {
            let host = rest.split(&['/', '?', '#'][..]).next().unwrap_or("");
            !host.is_empty() && host != "." && !host.contains('@') && !host.starts_with(':')
        }
    }
}

/// В базе храним только относительные пути внутри назначенной папки.
fn valid_asset_path(value: &str, prefix: &str) -> bool {
    let Some(name) = value.strip_prefix(prefix) else {
        return false;
    };
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.chars().any(char::is_control)
}

#[derive(Clone, Debug)]
pub struct TableDef {
    pub id: i64,
    pub name: String,
    pub id_label: String,
}
#[derive(Clone, Debug)]
pub struct FieldDef {
    pub id: i64,
    pub table_id: i64,
    pub name: String,
    pub kind: FieldKind,
    pub required: bool,
    pub default_value: String,
    pub order: i64,
    pub visible: bool,
    pub width: i64,
}
impl FieldDef {
    pub fn blank(table_id: i64) -> Self {
        Self {
            id: 0,
            table_id,
            name: String::new(),
            kind: FieldKind::ShortText,
            required: false,
            default_value: String::new(),
            order: 0,
            visible: true,
            width: 220,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub id: i64,
    pub table_id: i64,
    pub values: BTreeMap<i64, String>,
}
impl Record {
    pub fn new(table_id: i64, fields: &[FieldDef]) -> Self {
        Self {
            id: 0,
            table_id,
            values: fields
                .iter()
                .map(|f| (f.id, f.default_value.clone()))
                .collect(),
        }
    }
    pub fn value(&self, field_id: i64) -> &str {
        self.values.get(&field_id).map(String::as_str).unwrap_or("")
    }
}

#[derive(Clone)]
pub struct Database {
    pub root: PathBuf,
    pub path: PathBuf,
}
impl Database {
    pub fn new(root: PathBuf) -> DbResult<Self> {
        std::fs::create_dir_all(&root).map_err(err)?;
        let db = Self {
            path: root.join("program_data.db"),
            root,
        };
        db.initialize()?;
        Ok(db)
    }
    pub fn connection(&self) -> DbResult<Connection> {
        let conn = Connection::open(&self.path).map_err(err)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(err)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;").map_err(err)?;
        Ok(conn)
    }
    fn initialize(&self) -> DbResult<()> {
        let mut conn = self.connection()?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=DELETE;
            CREATE TABLE IF NOT EXISTS ui_tables (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                id_label TEXT NOT NULL DEFAULT 'ID'
            );
            CREATE TABLE IF NOT EXISTS ui_fields (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id INTEGER NOT NULL REFERENCES ui_tables(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                required INTEGER NOT NULL DEFAULT 0,
                default_value TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0,
                visible INTEGER NOT NULL DEFAULT 1,
                width INTEGER NOT NULL DEFAULT 220,
                UNIQUE(table_id, name COLLATE NOCASE)
            );
            CREATE TABLE IF NOT EXISTS ui_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id INTEGER NOT NULL REFERENCES ui_tables(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_records_table ON ui_records(table_id, id);
            CREATE TABLE IF NOT EXISTS ui_values (
                record_id INTEGER NOT NULL REFERENCES ui_records(id) ON DELETE CASCADE,
                field_id INTEGER NOT NULL REFERENCES ui_fields(id) ON DELETE CASCADE,
                value TEXT NOT NULL,
                PRIMARY KEY(record_id, field_id)
            );
            CREATE INDEX IF NOT EXISTS idx_values_field ON ui_values(field_id, value);
        "#,
        )
        .map_err(err)?;
        let count: i64 = conn
            .query_row("SELECT count(*) FROM ui_tables", [], |r| r.get(0))
            .map_err(err)?;
        if count > 0 {
            return Ok(());
        }
        let tx = conn.transaction().map_err(err)?;
        tx.execute(
            "INSERT INTO ui_tables(name, id_label) VALUES('Одноклассники', 'ID_Ученика')",
            [],
        )
        .map_err(err)?;
        let table_id = tx.last_insert_rowid();
        let defaults = [
            ("Фамилия", FieldKind::ShortText, true),
            ("Имя", FieldKind::ShortText, true),
            ("Отчество", FieldKind::ShortText, false),
            ("Дата_Рождения", FieldKind::Date, false),
            ("Телефон", FieldKind::ShortText, false),
            ("Адрес_Проживания", FieldKind::ShortText, false),
            ("Увлечения_Хобби", FieldKind::LongText, false),
            ("Заметки", FieldKind::LongText, false),
            ("Клички", FieldKind::ShortText, false),
            ("Фотография", FieldKind::Image, false),
            ("Ссылка_На_Сайт", FieldKind::Url, false),
            ("Документ", FieldKind::File, false),
        ];
        for (i, (name, kind, required)) in defaults.iter().enumerate() {
            tx.execute("INSERT INTO ui_fields(table_id,name,kind,required,sort_order) VALUES(?1,?2,?3,?4,?5)",
                params![table_id, name, kind.key(), *required as i64, i as i64]).map_err(err)?;
        }
        tx.commit().map_err(err)
    }
    pub fn tables(&self) -> DbResult<Vec<TableDef>> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare("SELECT id,name,id_label FROM ui_tables ORDER BY id")
            .map_err(err)?;
        // Сохраняем результат в локальной переменной: итератор MappedRows
        // должен освободить заимствование до уничтожения stmt и conn.
        let tables = stmt
            .query_map([], |r| {
                Ok(TableDef {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    id_label: r.get(2)?,
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(tables)
    }
    pub fn fields(&self, table_id: i64) -> DbResult<Vec<FieldDef>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT id,table_id,name,kind,required,default_value,sort_order,visible,width FROM ui_fields WHERE table_id=?1 ORDER BY sort_order,id").map_err(err)?;
        // Явно завершаем сбор строк до выхода из блока, чтобы stmt и conn
        // жили дольше заимствующего их курсора SQLite.
        let fields = stmt
            .query_map([table_id], |r| {
                let kind: String = r.get(3)?;
                Ok(FieldDef {
                    id: r.get(0)?,
                    table_id: r.get(1)?,
                    name: r.get(2)?,
                    kind: FieldKind::parse(&kind),
                    required: r.get::<_, i64>(4)? != 0,
                    default_value: r.get(5)?,
                    order: r.get(6)?,
                    visible: r.get::<_, i64>(7)? != 0,
                    width: r.get(8)?,
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(fields)
    }
    pub fn create_table(&self, name: &str) -> DbResult<i64> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 80 {
            return Err(crate::i18n::tr("Название таблицы: от 1 до 80 символов").into());
        }
        // SQLite NOCASE различает русские строчные и заглавные буквы.
        if self
            .tables()?
            .iter()
            .any(|t| t.name.to_lowercase() == name.to_lowercase())
        {
            return Err(crate::i18n::tr("Таблица с таким названием уже существует").into());
        }
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ui_tables(name,id_label) VALUES(?1,'ID')",
            [name],
        )
        .map_err(err)?;
        Ok(conn.last_insert_rowid())
    }
    pub fn update_field(&self, f: &FieldDef) -> DbResult<i64> {
        let normalized_name = f.name.trim();
        if normalized_name.is_empty() || normalized_name.chars().count() > 80 {
            return Err(crate::i18n::tr("Название поля: от 1 до 80 символов").into());
        }
        if self.fields(f.table_id)?.iter().any(|other| {
            other.id != f.id && other.name.to_lowercase() == normalized_name.to_lowercase()
        }) {
            return Err(crate::i18n::tr("Поле с таким названием уже существует").into());
        }
        f.kind
            .validate(&f.default_value)
            .map_err(|e| match crate::i18n::current() {
                crate::i18n::Language::Ru => format!("Значение по умолчанию: {e}"),
                crate::i18n::Language::De => format!("Standardwert: {e}"),
                crate::i18n::Language::En => format!("Default value: {e}"),
            })?;
        if f.required
            && f.default_value.trim().is_empty()
            && f.id == 0
            && self.record_ids(f.table_id, "")?.len() > 0
        {
            return Err(crate::i18n::tr(
                "Нельзя добавить обязательное поле без значения по умолчанию в непустую таблицу",
            )
            .into());
        }
        let mut conn = self.connection()?;
        let tx = conn.transaction().map_err(err)?;
        if f.id > 0 {
            let old_kind: String = tx
                .query_row(
                    "SELECT kind FROM ui_fields WHERE id=?1 AND table_id=?2",
                    params![f.id, f.table_id],
                    |r| r.get(0),
                )
                .map_err(err)?;
            if old_kind != f.kind.key() || f.required {
                let mut stmt = tx
                    .prepare("SELECT value FROM ui_values WHERE field_id=?1")
                    .map_err(err)?;
                let all = stmt
                    .query_map([f.id], |r| r.get::<_, String>(0))
                    .map_err(err)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(err)?;
                for value in all {
                    if f.required && value.trim().is_empty() {
                        return Err(match crate::i18n::current() {
                            crate::i18n::Language::Ru => {
                                format!("Поле «{}»: уже есть пустые значения", f.name)
                            }
                            crate::i18n::Language::De => {
                                format!("Feld „{}“ enthält bereits leere Werte", f.name)
                            }
                            crate::i18n::Language::En => {
                                format!("Field “{}” already contains empty values", f.name)
                            }
                        });
                    }
                    f.kind
                        .validate(&value)
                        .map_err(|e| match crate::i18n::current() {
                            crate::i18n::Language::Ru => {
                                format!("Несовместимое значение в «{}»: {e}", f.name)
                            }
                            crate::i18n::Language::De => {
                                format!("Unverträglicher Wert in „{}“: {e}", f.name)
                            }
                            crate::i18n::Language::En => {
                                format!("Incompatible value in “{}”: {e}", f.name)
                            }
                        })?;
                }
                if f.required && f.default_value.trim().is_empty() {
                    let missing:i64=tx.query_row("SELECT count(*) FROM ui_records r WHERE r.table_id=?1 AND NOT EXISTS(SELECT 1 FROM ui_values v WHERE v.record_id=r.id AND v.field_id=?2)",params![f.table_id,f.id],|r|r.get(0)).map_err(err)?;
                    if missing > 0 {
                        return Err(crate::i18n::tr("В существующих записях есть пустые значения: поле нельзя сделать обязательным").into());
                    }
                }
            }
            if f.required && !f.default_value.is_empty() {
                tx.execute("INSERT OR IGNORE INTO ui_values(record_id,field_id,value) SELECT id,?1,?2 FROM ui_records WHERE table_id=?3",
                    params![f.id,f.default_value,f.table_id]).map_err(err)?;
            }
            tx.execute("UPDATE ui_fields SET name=?1,kind=?2,required=?3,default_value=?4,sort_order=?5,visible=?6,width=?7 WHERE id=?8 AND table_id=?9",
                params![f.name.trim(),f.kind.key(),f.required as i64,f.default_value,f.order,f.visible as i64,f.width.clamp(100,600),f.id,f.table_id]).map_err(err)?;
            tx.commit().map_err(err)?;
            Ok(f.id)
        } else {
            tx.execute("INSERT INTO ui_fields(table_id,name,kind,required,default_value,sort_order,visible,width) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![f.table_id,f.name.trim(),f.kind.key(),f.required as i64,f.default_value,f.order,f.visible as i64,f.width.clamp(100,600)]).map_err(err)?;
            let id = tx.last_insert_rowid();
            if !f.default_value.is_empty() {
                tx.execute("INSERT INTO ui_values(record_id,field_id,value) SELECT id,?1,?2 FROM ui_records WHERE table_id=?3",
                    params![id,f.default_value,f.table_id]).map_err(err)?;
            }
            tx.commit().map_err(err)?;
            Ok(id)
        }
    }
    /// Меняет порядок полей в одной таблице без затрагивания данных учеников.
    /// Возвращает false, если поле уже первое/последнее.
    pub fn move_field(&self, table_id: i64, field_id: i64, direction: i32) -> DbResult<bool> {
        if direction != -1 && direction != 1 {
            return Err(crate::i18n::tr("Направление должно быть -1 или 1").into());
        }
        let fields = self.fields(table_id)?;
        let pos = fields
            .iter()
            .position(|f| f.id == field_id)
            .ok_or_else(|| crate::i18n::tr("Поле не найдено в выбранной таблице").to_string())?;
        let destination = pos as i64 + direction as i64;
        if destination < 0 || destination >= fields.len() as i64 {
            return Ok(false);
        }
        let mut order = fields.iter().map(|f| f.id).collect::<Vec<_>>();
        order.swap(pos, destination as usize);
        let mut conn = self.connection()?;
        let tx = conn.transaction().map_err(err)?;
        for (i, id) in order.into_iter().enumerate() {
            tx.execute(
                "UPDATE ui_fields SET sort_order=?1 WHERE id=?2 AND table_id=?3",
                params![i as i64, id, table_id],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)?;
        Ok(true)
    }

    pub fn delete_field(&self, field_id: i64, table_id: i64) -> DbResult<()> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ui_fields WHERE id=?1 AND table_id=?2",
            params![field_id, table_id],
        )
        .map_err(err)?;
        Ok(())
    }
    /// Поиск одним проходом по данным выбранной таблицы. Стандартный LIKE
    /// SQLite не умеет регистронезависимо сравнивать кириллицу, поэтому
    /// сопоставление выполняется Unicode-операцией Rust. Вместо одного
    /// запроса на каждую запись используем единый JOIN и потоковую обработку.
    pub fn record_ids(&self, table_id: i64, query: &str) -> DbResult<Vec<i64>> {
        let conn = self.connection()?;
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            let mut stmt = conn
                .prepare("SELECT id FROM ui_records WHERE table_id=?1 ORDER BY id")
                .map_err(err)?;
            return stmt
                .query_map([table_id], |r| r.get::<_, i64>(0))
                .map_err(err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(err);
        }

        let mut stmt = conn
            .prepare(
                r#"
            SELECT r.id, v.value
            FROM ui_records AS r
            LEFT JOIN ui_fields AS f ON f.table_id = r.table_id
            LEFT JOIN ui_values AS v ON v.record_id = r.id AND v.field_id = f.id
            WHERE r.table_id = ?1
            ORDER BY r.id
        "#,
            )
            .map_err(err)?;
        let mut rows = stmt.query([table_id]).map_err(err)?;
        let mut found = Vec::new();
        let mut previous_id: Option<i64> = None;
        let mut matched = false;
        while let Some(row) = rows.next().map_err(err)? {
            let id: i64 = row.get(0).map_err(err)?;
            if previous_id != Some(id) {
                if let Some(old_id) = previous_id {
                    if matched {
                        found.push(old_id);
                    }
                }
                previous_id = Some(id);
                matched = id.to_string().contains(&needle);
            }
            if !matched {
                let value: Option<String> = row.get(1).map_err(err)?;
                matched = value
                    .as_deref()
                    .is_some_and(|v| v.to_lowercase().contains(&needle));
            }
        }
        if let Some(id) = previous_id {
            if matched {
                found.push(id);
            }
        }
        Ok(found)
    }
    pub fn search_records(
        &self,
        table_id: i64,
        query: &str,
        limit: usize,
    ) -> DbResult<Vec<Record>> {
        let ids = self.record_ids(table_id, query)?;
        let conn = self.connection()?;
        let mut stmt=conn.prepare("SELECT v.field_id,v.value FROM ui_values v JOIN ui_fields f ON f.id=v.field_id WHERE v.record_id=?1 AND f.table_id=?2").map_err(err)?;
        let mut result = Vec::new();
        for id in ids.into_iter().take(limit) {
            let values = stmt
                .query_map(params![id, table_id], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(err)?
                .collect::<Result<BTreeMap<i64, String>, _>>()
                .map_err(err)?;
            result.push(Record {
                id,
                table_id,
                values,
            });
        }
        Ok(result)
    }
    pub fn record(&self, table_id: i64, id: i64) -> DbResult<Record> {
        let conn = self.connection()?;
        let found = conn
            .query_row(
                "SELECT id FROM ui_records WHERE id=?1 AND table_id=?2",
                params![id, table_id],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .map_err(err)?;
        if found.is_none() {
            return Err(crate::i18n::tr("Запись не найдена").into());
        }
        let mut stmt=conn.prepare("SELECT v.field_id,v.value FROM ui_values v JOIN ui_fields f ON f.id=v.field_id WHERE v.record_id=?1 AND f.table_id=?2").map_err(err)?;
        let values = stmt
            .query_map(params![id, table_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(err)?
            .collect::<Result<BTreeMap<i64, String>, _>>()
            .map_err(err)?;
        Ok(Record {
            id,
            table_id,
            values,
        })
    }
    pub fn save_record(&self, record: &Record) -> DbResult<i64> {
        let fields = self.fields(record.table_id)?;
        for f in &fields {
            let val = record.value(f.id);
            if f.required && val.trim().is_empty() {
                return Err(match crate::i18n::current() {
                    crate::i18n::Language::Ru => {
                        format!("Заполните поле «{}»", f.name.replace('_', " "))
                    }
                    crate::i18n::Language::De => {
                        format!("Feld „{}“ ausfüllen", f.name.replace('_', " "))
                    }
                    crate::i18n::Language::En => {
                        format!("Fill in field “{}”", f.name.replace('_', " "))
                    }
                });
            }
            f.kind
                .validate(val)
                .map_err(|e| match crate::i18n::current() {
                    crate::i18n::Language::Ru => {
                        format!("Поле «{}»: {e}", f.name.replace('_', " "))
                    }
                    crate::i18n::Language::De => {
                        format!("Feld „{}“: {e}", f.name.replace('_', " "))
                    }
                    crate::i18n::Language::En => {
                        format!("Field “{}”: {e}", f.name.replace('_', " "))
                    }
                })?;
        }
        let mut conn = self.connection()?;
        let tx = conn.transaction().map_err(err)?;
        let now = Utc::now().to_rfc3339();
        let id = if record.id == 0 {
            tx.execute(
                "INSERT INTO ui_records(table_id,created_at,updated_at) VALUES(?1,?2,?2)",
                params![record.table_id, now],
            )
            .map_err(err)?;
            tx.last_insert_rowid()
        } else {
            if tx
                .execute(
                    "UPDATE ui_records SET updated_at=?1 WHERE id=?2 AND table_id=?3",
                    params![now, record.id, record.table_id],
                )
                .map_err(err)?
                != 1
            {
                return Err(crate::i18n::tr("Запись была удалена или перемещена").into());
            }
            record.id
        };
        for f in &fields {
            let val = record.value(f.id);
            if val.is_empty() {
                tx.execute(
                    "DELETE FROM ui_values WHERE record_id=?1 AND field_id=?2",
                    params![id, f.id],
                )
                .map_err(err)?;
            } else {
                tx.execute("INSERT INTO ui_values(record_id,field_id,value) VALUES(?1,?2,?3) ON CONFLICT(record_id,field_id) DO UPDATE SET value=excluded.value",
                    params![id,f.id,val]).map_err(err)?;
            }
        }
        tx.commit().map_err(err)?;
        Ok(id)
    }
    pub fn delete_record(&self, table_id: i64, id: i64) -> DbResult<()> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ui_records WHERE id=?1 AND table_id=?2",
            params![id, table_id],
        )
        .map_err(err)?;
        Ok(())
    }
    /// Защищает CSV от формул Excel/LibreOffice, включая случай, когда
    /// перед =,+,-,@ стоит пробел, табуляция или перевод строки.
    fn csv_cell(value: &str) -> String {
        let significant = value.trim_start_matches(|c: char| c.is_whitespace() || c.is_control());
        let protected = if matches!(significant.chars().next(), Some('=' | '+' | '-' | '@')) {
            format!("'{value}")
        } else {
            value.to_owned()
        };
        format!("\"{}\"", protected.replace('"', "\"\""))
    }
    /// Экспортирует таблицу в UTF-8 CSV (с BOM для удобного открытия в Excel).
    /// Формулы из пользовательского текста экранируются, чтобы Excel не
    /// исполнял выражения из данных при открытии файла.
    pub fn export_csv(&self, table_id: i64, destination: &Path) -> DbResult<usize> {
        use std::io::{BufWriter, Write};
        let fields = self.fields(table_id)?;
        let name = self
            .tables()?
            .into_iter()
            .find(|t| t.id == table_id)
            .ok_or_else(|| crate::i18n::tr("Таблица не найдена").to_string())?;
        let file = std::fs::File::create(destination).map_err(err)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&[0xEF, 0xBB, 0xBF]).map_err(err)?;
        let mut header = vec![Self::csv_cell(&name.id_label)];
        header.extend(fields.iter().map(|f| Self::csv_cell(&f.name)));
        writeln!(writer, "{}", header.join(";")).map_err(err)?;
        // Экспортируются ВСЕ записи, в том числе скрытые сейчас в таблице UI.
        let ids = self.record_ids(table_id, "")?;
        for id in &ids {
            let record = self.record(table_id, *id)?;
            let mut row = vec![Self::csv_cell(&record.id.to_string())];
            row.extend(fields.iter().map(|f| Self::csv_cell(record.value(f.id))));
            writeln!(writer, "{}", row.join(";")).map_err(err)?;
        }
        writer.flush().map_err(err)?;
        Ok(ids.len())
    }

    pub fn backup_database(&self, destination: &Path) -> DbResult<()> {
        if destination.exists() {
            return Err(crate::i18n::tr("Файл резервной копии уже существует").into());
        }
        let conn = self.connection()?;
        conn.execute(
            "VACUUM INTO ?1",
            [destination.to_string_lossy().to_string()],
        )
        .map_err(err)?;
        Ok(())
    }
    pub fn quick_check(path: &Path) -> DbResult<()> {
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(err)?;
        let check: String = conn
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(err)?;
        if check != "ok" {
            return Err(match crate::i18n::current() {
                crate::i18n::Language::Ru => format!("Проверка SQLite: {check}"),
                crate::i18n::Language::De => format!("SQLite-Prüfung: {check}"),
                crate::i18n::Language::En => format!("SQLite check: {check}"),
            });
        }
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='ui_records'",
                [],
                |r| r.get(0),
            )
            .map_err(err)?;
        if count != 1 {
            return Err(crate::i18n::tr("Это не резервная копия нашей картотеки").into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp() -> PathBuf {
        std::env::temp_dir().join(format!("kartoteka-test-{}", uuid::Uuid::new_v4()))
    }
    #[test]
    fn sqlite_save_search_and_reload() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let table = db.tables().unwrap().remove(0);
        let fields = db.fields(table.id).unwrap();
        let mut record = Record::new(table.id, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                record.values.insert(f.id, "Смирнова".into());
            }
            if f.name == "Имя" {
                record.values.insert(f.id, "Мария".into());
            }
        }
        let id = db.save_record(&record).unwrap();
        assert_eq!(db.record_ids(table.id, "смир").unwrap(), vec![id]);
        assert_eq!(db.record_ids(table.id, "СМИР").unwrap(), vec![id]);
        assert_eq!(db.record(table.id, id).unwrap().id, id);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn rejects_duplicate_russian_names_with_different_case() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let table = db.tables().unwrap()[0].id;
        assert!(db.create_table("ОДНОКЛАССНИКИ").is_err());
        let mut field = FieldDef::blank(table);
        field.name = "фамилия".into();
        assert!(db.update_field(&field).is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reject_wrong_date_and_invalid_url() {
        assert!(FieldKind::Date.validate("2026-02-30").is_err());
        assert!(FieldKind::Url.validate("javascript:alert(1)").is_err());
        assert!(FieldKind::Url.validate("https://").is_err());
        assert!(FieldKind::Url.validate("https://example.org/").is_ok());
        assert!(FieldKind::Image.validate("assets/images/photo.png").is_ok());
        assert!(FieldKind::Image
            .validate("assets/documents/file.pdf")
            .is_err());
        assert!(FieldKind::File
            .validate("assets/documents/../evil.pdf")
            .is_err());
    }
    #[test]
    fn incompatible_type_change_is_rejected_without_losing_value() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let table = db.tables().unwrap().remove(0);
        let fields = db.fields(table.id).unwrap();
        let mut record = Record::new(table.id, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                record.values.insert(f.id, "Петрова".into());
            }
            if f.name == "Имя" {
                record.values.insert(f.id, "Анна".into());
            }
        }
        let id = db.save_record(&record).unwrap();
        let mut surname = fields.into_iter().find(|f| f.name == "Фамилия").unwrap();
        surname.kind = FieldKind::Integer;
        assert!(db.update_field(&surname).is_err());
        assert_eq!(
            db.record(table.id, id).unwrap().value(surname.id),
            "Петрова"
        );
        let actual = db
            .fields(table.id)
            .unwrap()
            .into_iter()
            .find(|f| f.id == surname.id)
            .unwrap();
        assert_eq!(actual.kind, FieldKind::ShortText);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn creating_another_table_does_not_replace_students() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let tid = db.create_table("Учителя").unwrap();
        assert_eq!(db.tables().unwrap().len(), 2);
        let mut f = FieldDef::blank(tid);
        f.name = "Предмет".into();
        db.update_field(&f).unwrap();
        assert_eq!(db.fields(tid).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn csv_export_escapes_excel_formulas() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let tid = db.tables().unwrap().remove(0).id;
        let fields = db.fields(tid).unwrap();
        let mut r = Record::new(tid, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                r.values.insert(f.id, "=1+1".into());
            }
            if f.name == "Имя" {
                r.values.insert(f.id, "Ольга;\"Тест\"".into());
            }
        }
        let id = db.save_record(&r).unwrap();
        assert!(id > 0);
        let dest = root.join("export.csv");
        assert_eq!(db.export_csv(tid, &dest).unwrap(), 1);
        let bytes = std::fs::read(&dest).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("' =1+1") == false);
        assert!(text.contains("'=1+1"));
        assert!(text.contains("Ольга;\"\"Тест\"\""));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn csv_blocks_formulas_after_leading_whitespace() {
        assert_eq!(Database::csv_cell(" =2+2"), "\"' =2+2\"");
        assert_eq!(Database::csv_cell("\t@SUM(1)"), "\"'\t@SUM(1)\"");
        assert_eq!(Database::csv_cell("\n+1"), "\"'\n+1\"");
        assert_eq!(Database::csv_cell("Привет"), "\"Привет\"");
    }

    #[test]
    fn unicode_search_uses_only_selected_table_and_matches_id() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let students = db.tables().unwrap()[0].id;
        let student_fields = db.fields(students).unwrap();
        let mut student = Record::new(students, &student_fields);
        for f in &student_fields {
            if f.name == "Имя" {
                student.values.insert(f.id, "Ёлка".into());
            }
            if f.name == "Фамилия" {
                student.values.insert(f.id, "Иванова".into());
            }
        }
        let id = db.save_record(&student).unwrap();
        let teachers = db.create_table("Учителя").unwrap();
        let mut field = FieldDef::blank(teachers);
        field.name = "Имя".into();
        let fid = db.update_field(&field).unwrap();
        let mut teacher = Record::new(teachers, &db.fields(teachers).unwrap());
        teacher.values.insert(fid, "Ёлка".into());
        db.save_record(&teacher).unwrap();
        assert_eq!(db.record_ids(students, "ёлка").unwrap(), vec![id]);
        assert_eq!(db.record_ids(students, "ЁЛКА").unwrap(), vec![id]);
        assert_eq!(db.record_ids(students, &id.to_string()).unwrap(), vec![id]);
        assert_eq!(
            db.record_ids(students, "Несуществующее").unwrap(),
            Vec::<i64>::new()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn moving_fields_preserves_values_and_is_reversible() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let table = db.tables().unwrap().remove(0);
        let fields = db.fields(table.id).unwrap();
        let surname = fields.iter().find(|f| f.name == "Фамилия").unwrap().id;
        let name = fields.iter().find(|f| f.name == "Имя").unwrap().id;
        let mut r = Record::new(table.id, &fields);
        r.values.insert(surname, "Петров".into());
        r.values.insert(name, "Олег".into());
        let id = db.save_record(&r).unwrap();
        assert!(!db.move_field(table.id, surname, -1).unwrap());
        assert!(db.move_field(table.id, surname, 1).unwrap());
        assert_eq!(db.fields(table.id).unwrap()[0].id, name);
        assert_eq!(db.record(table.id, id).unwrap().value(surname), "Петров");
        assert!(db.move_field(table.id, surname, -1).unwrap());
        assert_eq!(db.fields(table.id).unwrap()[0].id, surname);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn required_new_field_without_default_does_not_damage_records() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let tid = db.tables().unwrap().remove(0).id;
        let fields = db.fields(tid).unwrap();
        let mut r = Record::new(tid, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                r.values.insert(f.id, "Ким".into());
            }
            if f.name == "Имя" {
                r.values.insert(f.id, "Олег".into());
            }
        }
        let record_id = db.save_record(&r).unwrap();
        let mut f = FieldDef::blank(tid);
        f.name = "Школа".into();
        f.required = true;
        assert!(db.update_field(&f).is_err());
        assert_eq!(db.record(tid, record_id).unwrap().id, record_id);
        assert_eq!(db.fields(tid).unwrap().len(), fields.len());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn custom_field_with_default_populates_existing_records() {
        let root = tmp();
        let db = Database::new(root.clone()).unwrap();
        let table = db.tables().unwrap().remove(0);
        let fields = db.fields(table.id).unwrap();
        let mut r = Record::new(table.id, &fields);
        for f in &fields {
            if f.name == "Фамилия" {
                r.values.insert(f.id, "Тест".into());
            }
            if f.name == "Имя" {
                r.values.insert(f.id, "Миша".into());
            }
        }
        let id = db.save_record(&r).unwrap();
        let mut new_field = FieldDef::blank(table.id);
        new_field.name = "Любимый_Предмет".into();
        new_field.default_value = "Математика".into();
        let fid = db.update_field(&new_field).unwrap();
        assert_eq!(db.record(table.id, id).unwrap().value(fid), "Математика");
        let _ = std::fs::remove_dir_all(root);
    }
}
