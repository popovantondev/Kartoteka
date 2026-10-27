use super::*;

#[test]
fn russian_counters_and_field_labels_cover_teens_and_preserve_custom_names() {
    for (n, word) in [
        (0, "записей"),
        (1, "запись"),
        (2, "записи"),
        (4, "записи"),
        (5, "записей"),
        (11, "записей"),
        (12, "записей"),
        (14, "записей"),
        (21, "запись"),
        (22, "записи"),
        (111, "записей"),
    ] {
        assert_eq!(record_word(n), word);
    }
    assert_eq!(field_word(3), "поля");
    assert_eq!(field_word(12), "полей");
    assert_eq!(field_label("Адрес_Проживания"), "Адрес проживания");
    assert_eq!(field_label("Увлечения_Хобби"), "Увлечения, хобби");
    assert_eq!(field_label("Ссылка_На_Сайт"), "Ссылка на сайт");
    assert_eq!(field_label("URL API"), "URL API");
}

#[test]
fn sidebar_drag_can_collapse_hide_and_reopen_from_edge() {
    assert_eq!(sidebar_after_drag(Sidebar::Full, -70.0), Sidebar::Icons);
    assert_eq!(sidebar_after_drag(Sidebar::Full, -180.0), Sidebar::Hidden);
    assert_eq!(sidebar_after_drag(Sidebar::Icons, 90.0), Sidebar::Full);
    assert_eq!(sidebar_after_drag(Sidebar::Hidden, 180.0), Sidebar::Full);
    assert_eq!(sidebar_after_drag(Sidebar::Icons, 8.0), Sidebar::Icons);
}

#[test]
fn sidebar_accepts_a_slow_multi_frame_pointer_drag() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    let mut run = |events: Vec<egui::Event>| {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1360.0, 860.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.topbar(ctx);
                app.statusbar(ctx);
                app.sidebar(ctx);
            },
        );
    };
    run(vec![]);
    run(vec![]);
    run(vec![
        egui::Event::PointerMoved(egui::pos2(207.0, 300.0)),
        egui::Event::PointerButton {
            pos: egui::pos2(207.0, 300.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    for x in [
        195.0, 183.0, 171.0, 159.0, 147.0, 135.0, 123.0, 111.0, 99.0, 87.0,
    ] {
        run(vec![egui::Event::PointerMoved(egui::pos2(x, 300.0))]);
    }
    run(vec![egui::Event::PointerButton {
        pos: egui::pos2(87.0, 300.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert_eq!(
        app.prefs.sidebar,
        Sidebar::Icons,
        "drag must accumulate movement instead of checking only one frame's delta"
    );
}

#[test]
fn collapsed_sidebar_can_be_dragged_open_from_a_ten_pixel_grab_area() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(
        &ctx,
        fixture.0.clone(),
        Preferences {
            sidebar: Sidebar::Icons,
            ..Default::default()
        },
    );
    let mut run = |events: Vec<egui::Event>| {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1360.0, 860.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.topbar(ctx);
                app.statusbar(ctx);
                app.sidebar(ctx);
            },
        );
    };
    run(vec![]);
    run(vec![]);
    run(vec![
        egui::Event::PointerMoved(egui::pos2(60.0, 300.0)),
        egui::Event::PointerButton {
            pos: egui::pos2(60.0, 300.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    for x in [
        72.0, 84.0, 96.0, 108.0, 120.0, 132.0, 144.0, 156.0, 180.0, 210.0,
    ] {
        run(vec![egui::Event::PointerMoved(egui::pos2(x, 300.0))]);
    }
    run(vec![egui::Event::PointerButton {
        pos: egui::pos2(210.0, 300.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert_eq!(app.prefs.sidebar, Sidebar::Full);
}

#[test]
fn native_batched_pointer_drag_reopens_sidebar_even_with_press_and_release_in_one_frame() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(
        &ctx,
        fixture.0.clone(),
        Preferences {
            sidebar: Sidebar::Icons,
            ..Default::default()
        },
    );
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1360.0, 860.0),
            )),
            events: vec![
                egui::Event::PointerMoved(egui::pos2(60.0, 300.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(60.0, 300.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerMoved(egui::pos2(220.0, 300.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(220.0, 300.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ctx| {
            app.sidebar(ctx);
        },
    );
    assert_eq!(app.prefs.sidebar, Sidebar::Full);
}

#[test]
fn all_seven_dialogs_fit_small_viewports_and_short_prompts_do_not_grow_empty_space() {
    for (width, height) in [(1360.0, 860.0), (666.0, 466.0)] {
        for scenario in 0..7 {
            let fixture = Fixture::new();
            let ctx = egui::Context::default();
            let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
            let title = match scenario {
                0 => {
                    app.pending = Some(Action::Screen(Screen::Home));
                    "Несохранённые изменения"
                }
                1 => {
                    app.delete_record_confirm = true;
                    "Удалить запись?"
                }
                2 => {
                    app.create_table_dialog = true;
                    "Новая таблица"
                }
                3 => {
                    app.add_field_dialog = true;
                    "Добавить поле"
                }
                4 => {
                    app.add_field_dialog = true;
                    app.field_editor = app.fields[0].clone();
                    "Настроить поле"
                }
                5 => {
                    app.delete_field_confirm = true;
                    app.field_editor = app.fields[0].clone();
                    "Удалить поле и его данные?"
                }
                _ => {
                    app.restore_confirm = Some(PathBuf::from("synthetic-backup"));
                    "Восстановить резервную копию?"
                }
            };
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.dialogs(ctx),
                );
            }
            let rect = egui::AreaState::load(&ctx, egui::Id::new(title))
                .unwrap()
                .rect();
            assert!(
                rect.width() <= width + 1.0 && rect.height() <= height + 1.0,
                "{title}: {rect:?}"
            );
            if !matches!(scenario, 3 | 4) {
                assert!(
                    rect.height() < 320.0,
                    "short prompt has excessive empty space: {title} {rect:?}"
                );
            }
        }
    }
}

#[test]
fn real_record_column_drag_changes_and_persists_width_from_header_geometry() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    for name in ["Фамилия", "Имя"] {
        let id = app.fields.iter().find(|f| f.name == name).unwrap().id;
        app.selected.values.insert(id, "Тест".into());
    }
    assert!(app.save_record());
    app.screen = Screen::Table;
    let key = format!(
        "records-{}-{}",
        app.active_table,
        app.fields
            .iter()
            .map(|f| f.id.to_string())
            .collect::<Vec<_>>()
            .join("-")
    );
    let mut render = |events| {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1360.0, 860.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.topbar(ctx);
                app.statusbar(ctx);
                app.sidebar(ctx);
                app.main_area(ctx);
            },
        );
    };
    render(vec![]);
    render(vec![]);
    let rects = ctx
        .data(|d| d.get_temp::<Vec<egui::Rect>>(egui::Id::new(("column-headers", key.as_str()))))
        .unwrap();
    let r = rects[1];
    let start = egui::pos2(r.right() + 6.0, r.center().y);
    let end = start + egui::vec2(48.0, 0.0);
    render(vec![
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerMoved(end),
        egui::Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    assert!(app.prefs.column_widths[&key][1] >= r.width() + 40.0);
    let stored: Preferences =
        serde_json::from_slice(&std::fs::read(fixture.0.join("preferences.json")).unwrap())
            .unwrap();
    assert_eq!(stored.column_widths[&key], app.prefs.column_widths[&key]);
}

#[test]
fn search_result_visible_text_group_is_vertically_centered_even_when_wrapped() {
    for width in [220.0, 460.0] {
        let ctx = egui::Context::default();
        configure_style(&ctx, 1.0);
        let mut rect = egui::Rect::NOTHING;
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 240.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    rect = result_details(
                        ui,
                        "Длинная русская фамилия и имя",
                        "Телефон · учебный адрес без личных данных",
                        width,
                        84.0,
                    );
                });
            },
        );
        let union = out
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Text(t) = &s.shape {
                    Some(t.galley.mesh_bounds.translate(t.pos.to_vec2()))
                } else {
                    None
                }
            })
            .fold(egui::Rect::NOTHING, |a, b| a.union(b));
        assert!((union.center().y - rect.center().y).abs() < 0.8);
    }
}

#[test]
fn icon_and_label_form_one_centered_group_in_ltr_rtl_and_fractional_dpi() {
    for scale in [1.0, 1.25, 1.5] {
        for rtl in [false, true] {
            for primary in [false, true] {
                let ctx = egui::Context::default();
                configure_style(&ctx, 1.0);
                ctx.set_pixels_per_point(scale);
                let mut rect = egui::Rect::NOTHING;
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(650.0, 180.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.with_layout(
                                if rtl {
                                    egui::Layout::right_to_left(egui::Align::Center)
                                } else {
                                    egui::Layout::left_to_right(egui::Align::Center)
                                },
                                |ui| {
                                    rect = ui
                                        .add_sized(
                                            [460.0, 48.0],
                                            if primary {
                                                primary_button("+ Новая запись")
                                            } else {
                                                secondary_button("+ Новая запись")
                                            },
                                        )
                                        .rect;
                                },
                            );
                        });
                    },
                );
                let text = out
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Text(t) if t.galley.text() == "Новая запись" => {
                            Some(t)
                        }
                        _ => None,
                    })
                    .unwrap();
                let ink = text.galley.mesh_bounds.translate(text.pos.to_vec2());
                // The plus glyph occupies 14/24 of its 22px box: include the
                // layout box (not the short horizontal stroke) in the group.
                let label_width = ink.width();
                let expected_x = rect.center().x + 16.0;
                assert!((ink.center().x - expected_x).abs() <= 0.6 / scale + 0.1);
                assert!((ink.center().y - rect.center().y).abs() <= 0.6 / scale + 0.1);
                let group = egui::Rect::from_center_size(
                    rect.center(),
                    egui::vec2(label_width + 32.0, 22.0),
                );
                assert!(rect.contains_rect(group));
                assert!(
                    out.shapes
                        .iter()
                        .any(|shape| matches!(shape.shape, egui::Shape::LineSegment { .. })),
                    "icon not painted"
                );
            }
        }
    }
}

#[test]
fn preferences_load_041_and_persist_table_widths_without_changing_schema() {
    let old: Preferences = serde_json::from_str(r#"{"sidebar":"Full","zoom":1.0}"#).unwrap();
    assert!(old.column_widths.is_empty());
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), old);
    let schema_before = std::fs::read(&app.db().unwrap().path).unwrap();
    app.remember_widths("designer-1".into(), vec![310.0, 180.0, 150.0, 220.0]);
    let stored: Preferences =
        serde_json::from_slice(&std::fs::read(fixture.0.join("preferences.json")).unwrap())
            .unwrap();
    assert_eq!(
        stored.column_widths["designer-1"],
        vec![310.0, 180.0, 150.0, 220.0]
    );
    assert_eq!(
        schema_before,
        std::fs::read(&app.db().unwrap().path).unwrap()
    );
}

#[test]
fn form_preview_is_readable_and_does_not_write_record_defaults() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    let before = std::fs::read(&app.db().unwrap().path).unwrap();
    let out = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for field in &app.fields {
                preview_field(ui, field);
            }
        });
    });
    assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Введите значение" && t.fallback_color==INK)));
    assert!(!app.dirty);
    assert_eq!(before, std::fs::read(&app.db().unwrap().path).unwrap());
}

#[test]
fn button_text_stays_centered_when_stretched_and_in_rtl_layout() {
    for primary in [false, true] {
        for rtl in [false, true] {
            let ctx = egui::Context::default();
            configure_style(&ctx, 1.0);
            let mut button_rect = egui::Rect::NOTHING;
            let label = "Резервная копия";
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 200.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let layout = if rtl {
                            egui::Layout::right_to_left(egui::Align::Center)
                        } else {
                            egui::Layout::left_to_right(egui::Align::Center)
                        };
                        ui.with_layout(layout, |ui| {
                            button_rect = if primary {
                                ui.add_sized([400.0, 44.0], primary_button(label)).rect
                            } else {
                                ui.add_sized([400.0, 44.0], secondary_button(label)).rect
                            };
                        });
                    });
                },
            );
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.text() == label => Some(text),
                    _ => None,
                })
                .expect("button must render its label");
            let actual = egui::pos2(
                text.pos.x + text.galley.mesh_bounds.center().x,
                text.pos.y + text.galley.mesh_bounds.center().y,
            );
            assert!(
                actual.distance(button_rect.center()) <= 0.72,
                "label drift: {actual:?} / {:?}",
                button_rect.center()
            );
        }
    }
}

struct Fixture(PathBuf);

#[test]
fn complete_workflow_saves_attachments_custom_fields_export_and_restores_after_restart() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    let student_table = app.active_table;
    let field =
        |app: &Kartoteka, name: &str| app.fields.iter().find(|f| f.name == name).unwrap().id;
    app.request(Action::NewRecord);
    assert!(!app.save_record(), "required names must be validated");
    app.selected
        .values
        .insert(field(&app, "Фамилия"), "Полный цикл".into());
    app.selected
        .values
        .insert(field(&app, "Имя"), "Проверка".into());
    app.selected
        .values
        .insert(field(&app, "Дата_Рождения"), "2010-05-14".into());
    app.selected
        .values
        .insert(field(&app, "Ссылка_На_Сайт"), "https://example.com".into());
    let inputs = fixture.0.join("test-inputs");
    std::fs::create_dir_all(&inputs).unwrap();
    let photo = inputs.join("sample.png");
    image::RgbaImage::from_pixel(8, 8, image::Rgba([46, 107, 255, 255]))
        .save(&photo)
        .unwrap();
    let doc = inputs.join("document.txt");
    std::fs::write(&doc, "Проверочный документ\nUTF-8").unwrap();
    let photo_rel = assets::import_file(app.db().unwrap(), &photo, FieldKind::Image).unwrap();
    let doc_rel = assets::import_file(app.db().unwrap(), &doc, FieldKind::File).unwrap();
    app.selected
        .values
        .insert(field(&app, "Фотография"), photo_rel.clone());
    app.selected
        .values
        .insert(field(&app, "Документ"), doc_rel.clone());
    app.dirty = true;
    assert!(app.save_record(), "{}", app.error);
    let id = app.selected.id;
    assert!(assets::photo_texture(&ctx, app.db().unwrap(), &photo_rel).is_ok());
    let resolved = assets::resolve_attachment(app.db().unwrap(), &doc_rel).unwrap();
    assert_eq!(
        std::fs::read_to_string(resolved).unwrap(),
        "Проверочный документ\nUTF-8"
    );
    app.begin_new_field();
    app.field_editor.name = "Любимый предмет".into();
    app.field_editor.default_value = "История".into();
    app.save_field();
    let custom = field(&app, "Любимый предмет");
    assert_eq!(app.selected.value(custom), "История");
    app.selected.values.insert(custom, "Математика".into());
    app.dirty = true;
    assert!(app.save_record());
    app.search = "пОЛНЫЙ цИКЛ".into();
    app.refresh_search();
    assert_eq!(app.search_results.len(), 1);
    let csv = fixture.0.join("export.csv");
    assert_eq!(
        app.db().unwrap().export_csv(student_table, &csv).unwrap(),
        1
    );
    assert!(std::fs::read_to_string(csv).unwrap().contains("Математика"));
    app.request(Action::Backup);
    assert!(app.error.is_empty(), "{}", app.error);
    let backup = app.backups[0].clone();
    app.selected.values.insert(custom, "После копии".into());
    app.dirty = true;
    assert!(app.save_record());
    app.request(Action::Restore(backup));
    assert!(app.error.is_empty(), "{}", app.error);
    app.open_record_without_guard(id);
    assert_eq!(app.selected.value(custom), "Математика");
    let other = app.db().unwrap().create_table("Другая таблица").unwrap();
    app.refresh_tables();
    app.request(Action::SelectTable(other));
    app.begin_new_field();
    app.field_editor.name = "Название".into();
    app.save_field();
    app.request(Action::NewRecord);
    app.selected
        .values
        .insert(field(&app, "Название"), "Тест".into());
    assert!(app.save_record());
    let other_id = app.selected.id;
    app.db().unwrap().delete_record(other, other_id).unwrap();
    assert!(app.db().unwrap().record_ids(other, "").unwrap().is_empty());
    app.request(Action::SelectTable(student_table));
    let mut hidden = app.fields.iter().find(|f| f.id == custom).unwrap().clone();
    hidden.visible = false;
    app.db().unwrap().update_field(&hidden).unwrap();
    app.refresh_data();
    assert!(!app.fields.iter().find(|f| f.id == custom).unwrap().visible);
    drop(app);
    let mut reopened = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    reopened.open_record_without_guard(id);
    assert_eq!(reopened.selected.value(custom), "Математика");
    assert_eq!(
        reopened.selected.value(field(&reopened, "Документ")),
        doc_rel
    );
    assert!(assets::resolve_attachment(reopened.db().unwrap(), &photo_rel).is_ok());
    reopened
        .db()
        .unwrap()
        .delete_field(custom, student_table)
        .unwrap();
    reopened
        .db()
        .unwrap()
        .delete_record(student_table, id)
        .unwrap();
    assert!(reopened
        .db()
        .unwrap()
        .record_ids(student_table, "")
        .unwrap()
        .is_empty());
    Database::quick_check(&reopened.db().unwrap().path).unwrap();
}

#[test]
fn resizing_compacts_menu_without_overwriting_user_preference() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    for (width, expected) in [
        (1360.0, Sidebar::Full),
        (1120.0, Sidebar::Icons),
        (1190.0, Sidebar::Icons),
        (1240.0, Sidebar::Full),
    ] {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 800.0),
                )),
                ..Default::default()
            },
            |ctx| app.update_responsive_shell(ctx),
        );
        assert_eq!(app.effective_sidebar(), expected);
        assert_eq!(app.prefs.sidebar, Sidebar::Full);
    }
    app.prefs.sidebar = Sidebar::Hidden;
    assert_eq!(app.effective_sidebar(), Sidebar::Hidden);
}

#[test]
fn search_controls_leave_results_visible_without_vertical_stretching() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    app.screen = Screen::Search;
    for width in [1000.0, 1360.0] {
        for _ in 0..2 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 760.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    app.update_responsive_shell(ctx);
                    app.topbar(ctx);
                    app.statusbar(ctx);
                    app.sidebar(ctx);
                    app.main_area(ctx);
                },
            );
            let results = out
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if text.galley.text().starts_with("Результаты поиска") =>
                    {
                        Some(text)
                    }
                    _ => None,
                })
                .expect("search results heading must remain on screen");
            assert!(
                results.pos.y < 440.0,
                "controls pushed results below the fold: {}",
                results.pos.y
            );
        }
    }
}
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("kartoteka-ui-{}", uuid::Uuid::new_v4())))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn all_nine_screens_render_at_supported_sizes_and_menu_modes() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    assert!(app.error.is_empty(), "{}", app.error);
    for populated in [false, true] {
        if populated {
            for name in ["Фамилия", "Имя"] {
                let id = app.fields.iter().find(|f| f.name == name).unwrap().id;
                app.selected
                    .values
                    .insert(id, "Проверка длинной русской подписи".into());
            }
            assert!(app.save_record());
        }
        // The last case exercises the logical space of a 900 px window at 150% app zoom.
        for (width, height) in [
            (1360.0, 860.0),
            (1120.0, 720.0),
            (900.0, 620.0),
            (600.0, 413.0),
        ] {
            for menu in [Sidebar::Full, Sidebar::Icons, Sidebar::Hidden] {
                app.prefs.sidebar = menu;
                for screen in Screen::ALL {
                    app.screen = screen;
                    for _ in 0..2 {
                        // egui performs a sizing pass before a stable layout.
                        let out = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, height),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                app.topbar(ctx);
                                app.statusbar(ctx);
                                app.sidebar(ctx);
                                app.main_area(ctx);
                                app.dialogs(ctx);
                            },
                        );
                        assert!(!out.shapes.is_empty(), "{screen:?}");
                        for shape in out.shapes {
                            if let egui::Shape::Text(text) = &shape.shape {
                                if text.galley.rows.len() > 1 {
                                    assert!(
                                        !text.galley.job.justify,
                                        "{screen:?}: wrapped text must keep natural spacing: {}",
                                        text.galley.text()
                                    );
                                }
                            }
                            assert!(
                                shape.clip_rect.is_finite(),
                                "{screen:?} invalid clipping region"
                            );
                        }
                        assert!(app.error.is_empty(), "{screen:?}: {}", app.error);
                    }
                }
            }
        }
    }
}

#[test]
fn navigation_keeps_unsaved_card_and_requires_explicit_resolution() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    app.screen = Screen::Card;
    app.dirty = true;
    app.request(Action::Screen(Screen::Table));
    assert_eq!(app.screen, Screen::Card);
    assert!(app.pending.is_some());
    assert!(app.dirty);
    app.discard_changes();
    let action = app.pending.take().unwrap();
    app.apply_action(action);
    assert_eq!(app.screen, Screen::Table);
    assert!(!app.dirty);
}

#[test]
fn shortcuts_use_event_modifiers_and_physical_keys() {
    let ctx = egui::Context::default();
    let _ = ctx.run(
        egui::RawInput {
            modifiers: egui::Modifiers::NONE,
            events: vec![egui::Event::Key {
                key: egui::Key::A,
                physical_key: Some(egui::Key::N),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            }],
            ..Default::default()
        },
        |ctx| {
            ctx.input(|input| {
                assert!(shortcut_pressed(input, egui::Key::N));
                assert!(!shortcut_pressed(input, egui::Key::S));
            })
        },
    );
}

#[test]
fn numbers_are_centered_by_visible_ink_at_fractional_display_scales() {
    for pixels in [1.0_f32, 1.25, 1.5, 2.0] {
        let ctx = egui::Context::default();
        configure_style(&ctx, 1.0);
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 200.0),
        ));
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(pixels);
        let center = egui::pos2(53.5, 61.25);
        let output = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                paint_centered_text(
                    ui.painter(),
                    center,
                    "2",
                    egui::FontId::proportional(15.0),
                    egui::Color32::WHITE,
                );
            });
        });
        let text = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.text() == "2" => Some(t),
                _ => None,
            })
            .unwrap();
        let actual = text.pos + text.galley.mesh_bounds.center().to_vec2();
        assert!(
            actual.distance(center) * pixels <= 0.72,
            "visible number drift at {pixels}: {actual:?}"
        );
    }
}

#[test]
fn wrapped_card_text_keeps_natural_letter_spacing_inside_columns() {
    let ctx = egui::Context::default();
    configure_style(&ctx, 1.0);
    let label = "C:/Users/test/very-long-unbroken-folder-name-for-card-wrapping/database.db";
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600.0, 300.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(2, |columns| {
                    card(&mut columns[0], |ui| {
                        ui.label(label);
                    });
                });
            });
        },
    );
    let text = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == label => Some(t),
            _ => None,
        })
        .unwrap();
    assert!(text.galley.rows.len() > 1, "fixture must wrap");
    assert!(
        !text.galley.job.justify,
        "wrapped letters must not be expanded to fill the card"
    );
}

#[test]
fn switching_languages_preserves_database_and_persists_selection() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    let before = std::fs::read(&app.db().unwrap().path).unwrap();
    let schema = app
        .fields
        .iter()
        .map(|f| (f.id, f.name.clone()))
        .collect::<Vec<_>>();
    for language in crate::i18n::Language::ALL {
        app.prefs.language = language;
        crate::i18n::set(language);
        app.save_preferences();
        let stored: Preferences =
            serde_json::from_slice(&std::fs::read(fixture.0.join("preferences.json")).unwrap())
                .unwrap();
        assert_eq!(stored.language, language);
        assert_eq!(std::fs::read(&app.db().unwrap().path).unwrap(), before);
        assert_eq!(
            schema,
            app.fields
                .iter()
                .map(|f| (f.id, f.name.clone()))
                .collect::<Vec<_>>()
        );
        assert_eq!(field_label("Custom API"), "Custom API");
        if language != crate::i18n::Language::Ru {
            assert!(!field_label("Адрес_Проживания")
                .chars()
                .any(|c| ('А'..='я').contains(&c)));
            assert!(!crate::i18n::tr("Сохранить")
                .chars()
                .any(|c| ('А'..='я').contains(&c)));
            assert_eq!(record_word(2), record_word(12));
            assert_ne!(record_word(1), record_word(2));
        }
    }
    crate::i18n::set(crate::i18n::Language::Ru);
}
#[test]
fn all_localized_screens_render_without_changing_user_data() {
    let fixture = Fixture::new();
    let ctx = egui::Context::default();
    let mut app = Kartoteka::open(&ctx, fixture.0.clone(), Preferences::default());
    let before = std::fs::read(&app.db().unwrap().path).unwrap();
    for language in crate::i18n::Language::ALL {
        app.prefs.language = language;
        crate::i18n::set(language);
        for width in [1000.0, 1360.0, 1920.0] {
            for screen in Screen::ALL {
                app.screen = screen;
                for _ in 0..2 {
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 860.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            app.update_responsive_shell(ctx);
                            app.topbar(ctx);
                            app.statusbar(ctx);
                            app.sidebar(ctx);
                            app.main_area(ctx);
                            app.dialogs(ctx);
                        },
                    );
                    assert!(!output.shapes.is_empty());
                }
            }
        }
    }
    assert_eq!(before, std::fs::read(&app.db().unwrap().path).unwrap());
    crate::i18n::set(crate::i18n::Language::Ru);
}
#[test]
fn localization_catalog_has_complete_translations_and_matching_placeholders() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../locales/ui.json")).unwrap();
    fn placeholders(s: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut remaining = s;
        while let Some(start) = remaining.find('{') {
            remaining = &remaining[start + 1..];
            let end = remaining.find('}').unwrap();
            result.push(remaining[..end].to_owned());
            remaining = &remaining[end + 1..];
        }
        result.sort();
        result
    }
    for (ru, translations) in catalog.as_object().unwrap() {
        for language in ["de", "en"] {
            let text = translations[language].as_str().unwrap();
            assert!(!text.is_empty());
            assert_eq!(placeholders(ru), placeholders(text), "{ru}/{language}");
        }
    }
    let old: Preferences = serde_json::from_str(r#"{"sidebar":"Full","zoom":1.0}"#).unwrap();
    assert_eq!(old.language, crate::i18n::Language::Ru);
}

#[test]
fn field_types_and_validation_messages_follow_language() {
    for language in [crate::i18n::Language::De, crate::i18n::Language::En] {
        crate::i18n::set(language);
        for kind in FieldKind::ALL {
            assert!(!kind.label().chars().any(|c| ('А'..='я').contains(&c)));
        }
        for (kind, value) in [
            (FieldKind::Date, "wrong"),
            (FieldKind::Integer, "abc"),
            (FieldKind::Url, "bad"),
        ] {
            let error = kind.validate(value).unwrap_err();
            assert!(!error.chars().any(|c| ('А'..='я').contains(&c)), "{error}");
        }
    }
    crate::i18n::set(crate::i18n::Language::Ru);
}

#[test]
fn language_translations_preserve_color_literals() {
    for language in crate::i18n::Language::ALL {
        crate::i18n::set(language);
        assert!(crate::i18n::tr("Шрифт: Segoe UI · синий акцент #2E6BFF").contains("#2E6BFF"));
    }
    crate::i18n::set(crate::i18n::Language::Ru);
}
