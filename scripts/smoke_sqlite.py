#!/usr/bin/env python3
"""Офлайн smoke-test SQL-схемы, извлечённой из настоящего src/db.rs.

Эта проверка использует штатный Python sqlite3 и НЕ заменяет cargo test.
Команда: python scripts/smoke_sqlite.py
"""
from __future__ import annotations

import sqlite3
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DB_SOURCE = (ROOT / "src" / "db.rs").read_text(encoding="utf-8")
MARKER = 'conn.execute_batch(r#"'
assert DB_SOURCE.count(MARKER) == 1, "Не удалось однозначно извлечь SQL из src/db.rs"
SCHEMA = DB_SOURCE.split(MARKER, 1)[1].split('"#).map_err(err)?;', 1)[0]


def check() -> None:
    with tempfile.TemporaryDirectory(prefix="kartoteka-sql-test-") as folder:
        path = Path(folder) / "program_data.db"
        connection = sqlite3.connect(path)
        connection.execute("PRAGMA foreign_keys=ON")
        connection.executescript(SCHEMA)
        connection.execute("INSERT INTO ui_tables(name,id_label) VALUES (?,?)", ("Одноклассники", "ID_Ученика"))
        table = connection.execute("SELECT id FROM ui_tables").fetchone()[0]
        connection.execute("INSERT INTO ui_fields(table_id,name,kind,sort_order) VALUES (?,?,?,?)", (table,"Фамилия","short",0))
        field = connection.execute("SELECT id FROM ui_fields").fetchone()[0]
        connection.execute("INSERT INTO ui_records(table_id,created_at,updated_at) VALUES (?,?,?)", (table,"2026-10-24","2026-10-24"))
        record = connection.execute("SELECT id FROM ui_records").fetchone()[0]
        connection.execute("INSERT INTO ui_values(record_id,field_id,value) VALUES (?,?,?)", (record,field,"Иванова"))
        result = connection.execute("""SELECT r.id, v.value
            FROM ui_records AS r LEFT JOIN ui_fields AS f ON f.table_id=r.table_id
            LEFT JOIN ui_values AS v ON v.record_id=r.id AND v.field_id=f.id
            WHERE r.table_id=? ORDER BY r.id""", (table,)).fetchall()
        assert result == [(record, "Иванова")], result
        assert "иван" in result[0][1].lower()

        connection.execute("INSERT INTO ui_fields(table_id,name,kind,sort_order) VALUES (?,?,?,?)", (table,"Хобби","long",1))
        hobby = connection.execute("SELECT id FROM ui_fields WHERE name='Хобби'").fetchone()[0]
        connection.execute("UPDATE ui_fields SET sort_order=1 WHERE id=?", (field,))
        connection.execute("UPDATE ui_fields SET sort_order=0 WHERE id=?", (hobby,))
        assert connection.execute("SELECT name FROM ui_fields ORDER BY sort_order").fetchall() == [("Хобби",),("Фамилия",)]
        assert connection.execute("SELECT value FROM ui_values WHERE record_id=?", (record,)).fetchone()[0] == "Иванова"

        # SQLite NOCASE покрывает ASCII, но НЕ регистр кириллицы.
        # Проверка русских имён должна выполняться Rust-кодом update_field.
        connection.execute("INSERT INTO ui_fields(table_id,name,kind) VALUES (?,?,?)", (table,"Email","short"))
        try:
            connection.execute("INSERT INTO ui_fields(table_id,name,kind) VALUES (?,?,?)", (table,"email","short"))
        except sqlite3.IntegrityError:
            pass
        else:
            raise AssertionError("Повторяющиеся латинские имена полей не блокируются")
        assert "other.name.to_lowercase()==normalized_name.to_lowercase()" in DB_SOURCE, \
            "Русские дублирующиеся имена полей не проверяются в Rust"

        copy = Path(folder) / "backup.db"
        connection.commit()
        connection.execute("VACUUM INTO ?", (str(copy),))
        assert copy.exists()
        with sqlite3.connect(copy) as backup:
            assert backup.execute("PRAGMA quick_check").fetchone()[0] == "ok"
            assert backup.execute("SELECT value FROM ui_values").fetchone()[0] == "Иванова"
        connection.execute("DELETE FROM ui_fields WHERE id=?", (field,))
        assert connection.execute("SELECT count(*) FROM ui_values").fetchone()[0] == 0
        connection.close()
    print("OK: SQL-схема, сохранение, поиск, перестановка полей, уникальность, каскадное удаление, VACUUM INTO")


if __name__ == "__main__":
    check()
