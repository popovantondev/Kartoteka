//! Главный интерфейс приложения. Все экраны используют единый светлый стиль.
//! Сложные операции вынесены в «Конструктор» и «Настройки».

use crate::assets;
use crate::db::{Database, FieldDef, FieldKind, Record, TableDef};
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

mod theme;
use theme::*;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Home,
    Card,
    Table,
    Search,
    Designer,
    Forms,
    Backups,
    Settings,
    Help,
}
impl Screen {
    const ALL: [Self; 9] = [
        Self::Home,
        Self::Card,
        Self::Table,
        Self::Search,
        Self::Designer,
        Self::Forms,
        Self::Backups,
        Self::Settings,
        Self::Help,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Home => crate::i18n::tr("Главная"),
            Self::Card => crate::i18n::tr("Карточка"),
            Self::Table => crate::i18n::tr("Таблица"),
            Self::Search => crate::i18n::tr("Поиск"),
            Self::Designer => crate::i18n::tr("Конструктор"),
            Self::Forms => crate::i18n::tr("Формы"),
            Self::Backups => crate::i18n::tr("Резервные копии"),
            Self::Settings => crate::i18n::tr("Настройки"),
            Self::Help => crate::i18n::tr("Справка"),
        }
    }
    fn glyph(self) -> Glyph {
        match self {
            Self::Home => Glyph::Home,
            Self::Card => Glyph::Person,
            Self::Table => Glyph::Table,
            Self::Search => Glyph::Search,
            Self::Designer => Glyph::Tools,
            Self::Forms => Glyph::Form,
            Self::Backups => Glyph::Backup,
            Self::Settings => Glyph::Settings,
            Self::Help => Glyph::Help,
        }
    }
}
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Sidebar {
    Full,
    Icons,
    Hidden,
}
#[derive(Serialize, Deserialize)]
struct Preferences {
    #[serde(default)]
    language: crate::i18n::Language,
    sidebar: Sidebar,
    zoom: f32,
    #[serde(default)]
    column_widths: HashMap<String, Vec<f32>>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: crate::i18n::Language::Ru,
            sidebar: Sidebar::Full,
            zoom: 1.0,
            column_widths: HashMap::new(),
        }
    }
}

#[derive(Clone)]
enum Action {
    Screen(Screen),
    OpenRecord(i64),
    NewRecord,
    First,
    Previous,
    Next,
    Last,
    RefreshData,
    SelectTable(i64),
    Backup,
    Restore(PathBuf),
    Exit,
}

pub struct Kartoteka {
    db: Option<Database>,
    startup_error: String,
    prefs: Preferences,
    screen: Screen,
    tables: Vec<TableDef>,
    active_table: i64,
    fields: Vec<FieldDef>,
    ids: Vec<i64>,
    table_records: Vec<Record>,
    selected: Record,
    dirty: bool,
    status: String,
    error: String,
    search: String,
    search_last: String,
    search_table: i64,
    search_results: Vec<Record>,
    pending: Option<Action>,
    delete_record_confirm: bool,
    delete_field_confirm: bool,
    restore_confirm: Option<PathBuf>,
    add_field_dialog: bool,
    field_editor: FieldDef,
    selected_field: Option<i64>,
    create_table_dialog: bool,
    new_table_name: String,
    photo_cache: HashMap<String, egui::TextureHandle>,
    backups: Vec<PathBuf>,
    table_sort_field: Option<i64>,
    table_sort_desc: bool,
    table_filter: String,
    selected_row: Option<i64>,
    focus_search: bool,
    help_topic: usize,
    settings_topic: usize,
    auto_compact: bool,
    sidebar_pointer_drag: Option<(Sidebar, egui::Pos2)>,
}
impl Kartoteka {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let root = assets::default_data_dir();
        let prefs = std::fs::read(root.join("preferences.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Preferences>(&bytes).ok())
            .unwrap_or_default();
        Self::open(&cc.egui_ctx, root, prefs)
    }
    fn open(ctx: &egui::Context, root: PathBuf, prefs: Preferences) -> Self {
        crate::i18n::set(prefs.language);
        configure_style(ctx, prefs.zoom);
        let result = Database::new(root);
        let (db, startup_error) = match result {
            Ok(db) => (Some(db), String::new()),
            Err(e) => (None, e),
        };
        let mut app = Self {
            db,
            startup_error,
            prefs,
            screen: Screen::Home,
            tables: vec![],
            active_table: 0,
            fields: vec![],
            ids: vec![],
            table_records: vec![],
            selected: Record::new(0, &[]),
            dirty: false,
            status: crate::i18n::tr("Готово").into(),
            error: String::new(),
            search: String::new(),
            search_last: "\u{0}".into(),
            search_table: 0,
            search_results: vec![],
            pending: None,
            delete_record_confirm: false,
            delete_field_confirm: false,
            restore_confirm: None,
            add_field_dialog: false,
            field_editor: FieldDef::blank(0),
            selected_field: None,
            create_table_dialog: false,
            new_table_name: String::new(),
            photo_cache: HashMap::new(),
            backups: vec![],
            table_sort_field: None,
            table_sort_desc: false,
            table_filter: String::new(),
            selected_row: None,
            focus_search: false,
            help_topic: 0,
            settings_topic: 0,
            auto_compact: false,
            sidebar_pointer_drag: None,
        };
        app.refresh_tables();
        app.refresh_backups();
        app
    }
    fn db(&self) -> Result<&Database, String> {
        self.db.as_ref().ok_or_else(|| self.startup_error.clone())
    }
    fn fail(&mut self, error: String) {
        self.error = error;
    }
    fn notify(&mut self, message: impl Into<String>) {
        self.error.clear();
        self.status = message.into();
    }
    fn save_preferences(&self) {
        if let Some(db) = &self.db {
            if let Ok(bytes) = serde_json::to_vec_pretty(&self.prefs) {
                let _ = std::fs::write(db.root.join("preferences.json"), bytes);
            }
        }
    }
    fn refresh_tables(&mut self) {
        match self.db().and_then(Database::tables) {
            Ok(tables) => {
                self.tables = tables;
                if !self.tables.iter().any(|t| t.id == self.active_table) {
                    self.active_table = self.tables.first().map(|t| t.id).unwrap_or(0);
                }
                self.refresh_data();
            }
            Err(e) => self.fail(e),
        }
    }
    fn refresh_data(&mut self) {
        if self.active_table == 0 {
            return;
        }
        let output = (|| {
            let db = self.db()?;
            let fields = db.fields(self.active_table)?;
            let ids = db.record_ids(self.active_table, "")?;
            let records = db.search_records(self.active_table, "", 2000)?;
            Ok::<_, String>((fields, ids, records))
        })();
        match output {
            Ok((fields, ids, records)) => {
                self.fields = fields;
                self.ids = ids;
                self.table_records = records;
                self.search_last = "\u{0}".into();
                let id = self.selected.id;
                if id > 0 && self.ids.contains(&id) {
                    self.open_record_without_guard(id);
                } else if let Some(id) = self.ids.first().copied() {
                    self.open_record_without_guard(id);
                } else {
                    self.selected = Record::new(self.active_table, &self.fields);
                    self.dirty = false;
                }
            }
            Err(e) => self.fail(e),
        }
    }
    fn open_record_without_guard(&mut self, id: i64) {
        let result = self.db().and_then(|db| db.record(self.active_table, id));
        match result {
            Ok(record) => {
                self.selected = record;
                self.dirty = false;
            }
            Err(e) => self.fail(e),
        }
    }
    fn new_record_without_guard(&mut self) {
        self.selected = Record::new(self.active_table, &self.fields);
        self.dirty = false;
        self.screen = Screen::Card;
    }
    fn discard_changes(&mut self) {
        if self.selected.id > 0 {
            self.open_record_without_guard(self.selected.id);
        } else {
            self.selected = Record::new(self.active_table, &self.fields);
        }
        self.dirty = false;
    }
    fn request(&mut self, action: Action) {
        if self.dirty {
            self.pending = Some(action);
        } else {
            self.apply_action(action);
        }
    }
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Screen(screen) => self.screen = screen,
            Action::OpenRecord(id) => {
                self.open_record_without_guard(id);
                self.screen = Screen::Card;
            }
            Action::NewRecord => self.new_record_without_guard(),
            Action::RefreshData => {
                self.refresh_data();
                self.notify(crate::i18n::tr("Данные обновлены"));
            }
            Action::First => {
                if let Some(&id) = self.ids.first() {
                    self.open_record_without_guard(id)
                }
            }
            Action::Last => {
                if let Some(&id) = self.ids.last() {
                    self.open_record_without_guard(id)
                }
            }
            Action::Previous | Action::Next => {
                if let Some(index) = self.ids.iter().position(|id| *id == self.selected.id) {
                    let target = if matches!(action, Action::Previous) {
                        index.checked_sub(1)
                    } else if index + 1 < self.ids.len() {
                        Some(index + 1)
                    } else {
                        None
                    };
                    if let Some(index) = target {
                        self.open_record_without_guard(self.ids[index]);
                    }
                }
            }
            Action::SelectTable(id) => {
                self.active_table = id;
                self.selected = Record::new(id, &[]);
                self.table_sort_field = None;
                self.table_sort_desc = false;
                self.selected_row = None;
                self.selected_field = None;
                self.dirty = false;
                self.refresh_data();
            }
            Action::Backup => self.do_backup(),
            Action::Restore(path) => self.do_restore(path),
            Action::Exit => self.notify(crate::i18n::tr(
                "Закройте окно приложения, чтобы завершить работу",
            )),
        }
    }
    fn save_record(&mut self) -> bool {
        let draft = self.selected.clone();
        let result = self.db().and_then(|db| db.save_record(&draft));
        match result {
            Ok(id) => {
                self.dirty = false;
                self.selected.id = id;
                self.refresh_data();
                self.open_record_without_guard(id);
                self.notify(match crate::i18n::current() {
                    crate::i18n::Language::Ru => format!("Запись № {id} сохранена"),
                    crate::i18n::Language::De => format!("Datensatz Nr. {id} gespeichert"),
                    crate::i18n::Language::En => format!("Record no. {id} saved"),
                });
                true
            }
            Err(e) => {
                self.fail(e);
                false
            }
        }
    }
    fn refresh_backups(&mut self) {
        match self.db().and_then(assets::list_backups) {
            Ok(backups) => self.backups = backups,
            Err(e) => self.fail(e),
        }
    }
    fn do_backup(&mut self) {
        match self.db().and_then(assets::create_backup) {
            Ok(path) => {
                self.notify(match crate::i18n::current() {
                    crate::i18n::Language::Ru => {
                        format!("Резервная копия создана: {}", path.display())
                    }
                    crate::i18n::Language::De => format!("Sicherung erstellt: {}", path.display()),
                    crate::i18n::Language::En => format!("Backup created: {}", path.display()),
                });
                self.refresh_backups();
            }
            Err(e) => self.fail(e),
        }
    }
    fn do_restore(&mut self, path: PathBuf) {
        match self.db().and_then(|db| assets::restore_backup(db, &path)) {
            Ok(()) => {
                self.photo_cache.clear();
                self.selected = Record::new(0, &[]);
                self.dirty = false;
                self.refresh_tables();
                self.refresh_backups();
                self.notify(crate::i18n::tr(
                    "База и вложения восстановлены. Перед восстановлением создана защитная копия.",
                ));
            }
            Err(e) => self.fail(e),
        }
    }
    fn table_name(&self) -> &str {
        self.tables
            .iter()
            .find(|t| t.id == self.active_table)
            .map(|t| t.name.as_str())
            .unwrap_or(crate::i18n::tr("Новая база"))
    }
    fn table_title(&self) -> &str {
        let name = self.table_name();
        if self.active_table == 1 {
            crate::i18n::tr(name)
        } else {
            name
        }
    }
    fn visible_name(name: &str) -> String {
        field_label(name)
    }
    fn name_of_record(&self, record: &Record) -> String {
        let mut name = Vec::new();
        for field_name in ["Фамилия", "Имя", "Отчество"] {
            if let Some(field) = self.fields.iter().find(|f| f.name == field_name) {
                let value = record.value(field.id);
                if !value.is_empty() {
                    name.push(value.to_owned());
                }
            }
        }
        if name.is_empty() {
            match crate::i18n::current() {
                crate::i18n::Language::Ru => format!("Запись № {}", record.id),
                crate::i18n::Language::De => format!("Datensatz Nr. {}", record.id),
                crate::i18n::Language::En => format!("Record no. {}", record.id),
            }
        } else {
            name.join(" ")
        }
    }
    fn current_position(&self) -> usize {
        self.ids
            .iter()
            .position(|id| *id == self.selected.id)
            .map(|i| i + 1)
            .unwrap_or(0)
    }
    fn topbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar")
            .exact_height(80.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ctx, |ui| {
                let total = ui.available_width();
                ui.horizontal_centered(|ui| {
                    let (brand_rect, brand_response) =
                        ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
                    ui.painter().rect_filled(brand_rect, 14.0, BLUE);
                    draw_glyph(
                        ui.painter(),
                        brand_rect.shrink(7.0),
                        Glyph::Brand,
                        Color32::WHITE,
                    );
                    if brand_response
                        .on_hover_text(crate::i18n::tr("Полное меню / иконки / скрыто"))
                        .clicked()
                    {
                        self.cycle_sidebar();
                    }
                    if total > 780.0 {
                        ui.label(
                            RichText::new(if total > 1030.0 {
                                crate::i18n::tr("Одноклассники — Картотека")
                            } else {
                                crate::i18n::tr("Картотека")
                            })
                            .size(16.0)
                            .strong()
                            .color(INK),
                        );
                    }
                    if total > 1150.0 {
                        ui.label(RichText::new("/").color(BORDER));
                        ui.label(RichText::new(crate::i18n::tr(self.table_title())).color(MUTED));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if action_button(ui, Glyph::Plus, crate::i18n::tr("Новая запись"), true)
                            .clicked()
                        {
                            self.request(Action::NewRecord);
                        }
                        ui.add_space(8.0);
                        let response = search_input(
                            ui,
                            "global-search",
                            &mut self.search,
                            (total * 0.24).clamp(155.0, 300.0),
                            crate::i18n::tr("Поиск по записям"),
                        );
                        if response.changed() && self.screen != Screen::Search {
                            self.request(Action::Screen(Screen::Search));
                        }
                        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.request(Action::Screen(Screen::Search));
                        }
                    });
                });
                let r = ui.max_rect();
                ui.painter().hline(
                    r.x_range(),
                    r.bottom() + 12.0,
                    egui::Stroke::new(1.0_f32, BORDER),
                );
            });
    }
    fn cycle_sidebar(&mut self) {
        self.prefs.sidebar = match self.effective_sidebar() {
            Sidebar::Full => Sidebar::Icons,
            Sidebar::Icons => Sidebar::Hidden,
            Sidebar::Hidden => Sidebar::Full,
        };
        self.save_preferences();
    }
    fn remember_widths(&mut self, key: String, widths: Vec<f32>) {
        if widths.iter().all(|v| v.is_finite() && *v >= 30.0)
            && self.prefs.column_widths.get(&key).is_none_or(|old| {
                old.len() != widths.len()
                    || old.iter().zip(&widths).any(|(a, b)| (a - b).abs() > 0.5)
            })
        {
            self.prefs.column_widths.insert(key, widths);
            self.save_preferences();
        }
    }
    fn effective_sidebar(&self) -> Sidebar {
        if self.prefs.sidebar == Sidebar::Full && self.auto_compact {
            Sidebar::Icons
        } else {
            self.prefs.sidebar
        }
    }
    fn update_responsive_shell(&mut self, ctx: &egui::Context) {
        let width = ctx.screen_rect().width();
        if width < 1160.0 {
            self.auto_compact = true;
        } else if width >= 1240.0 {
            self.auto_compact = false;
        }
        // Viewport commands use egui points; keep the native minimum independent
        // of the user's application zoom. At 150% the content reflows instead.
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
            egui::vec2(1000.0, 700.0) / ctx.zoom_factor(),
        ));
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        self.sidebar_pointer_events(ctx);
        if self.effective_sidebar() == Sidebar::Hidden {
            // A small edge target remains available when the panel is hidden.
            egui::Area::new(egui::Id::new("hidden-sidebar-edge"))
                .fixed_pos(egui::pos2(0.0, 80.0))
                .show(ctx, |ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(12.0, (ctx.screen_rect().height() - 126.0).max(80.0)),
                        egui::Sense::hover(),
                    );
                    let response = ui.interact(
                        rect,
                        egui::Id::new("sidebar-divider-gesture"),
                        egui::Sense::click_and_drag(),
                    );
                    if response.hovered() || response.dragged() {
                        ui.painter().vline(
                            rect.center().x,
                            rect.y_range(),
                            egui::Stroke::new(2.0_f32, BLUE),
                        );
                    }
                    self.sidebar_gesture(response);
                });
            return;
        }
        let collapsed = self.effective_sidebar() == Sidebar::Icons;
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(if collapsed { 64.0 } else { 208.0 })
            .frame(
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(if collapsed { 8 } else { 12 }, 16)),
            )
            .show(ctx, |ui| {
                let area = ui.max_rect();
                ui.painter().vline(
                    area.right() + 12.0,
                    area.y_range(),
                    egui::Stroke::new(1.0_f32, BORDER),
                );
                let boundary = area.right() + if collapsed { 8.0 } else { 12.0 };
                let edge = egui::Rect::from_min_max(
                    egui::pos2(boundary - 10.0, area.top()),
                    egui::pos2(boundary, area.bottom()),
                );
                let toggle = ui.interact(
                    edge,
                    egui::Id::new("sidebar-divider-gesture"),
                    egui::Sense::click_and_drag(),
                );
                if toggle.hovered() {
                    ui.painter().vline(
                        edge.center().x,
                        edge.y_range(),
                        egui::Stroke::new(2.0_f32, BLUE),
                    );
                }
                self.sidebar_gesture(toggle);
                egui::ScrollArea::vertical()
                    .max_height((ui.available_height() - 52.0).max(50.0))
                    .id_salt("sidebar-scroll")
                    .show(ui, |ui| {
                        for screen in Screen::ALL {
                            if screen == Screen::Designer {
                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(12.0);
                            }
                            if nav_item(
                                ui,
                                screen.glyph(),
                                screen.label(),
                                self.screen == screen,
                                collapsed,
                            )
                            .clicked()
                            {
                                self.request(Action::Screen(screen));
                            }
                            ui.add_space(4.0);
                        }
                    });
                if !collapsed {
                    let r = egui::Rect::from_min_size(
                        egui::pos2(area.left(), area.bottom() - 32.0),
                        egui::vec2(area.width(), 28.0),
                    );
                    let label = crate::i18n::tr("Работает офлайн");
                    let galley = ui.painter().layout_no_wrap(
                        label.into(),
                        egui::FontId::proportional(14.0),
                        MUTED,
                    );
                    let start = r.center().x - (26.0 + galley.mesh_bounds.width()) / 2.0;
                    draw_glyph(
                        ui.painter(),
                        egui::Rect::from_center_size(
                            egui::pos2(start + 8.0, r.center().y),
                            egui::vec2(18.0, 18.0),
                        ),
                        Glyph::Shield,
                        GREEN,
                    );
                    paint_centered_text(
                        ui.painter(),
                        egui::pos2(
                            start + 26.0 + galley.mesh_bounds.width() / 2.0,
                            r.center().y,
                        ),
                        label,
                        egui::FontId::proportional(14.0),
                        MUTED,
                    );
                }
            });
    }
    fn sidebar_gesture(&mut self, response: egui::Response) {
        if response.double_clicked() {
            self.prefs.sidebar = if self.effective_sidebar() == Sidebar::Full {
                Sidebar::Icons
            } else {
                Sidebar::Full
            };
            self.save_preferences();
        }
        response
            .on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
            .on_hover_text(crate::i18n::tr(
                "Перетяните к краю или от края · двойной щелчок переключает меню",
            ));
    }
    fn sidebar_pointer_events(&mut self, ctx: &egui::Context) {
        // Native integrations may deliver a short press/move/release in one
        // frame. Process the ordered events, rather than only drag flags.
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    ..
                } => {
                    let mode = self.effective_sidebar();
                    let boundary: f32 = match mode {
                        Sidebar::Full => 208.0,
                        Sidebar::Icons => 64.0,
                        Sidebar::Hidden => 0.0,
                    };
                    if pos.y >= 80.0
                        && pos.y <= ctx.screen_rect().bottom() - 46.0
                        && pos.x >= (boundary - 12.0).max(0.0)
                        && pos.x <= boundary + 12.0
                    {
                        self.sidebar_pointer_drag = Some((mode, pos));
                    }
                }
                egui::Event::PointerMoved(pos) => {
                    if let Some((initial, start)) = self.sidebar_pointer_drag {
                        let mode = sidebar_after_drag(initial, pos.x - start.x);
                        if (pos.x - start.x).abs() >= 6.0 && self.prefs.sidebar != mode {
                            self.prefs.sidebar = mode;
                            self.save_preferences();
                        }
                    }
                }
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                } => {
                    if let Some((initial, start)) = self.sidebar_pointer_drag.take() {
                        let mode = sidebar_after_drag(initial, pos.x - start.x);
                        if (pos.x - start.x).abs() >= 6.0 && self.prefs.sidebar != mode {
                            self.prefs.sidebar = mode;
                            self.save_preferences();
                        }
                    }
                }
                egui::Event::PointerGone => self.sidebar_pointer_drag = None,
                _ => {}
            }
        }
    }

    fn statusbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(46.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                let bounds = ui.max_rect();
                let summary = format!(
                    "{}   ·   {} {}",
                    self.table_title(),
                    self.ids.len(),
                    record_word(self.ids.len())
                );
                let summary_galley = ui.painter().layout_no_wrap(
                    summary.clone(),
                    egui::FontId::proportional(14.0),
                    MUTED,
                );
                let reserve = summary_galley.size().x.min(bounds.width() * 0.48);
                let state = if self.dirty {
                    crate::i18n::tr("Есть несохранённые изменения")
                } else {
                    &self.status
                };
                let left = egui::Rect::from_min_max(
                    bounds.min,
                    egui::pos2(bounds.right() - reserve - 24.0, bounds.bottom()),
                );
                let text =
                    egui::WidgetText::from(RichText::new(state).size(14.0).color(if self.dirty {
                        DANGER
                    } else {
                        MUTED
                    }))
                    .into_galley(
                        ui,
                        Some(egui::TextWrapMode::Truncate),
                        left.width(),
                        egui::TextStyle::Body,
                    );
                ui.painter().galley(
                    egui::pos2(left.left(), bounds.center().y - text.size().y / 2.0),
                    text,
                    MUTED,
                );
                let right = egui::Rect::from_min_max(
                    egui::pos2(bounds.right() - reserve, bounds.top()),
                    bounds.max,
                );
                let text = egui::WidgetText::from(RichText::new(summary).size(14.0).color(MUTED))
                    .into_galley(
                        ui,
                        Some(egui::TextWrapMode::Truncate),
                        reserve,
                        egui::TextStyle::Body,
                    );
                ui.painter().galley(
                    egui::pos2(
                        right.right() - text.size().x,
                        bounds.center().y - text.size().y / 2.0,
                    ),
                    text,
                    MUTED,
                );
                ui.interact(left, ui.id().with("status-tip"), egui::Sense::hover())
                    .on_hover_text(state);
            });
    }
    fn main_area(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(24)),
            )
            .show(ctx, |ui| {
                if !self.error.is_empty() {
                    egui::Frame::new()
                        .fill(RED_SOFT)
                        .corner_radius(14)
                        .inner_margin(14)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.colored_label(
                                    DANGER,
                                    match crate::i18n::current() {
                                        crate::i18n::Language::Ru => {
                                            format!("Не удалось выполнить действие: {}", self.error)
                                        }
                                        crate::i18n::Language::De => {
                                            format!("Aktion fehlgeschlagen: {}", self.error)
                                        }
                                        crate::i18n::Language::En => {
                                            format!("Action failed: {}", self.error)
                                        }
                                    },
                                );
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Закрыть")))
                                    .clicked()
                                {
                                    self.error.clear();
                                }
                            });
                        });
                    ui.add_space(12.0);
                }
                match self.screen {
                    Screen::Card => self.card_screen(ui, ctx),
                    Screen::Table => self.table_screen(ui),
                    Screen::Search => self.search_screen(ui),
                    screen => {
                        egui::ScrollArea::vertical()
                            .id_salt(("page", screen.label()))
                            .auto_shrink([false, false])
                            .show(ui, |ui| match screen {
                                Screen::Home => self.home_screen(ui),
                                Screen::Designer => self.designer_screen(ui),
                                Screen::Forms => self.forms_screen(ui),
                                Screen::Backups => self.backups_screen(ui),
                                Screen::Settings => self.settings_screen(ui, ctx),
                                Screen::Help => self.help_screen(ui),
                                _ => {}
                            });
                    }
                }
            });
    }
    fn home_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Моя база данных"),
            crate::i18n::tr("Выберите действие, чтобы продолжить работу"),
        );
        ui.add_space(24.0);
        let items = [
            (
                Glyph::Folder,
                crate::i18n::tr("Открыть картотеку"),
                crate::i18n::tr("Продолжить работу с карточками"),
                Action::Screen(Screen::Card),
            ),
            (
                Glyph::Plus,
                crate::i18n::tr("Новая запись"),
                crate::i18n::tr("Добавить одноклассника"),
                Action::NewRecord,
            ),
            (
                Glyph::Table,
                crate::i18n::tr("Таблица"),
                crate::i18n::tr("Все записи в одном месте"),
                Action::Screen(Screen::Table),
            ),
            (
                Glyph::Tools,
                crate::i18n::tr("Конструктор"),
                crate::i18n::tr("Настроить таблицы и поля"),
                Action::Screen(Screen::Designer),
            ),
        ];
        let count = if ui.available_width() < 520.0 {
            1
        } else if ui.available_width() < 780.0 {
            2
        } else {
            4
        };
        let mut chosen = None;
        for chunk in items.chunks(count) {
            ui.columns(count, |columns| {
                for (i, (glyph, label, sub, action)) in chunk.iter().enumerate() {
                    card(&mut columns[i], |ui| {
                        ui.set_min_height(172.0);
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            icon_badge(ui, *glyph, 48.0, SOFT_BLUE, BLUE);
                            ui.add_space(10.0);
                            ui.label(RichText::new(*label).size(16.0).strong().color(INK));
                            centered_caption(ui, sub, ui.available_width(), 44.0);
                            ui.add_space(8.0);
                            if ui
                                .add_sized(
                                    [ui.available_width(), 38.0],
                                    secondary_button(
                                        if *label == crate::i18n::tr("Новая запись") {
                                            crate::i18n::tr("Создать запись")
                                        } else {
                                            crate::i18n::tr("Открыть")
                                        },
                                    ),
                                )
                                .clicked()
                            {
                                chosen = Some(action.clone());
                            }
                        });
                    });
                }
            });
            ui.add_space(16.0);
        }
        let database_panel = |ui: &mut egui::Ui| {
            card(ui, |ui| {
                ui.set_min_height(156.0);
                section_title(ui, Glyph::Database, crate::i18n::tr("Текущая база данных"));
                ui.label(
                    RichText::new(self.table_title())
                        .size(14.0)
                        .strong()
                        .color(INK),
                );
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.ids.len().to_string())
                            .size(28.0)
                            .strong()
                            .color(INK)
                            .color(BLUE),
                    );
                    ui.label(RichText::new(record_word(self.ids.len())).color(MUTED));
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(self.fields.len().to_string())
                            .size(28.0)
                            .strong()
                            .color(INK),
                    );
                    ui.label(RichText::new(field_word(self.fields.len())).color(MUTED));
                });
                ui.label(
                    RichText::new(crate::i18n::tr("Все данные хранятся на этом компьютере"))
                        .size(14.0)
                        .color(MUTED),
                );
            });
        };
        let mut backup_action = false;
        let mut backup_panel = |ui: &mut egui::Ui| {
            card(ui, |ui| {
                ui.set_min_height(156.0);
                section_title(ui, Glyph::Shield, crate::i18n::tr("Безопасность данных"));
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        self.backups.len(),
                        copy_word(self.backups.len())
                    ))
                    .strong()
                    .color(INK),
                );
                ui.label(
                    RichText::new(crate::i18n::tr(
                        "Копия включает базу, фотографии и документы.",
                    ))
                    .size(14.0)
                    .color(MUTED),
                );
                ui.add_space(4.0);
                if ui
                    .add(secondary_button(crate::i18n::tr("Открыть резервные копии")))
                    .clicked()
                {
                    backup_action = true;
                }
            });
        };
        if ui.available_width() < 600.0 {
            database_panel(ui);
            ui.add_space(16.0);
            backup_panel(ui);
        } else {
            ui.columns(2, |columns| {
                database_panel(&mut columns[0]);
                backup_panel(&mut columns[1]);
            });
        }
        if backup_action {
            chosen = Some(Action::Screen(Screen::Backups));
        }
        if self.ids.is_empty() {
            ui.add_space(20.0);
            empty_state(
                ui,
                Glyph::Person,
                crate::i18n::tr("Здесь появятся ваши одноклассники"),
                crate::i18n::tr("Нажмите «Новая запись», заполните карточку и сохраните её."),
            );
        }
        if let Some(action) = chosen {
            self.request(action);
        }
    }
    fn card_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(
                    RichText::new(if self.table_name() == "Одноклассники" {
                        crate::i18n::tr("Карточка ученика")
                    } else {
                        crate::i18n::tr("Карточка записи")
                    })
                    .color(INK)
                    .size(23.0),
                );
                ui.label(
                    RichText::new(if self.selected.id == 0 {
                        crate::i18n::tr("Новая запись").into()
                    } else {
                        match crate::i18n::current() {
                            crate::i18n::Language::Ru => {
                                format!("Запись {} из {}", self.current_position(), self.ids.len())
                            }
                            crate::i18n::Language::De => format!(
                                "Datensatz {} von {}",
                                self.current_position(),
                                self.ids.len()
                            ),
                            crate::i18n::Language::En => {
                                format!("Record {} of {}", self.current_position(), self.ids.len())
                            }
                        }
                    })
                    .color(MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(if self.dirty {
                        crate::i18n::tr("Изменения не сохранены")
                    } else {
                        crate::i18n::tr("Все изменения сохранены")
                    })
                    .size(14.0)
                    .color(MUTED),
                );
            });
        });
        ui.add_space(12.0);
        let controls = egui::TopBottomPanel::bottom("record-actions")
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(0, 10)),
            )
            .show_inside(ui, |ui| {
                card(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (label, action) in [
                            (crate::i18n::tr("В начало"), Action::First),
                            (crate::i18n::tr("Назад"), Action::Previous),
                            (crate::i18n::tr("Вперёд"), Action::Next),
                            (crate::i18n::tr("В конец"), Action::Last),
                        ] {
                            if ui.add(secondary_button(label)).clicked() {
                                self.request(action);
                            }
                        }
                        ui.separator();
                        if ui
                            .add(secondary_button(crate::i18n::tr("+ Новая")))
                            .clicked()
                        {
                            self.request(Action::NewRecord);
                        }
                        if ui
                            .add(primary_button(crate::i18n::tr("Сохранить")))
                            .clicked()
                        {
                            self.save_record();
                        }
                        if self.dirty
                            && ui
                                .add(secondary_button(crate::i18n::tr("Отменить правки")))
                                .clicked()
                        {
                            self.discard_changes();
                            self.notify(crate::i18n::tr("Несохранённые правки отменены"));
                        }
                        if self.selected.id != 0
                            && ui.add(danger_button(crate::i18n::tr("Удалить"))).clicked()
                        {
                            if self.dirty {
                                self.fail(
                                    crate::i18n::tr(
                                        "Перед удалением сохраните или отмените изменения карточки",
                                    )
                                    .into(),
                                );
                            } else {
                                self.delete_record_confirm = true;
                            }
                        }
                    });
                });
            });
        let fields = self.fields.clone();
        egui::ScrollArea::vertical().id_salt("record-form").auto_shrink([false,false]).show(ui,|ui|{
            let width=ui.available_width();
            ui.spacing_mut().item_spacing=egui::vec2(16.0,6.0);
            ui.spacing_mut().interact_size.y=28.0;
            let narrow=width<600.0;
            ui.with_layout(if narrow{egui::Layout::top_down(egui::Align::Min)}else{egui::Layout::left_to_right(egui::Align::Min)},|ui|{
                ui.allocate_ui_with_layout(egui::vec2(if narrow{width}else{(width-16.0)*0.65},0.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                    card(ui,|ui|{
                        section_title(ui,Glyph::Person,crate::i18n::tr("Основные данные"));
                        ui.add_space(4.0);
                        ui.horizontal(|ui|{
                            ui.spacing_mut().item_spacing.x=6.0;
                            ui.label(RichText::new("ID").color(MUTED));
                            ui.label(if self.selected.id==0 {crate::i18n::tr("Присвоится автоматически").into()}else{self.selected.id.to_string()});
                        });
                        ui.add_space(4.0);
                        for f in &fields {
                            if !f.visible || !matches!(f.name.as_str(),"Фамилия"|"Имя"|"Отчество"|"Дата_Рождения"){continue;}
                            ui.add_space(1.0);
                            let text=self.selected.values.entry(f.id).or_default();
                            if edit_field(ui,f,text){self.dirty=true;}
                        }
                    });
                    ui.add_space(12.0);
                    card(ui,|ui|{
                        section_title(ui,Glyph::Phone,crate::i18n::tr("Контакты"));ui.add_space(4.0);
                        let mut shown=false;
                        for f in &fields {
                            if f.visible && (f.name=="Телефон"||f.name=="Адрес_Проживания") {
                                shown=true;
                                let text=self.selected.values.entry(f.id).or_default();
                                if edit_field(ui,f,text){self.dirty=true;}
                                ui.add_space(4.0);
                            }
                        }
                        if !shown {ui.label(RichText::new(crate::i18n::tr("Поля контактов не добавлены")).color(MUTED));}
                    });
                    ui.add_space(12.0);
                    card(ui,|ui|{
                        section_title(ui,Glyph::Form,crate::i18n::tr("Дополнительная информация"));
                        ui.add_space(4.0);
                        for f in &fields {
                            if !f.visible || matches!(f.kind,FieldKind::Image|FieldKind::Url|FieldKind::File) {continue;}
                            if matches!(f.name.as_str(),"Фамилия"|"Имя"|"Отчество"|"Дата_Рождения"|"Телефон"|"Адрес_Проживания") {continue;}
                            let value=self.selected.values.entry(f.id).or_default();
                            if edit_field(ui,f,value){self.dirty=true;}
                            ui.add_space(5.0);
                        }
                        if ui.add(secondary_button(crate::i18n::tr("Добавить своё поле"))).clicked(){self.begin_new_field();}
                    });
                });
                ui.allocate_ui_with_layout(egui::vec2(if narrow{width}else{(width-16.0)*0.35},0.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                    let images=fields.iter().filter(|f|f.kind==FieldKind::Image&&f.visible).cloned().collect::<Vec<_>>();
                    for f in &images {
                        card(ui,|ui|{
                            let mut pick_photo=false;
                            let path=self.selected.value(f.id).to_owned();
                            let mut visible=false;
                            if !path.is_empty() {
                                let tex=if let Some(tex)=self.photo_cache.get(&path){Some(tex.clone())}
                                    else {
                                        match self.db().and_then(|db|assets::photo_texture(ctx,db,&path)) {
                                            Ok(tex)=>{self.photo_cache.insert(path.clone(),tex.clone());Some(tex)},
                                            Err(_)=>None,
                                        }
                                    };
                                if let Some(tex)=tex {
                                    let size=tex.size_vec2();
                                    let side=ui.available_width();
                                    let (rect,_)=ui.allocate_exact_size(egui::vec2(side,side),egui::Sense::hover());
                                    ui.painter().rect_filled(rect,14.0,egui::Color32::from_rgb(241,245,251));
                                    let scale=(side/size.x).min(side/size.y);
                                    let target=egui::Rect::from_center_size(rect.center(),size*scale);
                                    egui::Image::new((tex.id(),target.size())).corner_radius(12.0).paint_at(ui,target);visible=true;
                                }
                            }
                            if !visible {
                                let (rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),ui.available_width()),egui::Sense::click());
                                ui.painter().rect_filled(rect,14.0,if response.hovered(){SOFT_BLUE}else{egui::Color32::from_rgb(241,245,251)});
                                draw_glyph(ui.painter(),egui::Rect::from_center_size(rect.center()-egui::vec2(0.0,24.0),egui::vec2(32.0,32.0)),Glyph::Person,MUTED);
                                paint_centered_text(ui.painter(),rect.center()+egui::vec2(0.0,20.0),crate::i18n::tr("Выбрать фото"),egui::FontId::proportional(16.0),BLUE);
                                pick_photo=response.on_hover_text(crate::i18n::tr("Выбрать фотографию с компьютера")).clicked();
                            }
                            if visible || !path.is_empty() || pick_photo {
                            ui.add_space(14.0);
                            ui.horizontal_wrapped(|ui|{
                                ui.spacing_mut().item_spacing=egui::vec2(16.0,12.0);
                                if pick_photo || (visible && ui.add(secondary_button(crate::i18n::tr("Заменить фото"))).clicked()){
                                    if let Some(source)=rfd::FileDialog::new().add_filter(crate::i18n::tr("Изображения"),&["jpg","jpeg","png","bmp","webp"]).pick_file(){
                                        match self.db().and_then(|db|assets::import_file(db,&source,FieldKind::Image)){
                                            Ok(rel)=>{self.selected.values.insert(f.id,rel);self.dirty=true;self.notify(crate::i18n::tr("Фото добавлено. Нажмите «Сохранить»."));},
                                            Err(e)=>self.fail(e),
                                        }
                                    }
                                }
                                if !path.is_empty() && ui.add(secondary_button(crate::i18n::tr("Убрать"))).clicked(){self.selected.values.remove(&f.id);self.dirty=true;}
                            });
                            }
                        });ui.add_space(12.0);
                    }
                    for f in &fields {
                        if !f.visible {continue;}
                        match f.kind {
                            FieldKind::Url=>{
                                card(ui,|ui|{
                                    ui.label(RichText::new(Self::visible_name(&f.name)).strong().color(INK));
                                    let value=self.selected.values.entry(f.id).or_default();
                                    if ui.add_sized([ui.available_width(),44.0],egui::TextEdit::singleline(value).margin(egui::vec2(12.0,8.0)).vertical_align(egui::Align::Center).hint_text("https://example.com")).changed(){self.dirty=true;}
                                    ui.add_space(8.0);
                                    if ui.add(secondary_button(crate::i18n::tr("Перейти на сайт"))).clicked(){
                                        let url=self.selected.value(f.id).trim().to_owned();
                                        if FieldKind::Url.validate(&url).is_ok() && !url.is_empty() {
                                            if let Err(e)=open::that(url){self.fail(match crate::i18n::current(){crate::i18n::Language::Ru=>format!("Не удалось открыть сайт: {e}"),crate::i18n::Language::De=>format!("Website konnte nicht geöffnet werden: {e}"),crate::i18n::Language::En=>format!("Could not open website: {e}")});}
                                        }else{self.fail(crate::i18n::tr("Введите корректный адрес сайта").into());}
                                    }
                                });ui.add_space(12.0);
                            },
                            FieldKind::File=>{
                                card(ui,|ui|{
                                    ui.label(RichText::new(Self::visible_name(&f.name)).strong().color(INK));
                                    ui.add_space(8.0);
                                    let rel=self.selected.value(f.id).to_owned();
                                    ui.label(RichText::new(if rel.is_empty(){crate::i18n::tr("Файл не прикреплён").into()}else{rel.rsplit('/').next().unwrap_or("").to_owned()}).color(MUTED).size(14.0));
                                    ui.add_space(10.0);
                                    ui.horizontal_wrapped(|ui|{
                                        ui.spacing_mut().item_spacing=egui::vec2(16.0,12.0);
                                        if ui.add(secondary_button(crate::i18n::tr("+ Выбрать файл"))).clicked(){
                                            if let Some(source)=rfd::FileDialog::new().pick_file(){
                                                match self.db().and_then(|db|assets::import_file(db,&source,FieldKind::File)){
                                                    Ok(path)=>{self.selected.values.insert(f.id,path);self.dirty=true;self.notify(crate::i18n::tr("Документ добавлен. Сохраните запись."));},
                                                    Err(e)=>self.fail(e),
                                                }
                                            }
                                        }
                                        if !rel.is_empty() {
                                            if ui.add(secondary_button(crate::i18n::tr("Открыть файл"))).clicked(){
                                                match self.db().and_then(|db|assets::resolve_attachment(db,&rel)){
                                                    Ok(file)=>if let Err(e)=open::that(file){self.fail(e.to_string());},
                                                    Err(e)=>self.fail(e),
                                                }
                                            }
                                            if ui.add(secondary_button(crate::i18n::tr("Убрать"))).clicked(){self.selected.values.remove(&f.id);self.dirty=true;}
                                        }
                                    });
                                });ui.add_space(12.0);
                            },_=>{},
                        }
                    }
                });
            });
        });
        // Leave breathing room above the sticky toolbar rather than exposing
        // a clipped sliver of the next rounded card at the scroll boundary.
        let cut = controls.response.rect.top() + 10.0;
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(ui.max_rect().left(), cut - 20.0),
                egui::pos2(ui.max_rect().right(), cut),
            ),
            0.0,
            BG,
        );
    }
    fn export_csv(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(crate::i18n::tr("Таблица CSV"), &["csv"])
            .set_file_name(crate::i18n::tr("Картотека.csv"))
            .save_file()
        {
            match self
                .db()
                .and_then(|db| db.export_csv(self.active_table, &path))
            {
                Ok(count) => self.notify(match crate::i18n::current() {
                    crate::i18n::Language::Ru => format!(
                        "Экспортировано: {count} {}. {}",
                        record_word(count),
                        path.display()
                    ),
                    crate::i18n::Language::De => format!(
                        "Exportiert: {count} {}. {}",
                        record_word(count),
                        path.display()
                    ),
                    crate::i18n::Language::En => format!(
                        "Exported: {count} {}. {}",
                        record_word(count),
                        path.display()
                    ),
                }),
                Err(e) => self.fail(match crate::i18n::current() {
                    crate::i18n::Language::Ru => format!("Не удалось сохранить CSV: {e}"),
                    crate::i18n::Language::De => {
                        format!("CSV konnte nicht gespeichert werden: {e}")
                    }
                    crate::i18n::Language::En => format!("Could not save CSV: {e}"),
                }),
            }
        }
    }
    fn table_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Все записи"),
            &match crate::i18n::current() {
                crate::i18n::Language::Ru => format!(
                    "Таблица «{}» · {} {}",
                    self.table_title(),
                    self.ids.len(),
                    record_word(self.ids.len())
                ),
                crate::i18n::Language::De => format!(
                    "Tabelle „{}“ · {} {}",
                    self.table_title(),
                    self.ids.len(),
                    record_word(self.ids.len())
                ),
                crate::i18n::Language::En => format!(
                    "Table “{}” · {} {}",
                    self.table_title(),
                    self.ids.len(),
                    record_word(self.ids.len())
                ),
            },
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(primary_button(crate::i18n::tr("+ Новая запись")))
                .clicked()
            {
                self.request(Action::NewRecord);
            }
            if ui
                .add(secondary_button(crate::i18n::tr("Обновить")))
                .clicked()
            {
                self.request(Action::RefreshData);
            }
            if ui
                .add(secondary_button(crate::i18n::tr("Экспорт CSV")))
                .clicked()
            {
                self.export_csv();
            }
            if ui.add(secondary_button(crate::i18n::tr("Поиск"))).clicked() {
                self.screen = Screen::Search;
            }
            if ui
                .add_enabled(
                    self.selected_row.is_some(),
                    secondary_button(crate::i18n::tr("Открыть карточку")),
                )
                .clicked()
            {
                if let Some(id) = self.selected_row {
                    self.request(Action::OpenRecord(id));
                }
            }
            if ui
                .add(secondary_button(crate::i18n::tr("Поля таблицы")))
                .clicked()
            {
                self.screen = Screen::Designer;
            }
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.add_sized(
                [240.0, 35.0],
                egui::TextEdit::singleline(&mut self.table_filter)
                    .margin(egui::vec2(10.0, 8.0))
                    .hint_text(crate::i18n::tr("Поиск в таблице…")),
            );
            if !self.table_filter.is_empty()
                && ui
                    .add(secondary_button(crate::i18n::tr("Очистить поиск")))
                    .clicked()
            {
                self.table_filter.clear();
            }
            ui.separator();
            ui.label(RichText::new(crate::i18n::tr("Сортировка:")).color(MUTED));
            egui::ComboBox::from_id_salt("table-sort-field")
                .selected_text(
                    self.table_sort_field
                        .and_then(|id| self.fields.iter().find(|f| f.id == id))
                        .map(|f| Self::visible_name(&f.name))
                        .unwrap_or_else(|| "ID".into()),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.table_sort_field, None, "ID");
                    for f in &self.fields {
                        ui.selectable_value(
                            &mut self.table_sort_field,
                            Some(f.id),
                            Self::visible_name(&f.name),
                        );
                    }
                });
            quiet_checkbox(
                ui,
                &mut self.table_sort_desc,
                crate::i18n::tr("По убыванию"),
            );
        });
        ui.add_space(12.0);
        let fields = self
            .fields
            .iter()
            .filter(|f| f.visible)
            .cloned()
            .collect::<Vec<_>>();
        // Мгновенный фильтр показанных строк, без дополнительных запросов к диску.
        // Глобальный поиск во всей базе доступен в отдельном разделе «Поиск».
        let needle = self.table_filter.trim().to_lowercase();
        let mut records = self
            .table_records
            .iter()
            .filter(|record| {
                needle.is_empty()
                    || record.id.to_string().contains(&needle)
                    || record
                        .values
                        .values()
                        .any(|value| value.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect::<Vec<_>>();
        if let Some(fid) = self.table_sort_field {
            let kind = self.fields.iter().find(|f| f.id == fid).map(|f| f.kind);
            records.sort_by(|a, b| {
                let av = a.value(fid);
                let bv = b.value(fid);
                match kind {
                    Some(FieldKind::Integer | FieldKind::Decimal) => {
                        let parse = |s: &str| s.replace(',', ".").parse::<f64>().unwrap_or(0.0);
                        parse(av).total_cmp(&parse(bv))
                    }
                    _ => av.to_lowercase().cmp(&bv.to_lowercase()),
                }
            });
        }
        if self.table_sort_desc {
            records.reverse();
        }
        let mut open_record = None;
        let mut selected_row = self.selected_row;
        if records.is_empty() {
            empty_state(
                ui,
                Glyph::Table,
                crate::i18n::tr("Записей пока нет"),
                crate::i18n::tr("Создайте запись или измените фильтр таблицы."),
            );
            return;
        }
        ui.label(
            RichText::new(crate::i18n::tr(
                "Столбцы можно менять по ширине · остальные поля доступны прокруткой вправо",
            ))
            .size(14.0)
            .color(MUTED),
        );
        let widths_key = format!(
            "records-{}-{}",
            self.active_table,
            fields
                .iter()
                .map(|f| f.id.to_string())
                .collect::<Vec<_>>()
                .join("-")
        );
        let mut saved = self
            .prefs
            .column_widths
            .get(&widths_key)
            .cloned()
            .unwrap_or_default();
        let mins = (0..=fields.len())
            .map(|i| if i == 0 { 44.0 } else { 80.0 })
            .collect::<Vec<_>>();
        let reset_columns = batched_column_resize(ui.ctx(), &widths_key, &mut saved, &mins);
        if reset_columns {
            self.prefs
                .column_widths
                .insert(widths_key.clone(), saved.clone());
            self.save_preferences();
        }
        let mut header_rects = Vec::new();
        let mut table_widths = Vec::new();
        let table_height = (ui.available_height() - 70.0).max(100.0);
        egui::Frame::new()
            .fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0_f32, BORDER))
            .corner_radius(16)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::ScrollArea::horizontal()
                    .id_salt("records-x")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        let mut builder = TableBuilder::new(ui)
                            .id_salt((
                                "record-columns",
                                self.active_table,
                                fields.iter().map(|f| (f.id, f.width)).collect::<Vec<_>>(),
                            ))
                            .striped(true)
                            .resizable(true)
                            .sense(egui::Sense::click())
                            .max_scroll_height(table_height)
                            .auto_shrink([false, false])
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(
                                Column::initial(saved.first().copied().unwrap_or(52.0))
                                    .at_least(44.0)
                                    .clip(true),
                            );
                        for (index, f) in fields.iter().enumerate() {
                            let fallback = match f.name.as_str() {
                                "Дата_Рождения" => 160.0,
                                "Телефон" => 210.0,
                                _ => f.width as f32,
                            };
                            builder = builder.column(
                                Column::initial(saved.get(index + 1).copied().unwrap_or(fallback))
                                    .at_least(80.0)
                                    .clip(true),
                            );
                        }
                        if reset_columns {
                            builder.reset();
                        }
                        builder
                            .header(38.0, |mut header| {
                                header.col(|ui| {
                                    header_rects.push(ui.max_rect());
                                    table_heading(ui, "ID");
                                });
                                for field in &fields {
                                    header.col(|ui| {
                                        header_rects.push(ui.max_rect());
                                        table_heading(ui, &Self::visible_name(&field.name));
                                    });
                                }
                            })
                            .body(|body| {
                                table_widths = body.widths().to_vec();
                                body.rows(44.0, records.len(), |mut row| {
                                    let record = &records[row.index()];
                                    row.set_selected(selected_row == Some(record.id));
                                    row.col(|ui| {
                                        if cell_label(ui, &record.id.to_string(), false, false)
                                            .clicked()
                                        {
                                            open_record = Some(record.id);
                                        }
                                    });
                                    for f in &fields {
                                        row.col(|ui| {
                                            let val = record.value(f.id);
                                            if f.kind == FieldKind::Image {
                                                self.photo_thumbnail(ui, val, 32.0);
                                                return;
                                            }
                                            let text = match f.kind {
                                                FieldKind::Image => {
                                                    if val.is_empty() {
                                                        "—".into()
                                                    } else {
                                                        crate::i18n::tr("Фото").into()
                                                    }
                                                }
                                                FieldKind::File => {
                                                    if val.is_empty() {
                                                        "—".into()
                                                    } else {
                                                        crate::i18n::tr("Файл").into()
                                                    }
                                                }
                                                FieldKind::Boolean => {
                                                    if val == "1" {
                                                        crate::i18n::tr("Да").into()
                                                    } else {
                                                        crate::i18n::tr("Нет").into()
                                                    }
                                                }
                                                _ => val.replace('\n', " "),
                                            };
                                            let truncated = if text.chars().count() > 85 {
                                                format!(
                                                    "{}…",
                                                    text.chars().take(84).collect::<String>()
                                                )
                                            } else {
                                                text
                                            };
                                            let response = cell_label(ui, &truncated, false, false);
                                            if response.double_clicked() {
                                                open_record = Some(record.id);
                                            }
                                        });
                                    }
                                    let response = row.response();
                                    if response.clicked() {
                                        selected_row = Some(record.id);
                                    }
                                    if response.double_clicked() {
                                        open_record = Some(record.id);
                                    }
                                });
                            });
                    });
            });
        cache_column_headers(ui.ctx(), &widths_key, header_rects);
        self.remember_widths(widths_key, table_widths);
        self.selected_row = selected_row;
        if records.is_empty() {
            empty_state(
                ui,
                Glyph::Table,
                crate::i18n::tr("Записей пока нет"),
                crate::i18n::tr("Создайте запись или измените фильтр таблицы."),
            );
        }

        if let Some(id) = open_record {
            self.request(Action::OpenRecord(id));
        }
        if self.table_records.len() < self.ids.len() {
            ui.label(RichText::new(crate::i18n::tr("Показаны первые 2000 записей. Сортировка применяется только к показанным строкам; поиск работает по всей базе.")).color(MUTED));
        }
    }
    fn refresh_search(&mut self) {
        if self.search == self.search_last && self.search_table == self.active_table {
            return;
        }
        let result = self
            .db()
            .and_then(|db| db.search_records(self.active_table, &self.search, 500));
        match result {
            Ok(found) => {
                self.search_results = found;
                self.search_last = self.search.clone();
                self.search_table = self.active_table;
            }
            Err(e) => self.fail(e),
        }
    }
    fn search_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Поиск записей"),
            crate::i18n::tr("Введите фамилию, имя, телефон или любое слово"),
        );
        ui.add_space(20.0);
        card(ui, |ui| {
            let narrow = ui.available_width() < 620.0;
            ui.horizontal(|ui| {
                let reserve = if narrow { 0.0 } else { 300.0 };
                let input = search_input(
                    ui,
                    "page-search",
                    &mut self.search,
                    (ui.available_width() - reserve).max(100.0),
                    crate::i18n::tr("Например, Петрова"),
                );
                if self.focus_search {
                    input.request_focus();
                    self.focus_search = false;
                }
                if !narrow {
                    if action_button(ui, Glyph::Search, crate::i18n::tr("Найти"), true).clicked()
                    {
                        self.search_last = "\u{0}".into();
                    }
                    if ui
                        .add(secondary_button(crate::i18n::tr("Сбросить")))
                        .clicked()
                    {
                        self.search.clear();
                    }
                }
            });
            if narrow {
                ui.horizontal(|ui| {
                    if action_button(ui, Glyph::Search, crate::i18n::tr("Найти"), true).clicked()
                    {
                        self.search_last = "\u{0}".into();
                    }
                    if ui
                        .add(secondary_button(crate::i18n::tr("Сбросить")))
                        .clicked()
                    {
                        self.search.clear();
                    }
                });
            }
            ui.label(
                RichText::new(crate::i18n::tr("Поиск по всем полям текущей таблицы"))
                    .size(14.0)
                    .color(MUTED),
            );
        });
        ui.add_space(18.0);
        self.refresh_search();
        let results = self.search_results.clone();
        ui.label(
            RichText::new(match crate::i18n::current() {
                crate::i18n::Language::Ru => format!("Результаты поиска ({})", results.len()),
                crate::i18n::Language::De => format!("Suchergebnisse ({})", results.len()),
                crate::i18n::Language::En => format!("Search results ({})", results.len()),
            })
            .size(16.0)
            .strong()
            .color(INK),
        );
        ui.add_space(12.0);
        let mut selected = None;
        egui::ScrollArea::vertical()
            .id_salt("search-results")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for r in &results {
                    card(ui, |ui| {
                        ui.horizontal(|ui| {
                            self.record_avatar(ui, r, 84.0);
                            let info_width = (ui.available_width() - 120.0).max(50.0);
                            let name = self.name_of_record(r);
                            let details = self
                                .fields
                                .iter()
                                .filter(|f| {
                                    matches!(
                                        f.name.as_str(),
                                        "Дата_Рождения" | "Телефон" | "Адрес_Проживания"
                                    )
                                })
                                .map(|f| r.value(f.id))
                                .filter(|v| !v.is_empty())
                                .collect::<Vec<_>>()
                                .join("   ·   ");
                            result_details(
                                ui,
                                &name,
                                &if details.is_empty() {
                                    match crate::i18n::current() {
                                        crate::i18n::Language::Ru => format!("Запись № {}", r.id),
                                        crate::i18n::Language::De => {
                                            format!("Datensatz Nr. {}", r.id)
                                        }
                                        crate::i18n::Language::En => format!("Record no. {}", r.id),
                                    }
                                } else {
                                    details
                                },
                                info_width,
                                84.0,
                            );
                            if ui
                                .add(secondary_button(crate::i18n::tr("Открыть")))
                                .clicked()
                            {
                                selected = Some(r.id);
                            }
                        });
                    });
                    ui.add_space(10.0);
                }
                if results.is_empty() {
                    empty_state(
                        ui,
                        Glyph::Search,
                        crate::i18n::tr("Ничего не найдено"),
                        crate::i18n::tr("Измените запрос или добавьте первую запись."),
                    );
                }
                if results.len() == 500 {
                    ui.label(
                        RichText::new(crate::i18n::tr(
                            "Первые 500 результатов. Уточните запрос, чтобы сузить поиск.",
                        ))
                        .color(MUTED),
                    );
                }
            });
        if let Some(id) = selected {
            self.request(Action::OpenRecord(id));
        }
    }
    fn record_avatar(&mut self, ui: &mut egui::Ui, record: &Record, size: f32) {
        let path = self
            .fields
            .iter()
            .find(|f| f.kind == FieldKind::Image && f.visible)
            .map(|f| record.value(f.id))
            .unwrap_or("")
            .to_owned();
        self.photo_thumbnail(ui, &path, size);
    }
    fn photo_thumbnail(&mut self, ui: &mut egui::Ui, path: &str, size: f32) {
        if !path.is_empty() {
            let texture = self.photo_cache.get(path).cloned().or_else(|| {
                let tex = self
                    .db()
                    .and_then(|db| assets::photo_texture(ui.ctx(), db, &path))
                    .ok()?;
                self.photo_cache.insert(path.to_owned(), tex.clone());
                Some(tex)
            });
            if let Some(tex) = texture {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
                let dims = tex.size_vec2();
                let scale = (size / dims.x).min(size / dims.y);
                let target = egui::Rect::from_center_size(rect.center(), dims * scale);
                ui.painter().rect_filled(rect, 10.0, SOFT_BLUE);
                egui::Image::new((tex.id(), target.size()))
                    .corner_radius(12.0)
                    .paint_at(ui, target);
                return;
            }
        }
        icon_badge(ui, Glyph::Person, size, SOFT_BLUE, BLUE);
    }
    fn begin_new_field(&mut self) {
        if self.dirty {
            self.fail(
                crate::i18n::tr("Сначала сохраните изменения в карточке, затем добавьте поле")
                    .into(),
            );
            return;
        }
        self.field_editor = FieldDef::blank(self.active_table);
        self.field_editor.order = self.fields.len() as i64;
        self.add_field_dialog = true;
    }
    fn edit_field(&mut self, id: i64) {
        if let Some(field) = self.fields.iter().find(|f| f.id == id) {
            self.field_editor = field.clone();
            self.add_field_dialog = true;
        }
    }
    fn save_field(&mut self) {
        self.prefs
            .column_widths
            .retain(|key, _| !key.starts_with(&format!("records-{}-", self.active_table)));
        // При изменении типа существующего поля копируем базу заранее.
        let should_backup = self.field_editor.id > 0
            && self
                .fields
                .iter()
                .any(|f| f.id == self.field_editor.id && f.kind != self.field_editor.kind);
        if should_backup {
            if let Err(e) = self.db().and_then(assets::create_backup) {
                self.fail(match crate::i18n::current() {
                    crate::i18n::Language::Ru => {
                        format!("Не удалось сохранить защитную копию: {e}")
                    }
                    crate::i18n::Language::De => {
                        format!("Schutzkopie konnte nicht gespeichert werden: {e}")
                    }
                    crate::i18n::Language::En => format!("Could not save safety copy: {e}"),
                });
                return;
            }
        }
        let result = self.db().and_then(|db| db.update_field(&self.field_editor));
        match result {
            Ok(id) => {
                self.add_field_dialog = false;
                self.refresh_data();
                self.selected_field = Some(id);
                self.notify(crate::i18n::tr("Структура таблицы сохранена"));
            }
            Err(e) => self.fail(e),
        }
    }
    fn move_field(&mut self, id: i64, direction: i32) {
        if self.dirty {
            self.fail(
                crate::i18n::tr("Сначала сохраните карточку, затем меняйте порядок полей").into(),
            );
            return;
        }
        match self
            .db()
            .and_then(|db| db.move_field(self.active_table, id, direction))
        {
            Ok(true) => {
                self.refresh_data();
                self.selected_field = Some(id);
                self.notify(crate::i18n::tr("Порядок полей изменён"));
            }
            Ok(false) => self.notify(crate::i18n::tr("Поле уже находится на краю списка")),
            Err(e) => self.fail(e),
        }
    }
    fn designer_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Конструктор таблиц"),
            crate::i18n::tr("Настройте структуру картотеки и добавьте собственные поля"),
        );
        ui.add_space(20.0);
        let mut chosen = None;
        let mut edit = None;
        let mut movement = None;
        let mut new_field = false;
        let width = ui.available_width();
        let stacked = width < 1040.0;
        ui.with_layout(
            if stacked {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Min)
            },
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(if stacked { width } else { 260.0 }, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            section_title(ui, Glyph::Database, crate::i18n::tr("Таблицы"));
                            ui.add_space(8.0);
                            for table in &self.tables {
                                if nav_item(
                                    ui,
                                    Glyph::Table,
                                    if table.id == 1 {
                                        crate::i18n::tr(&table.name)
                                    } else {
                                        &table.name
                                    },
                                    table.id == self.active_table,
                                    false,
                                )
                                .clicked()
                                {
                                    chosen = Some(table.id);
                                }
                            }
                            ui.add_space(12.0);
                            if ui
                                .add(secondary_button(crate::i18n::tr("Создать таблицу")))
                                .clicked()
                            {
                                self.create_table_dialog = true;
                            }
                        });
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(
                        if stacked {
                            width
                        } else {
                            (width - 276.0).max(200.0)
                        },
                        0.0,
                    ),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(match crate::i18n::current() {
                                        crate::i18n::Language::Ru => {
                                            format!("Структура таблицы «{}»", self.table_title())
                                        }
                                        crate::i18n::Language::De => {
                                            format!("Struktur der Tabelle „{}“", self.table_title())
                                        }
                                        crate::i18n::Language::En => {
                                            format!("Structure of table “{}”", self.table_title())
                                        }
                                    })
                                    .size(18.0)
                                    .strong()
                                    .color(INK),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if action_button(
                                            ui,
                                            Glyph::Plus,
                                            crate::i18n::tr("Добавить поле"),
                                            true,
                                        )
                                        .clicked()
                                        {
                                            new_field = true;
                                        }
                                    },
                                );
                            });
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(crate::i18n::tr(
                                    "Потяните границы заголовков, чтобы изменить ширину колонок",
                                ))
                                .size(14.0)
                                .color(MUTED),
                            );
                            let key = format!("designer-{}", self.active_table);
                            let mut saved = self
                                .prefs
                                .column_widths
                                .get(&key)
                                .cloned()
                                .unwrap_or_default();
                            let reset_columns = batched_column_resize(
                                ui.ctx(),
                                &key,
                                &mut saved,
                                &[160.0, 140.0, 130.0, 220.0],
                            );
                            if reset_columns {
                                self.prefs.column_widths.insert(key.clone(), saved.clone());
                                self.save_preferences();
                            }
                            let mut header_rects = Vec::new();
                            let initial = |index: usize, fallback: f32| {
                                saved.get(index).copied().unwrap_or(fallback)
                            };
                            let mut widths = Vec::new();
                            let name_width = (ui.available_width() - 580.0).max(180.0);
                            let body_height =
                                (ui.ctx().screen_rect().height() * 0.42).clamp(220.0, 420.0);
                            egui::ScrollArea::horizontal()
                                .id_salt("designer-x")
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                                .show(ui, |ui| {
                                    ui.set_min_width(728.0);
                                    let builder = TableBuilder::new(ui)
                                        .id_salt(("designer-columns", self.active_table))
                                        .striped(true)
                                        .resizable(true)
                                        .vscroll(true)
                                        .min_scrolled_height(body_height)
                                        .max_scroll_height(body_height)
                                        .cell_layout(egui::Layout::left_to_right(
                                            egui::Align::Center,
                                        ))
                                        .column(
                                            Column::initial(initial(0, name_width))
                                                .at_least(160.0)
                                                .clip(true),
                                        )
                                        .column(
                                            Column::initial(initial(1, 156.0))
                                                .at_least(140.0)
                                                .clip(true),
                                        )
                                        .column(
                                            Column::initial(initial(2, 136.0))
                                                .at_least(130.0)
                                                .clip(true),
                                        )
                                        .column(
                                            Column::initial(initial(3, 220.0))
                                                .at_least(220.0)
                                                .clip(true),
                                        );
                                    if reset_columns {
                                        builder.reset();
                                    }
                                    builder
                                        .header(44.0, |mut row| {
                                            for text in [
                                                crate::i18n::tr("Имя поля"),
                                                crate::i18n::tr("Тип данных"),
                                                crate::i18n::tr("Обязательное"),
                                                crate::i18n::tr("Действия"),
                                            ] {
                                                row.col(|ui| {
                                                    header_rects.push(ui.max_rect());
                                                    table_heading(ui, text);
                                                });
                                            }
                                        })
                                        .body(|body| {
                                            widths = body.widths().to_vec();
                                            body.rows(44.0, self.fields.len() + 1, |mut row| {
                                                let index = row.index();
                                                if index == 0 {
                                                    for (column, text) in [
                                                        crate::i18n::tr("ID ученика"),
                                                        crate::i18n::tr("Счётчик"),
                                                        crate::i18n::tr("Да"),
                                                        crate::i18n::tr("Автоматически"),
                                                    ]
                                                    .into_iter()
                                                    .enumerate()
                                                    {
                                                        row.col(|ui| {
                                                            cell_label(
                                                                ui,
                                                                text,
                                                                column == 2,
                                                                false,
                                                            );
                                                        });
                                                    }
                                                } else {
                                                    let f = &self.fields[index - 1];
                                                    row.set_selected(
                                                        self.selected_field == Some(f.id),
                                                    );
                                                    row.col(|ui| {
                                                        if cell_label(
                                                            ui,
                                                            &Self::visible_name(&f.name),
                                                            false,
                                                            false,
                                                        )
                                                        .clicked()
                                                        {
                                                            self.selected_field = Some(f.id);
                                                        }
                                                    });
                                                    row.col(|ui| {
                                                        cell_label(
                                                            ui,
                                                            f.kind.label(),
                                                            false,
                                                            false,
                                                        );
                                                    });
                                                    row.col(|ui| {
                                                        cell_label(
                                                            ui,
                                                            if f.required {
                                                                crate::i18n::tr("Да")
                                                            } else {
                                                                "—"
                                                            },
                                                            true,
                                                            false,
                                                        );
                                                    });
                                                    row.col(|ui| {
                                                        ui.horizontal_centered(|ui| {
                                                            ui.add_space(12.0);
                                                            if ui
                                                                .add(secondary_button(
                                                                    crate::i18n::tr("Настроить"),
                                                                ))
                                                                .clicked()
                                                            {
                                                                edit = Some(f.id);
                                                            }
                                                            if navigation_icon_button(
                                                                ui,
                                                                Glyph::Up,
                                                                crate::i18n::tr("Поднять поле"),
                                                            )
                                                            .clicked()
                                                            {
                                                                movement = Some((f.id, -1));
                                                            }
                                                            if navigation_icon_button(
                                                                ui,
                                                                Glyph::Down,
                                                                crate::i18n::tr("Опустить поле"),
                                                            )
                                                            .clicked()
                                                            {
                                                                movement = Some((f.id, 1));
                                                            }
                                                        });
                                                    });
                                                }
                                            });
                                        });
                                });
                            cache_column_headers(ui.ctx(), &key, header_rects);
                            self.remember_widths(key, widths);
                            ui.add_space(12.0);
                        });
                        ui.add_space(16.0);
                        card(ui, |ui| {
                            section_title(ui, Glyph::Form, crate::i18n::tr("Свойства поля"));
                            if let Some(f) = self
                                .selected_field
                                .and_then(|id| self.fields.iter().find(|f| f.id == id))
                            {
                                ui.label(
                                    RichText::new(Self::visible_name(&f.name))
                                        .strong()
                                        .color(INK),
                                );
                                ui.label(
                                    RichText::new(match crate::i18n::current() {
                                        crate::i18n::Language::Ru => format!(
                                            "{} · ширина {} px · {}",
                                            f.kind.label(),
                                            f.width,
                                            if f.visible {
                                                crate::i18n::tr("видно в форме")
                                            } else {
                                                crate::i18n::tr("скрыто в форме")
                                            }
                                        ),
                                        crate::i18n::Language::De => format!(
                                            "{} · Breite {} px · {}",
                                            f.kind.label(),
                                            f.width,
                                            if f.visible {
                                                crate::i18n::tr("видно в форме")
                                            } else {
                                                crate::i18n::tr("скрыто в форме")
                                            }
                                        ),
                                        crate::i18n::Language::En => format!(
                                            "{} · width {} px · {}",
                                            f.kind.label(),
                                            f.width,
                                            if f.visible {
                                                crate::i18n::tr("видно в форме")
                                            } else {
                                                crate::i18n::tr("скрыто в форме")
                                            }
                                        ),
                                    })
                                    .color(MUTED),
                                );
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Изменить свойства")))
                                    .clicked()
                                {
                                    edit = Some(f.id);
                                }
                            } else {
                                ui.label(
                                    RichText::new(crate::i18n::tr(
                                        "Выберите поле в таблице, чтобы посмотреть его свойства.",
                                    ))
                                    .color(MUTED),
                                );
                            }
                        });
                    },
                );
            },
        );
        if let Some(id) = chosen {
            self.request(Action::SelectTable(id));
        }
        if let Some(id) = edit {
            self.edit_field(id);
        }
        if let Some((id, direction)) = movement {
            self.move_field(id, direction);
        }
        if new_field {
            self.begin_new_field();
        }
    }
    fn forms_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Конструктор форм"),
            crate::i18n::tr("Выберите поля для карточки и настройте их свойства"),
        );
        ui.add_space(20.0);
        let mut edit = None;
        let mut toggle = None;
        let mut new_field = false;
        let width = ui.available_width();
        let fields = self.fields.clone();
        let wide = width >= 960.0;
        if self.selected_field.is_none() {
            self.selected_field = fields.first().map(|f| f.id);
        }
        let stacked = width < 720.0;
        ui.with_layout(
            if stacked {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Min)
            },
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(if stacked { width } else { 240.0 }, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            section_title(ui, Glyph::Form, crate::i18n::tr("Доступные поля"));
                            ui.label(
                                RichText::new(crate::i18n::tr("Галочка — показывать поле"))
                                    .size(14.0)
                                    .color(MUTED),
                            );
                            for f in &fields {
                                ui.horizontal(|ui| {
                                    let mut visible = f.visible;
                                    if quiet_checkbox(ui, &mut visible, "")
                                        .on_hover_text(match crate::i18n::current() {
                                            crate::i18n::Language::Ru => format!(
                                                "Показывать поле «{}» в карточке и таблице",
                                                Self::visible_name(&f.name)
                                            ),
                                            crate::i18n::Language::De => format!(
                                                "Feld „{}“ in Karte und Tabelle anzeigen",
                                                Self::visible_name(&f.name)
                                            ),
                                            crate::i18n::Language::En => format!(
                                                "Show field “{}” in the card and table",
                                                Self::visible_name(&f.name)
                                            ),
                                        })
                                        .changed()
                                    {
                                        toggle = Some((f.id, visible));
                                    }
                                    if ui
                                        .selectable_label(
                                            self.selected_field == Some(f.id),
                                            Self::visible_name(&f.name),
                                        )
                                        .clicked()
                                    {
                                        self.selected_field = Some(f.id);
                                    }
                                });
                            }
                            ui.add_space(10.0);
                            if ui
                                .add(secondary_button(crate::i18n::tr("Добавить поле")))
                                .clicked()
                            {
                                new_field = true;
                            }
                        });
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(
                        if stacked {
                            width
                        } else {
                            (width - if wide { 524.0 } else { 252.0 }).max(200.0)
                        },
                        0.0,
                    ),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            section_title(ui, Glyph::Person, crate::i18n::tr("Предпросмотр формы"));
                            ui.label(
                                RichText::new(crate::i18n::tr("Карточка ученика"))
                                    .size(18.0)
                                    .strong()
                                    .color(INK),
                            );
                            ui.label(
                                RichText::new(crate::i18n::tr(
                                    "Поля показываются в сохранённом порядке",
                                ))
                                .size(14.0)
                                .color(MUTED),
                            );
                            ui.add_space(12.0);
                            for f in fields.iter().filter(|f| f.visible) {
                                ui.push_id(("preview", f.id), |ui| {
                                    if f.kind == FieldKind::Image {
                                        icon_badge(ui, Glyph::Person, 56.0, SOFT_BLUE, MUTED);
                                    } else {
                                        preview_field(ui, f);
                                    }
                                    ui.add_space(4.0);
                                });
                            }
                        });
                        if !wide {
                            ui.add_space(16.0);
                            if self.form_properties(ui) {
                                edit = self.selected_field;
                            }
                        }
                    },
                );
                if wide {
                    ui.allocate_ui_with_layout(
                        egui::vec2(260.0, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if self.form_properties(ui) {
                                edit = self.selected_field;
                            }
                        },
                    );
                }
            },
        );
        if let Some((id, visible)) = toggle {
            if let Some(mut f) = fields.iter().find(|f| f.id == id).cloned() {
                f.visible = visible;
                match self.db().and_then(|db| db.update_field(&f)) {
                    Ok(_) => {
                        self.fields
                            .iter_mut()
                            .filter(|f| f.id == id)
                            .for_each(|f| f.visible = visible);
                        self.notify(crate::i18n::tr("Видимость поля обновлена"));
                    }
                    Err(e) => self.fail(e),
                }
            }
        }
        if let Some(id) = edit {
            self.edit_field(id);
        }
        if new_field {
            self.begin_new_field();
        }
    }
    fn form_properties(&self, ui: &mut egui::Ui) -> bool {
        card(ui, |ui| {
            section_title(ui, Glyph::Settings, crate::i18n::tr("Настройки поля"));
            if let Some(f) = self
                .selected_field
                .and_then(|id| self.fields.iter().find(|f| f.id == id))
            {
                ui.label(
                    RichText::new(Self::visible_name(&f.name))
                        .size(16.0)
                        .strong()
                        .color(INK),
                );
                ui.label(
                    RichText::new(crate::i18n::tr("Тип отображения"))
                        .color(MUTED)
                        .size(14.0),
                );
                ui.label(f.kind.label());
                ui.label(match crate::i18n::current() {
                    crate::i18n::Language::Ru => format!("Ширина: {} px", f.width),
                    crate::i18n::Language::De => format!("Breite: {} px", f.width),
                    crate::i18n::Language::En => format!("Width: {} px", f.width),
                });
                ui.label(if f.visible {
                    crate::i18n::tr("Показывать в карточке")
                } else {
                    crate::i18n::tr("Скрыто в карточке")
                });
                ui.label(if f.required {
                    crate::i18n::tr("Обязательное поле")
                } else {
                    crate::i18n::tr("Необязательное поле")
                });
                ui.add_space(8.0);
                ui.add(secondary_button(crate::i18n::tr("Изменить свойства")))
                    .clicked()
            } else {
                ui.label(RichText::new(crate::i18n::tr("Выберите поле слева")).color(MUTED));
                false
            }
        })
    }
    fn backups_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Резервные копии"),
            crate::i18n::tr("База, фотографии и документы — в одной копии"),
        );
        ui.add_space(20.0);
        let mut create = false;
        ui.columns(2, |columns| {
            card(&mut columns[0], |ui| {
                ui.set_min_height(150.0);
                section_title(ui, Glyph::Database, crate::i18n::tr("Текущая база данных"));
                ui.label(RichText::new(self.table_title()).strong().color(INK));
                ui.label(format!(
                    "{} {}",
                    self.ids.len(),
                    record_word(self.ids.len())
                ));
                if let Some(db) = &self.db {
                    if let Ok(meta) = std::fs::metadata(&db.path) {
                        ui.label(
                            RichText::new(match crate::i18n::current() {
                                crate::i18n::Language::Ru => {
                                    format!("Размер базы: {:.2} МБ", meta.len() as f64 / 1048576.0)
                                }
                                crate::i18n::Language::De => format!(
                                    "Datenbankgröße: {:.2} MB",
                                    meta.len() as f64 / 1048576.0
                                ),
                                crate::i18n::Language::En => format!(
                                    "Database size: {:.2} MB",
                                    meta.len() as f64 / 1048576.0
                                ),
                            })
                            .size(14.0)
                            .color(MUTED),
                        );
                    }
                    ui.label(
                        RichText::new(display_data_path(&db.path))
                            .size(14.0)
                            .color(MUTED),
                    );
                }
            });
            card(&mut columns[1], |ui| {
                ui.set_min_height(150.0);
                section_title(
                    ui,
                    Glyph::Shield,
                    crate::i18n::tr("Создать резервную копию"),
                );
                ui.label(
                    RichText::new(crate::i18n::tr("Будут сохранены все таблицы и вложения."))
                        .size(14.0)
                        .color(MUTED),
                );
                ui.label(
                    RichText::new(crate::i18n::tr("Папка: backups рядом с базой"))
                        .size(14.0)
                        .color(MUTED),
                );
                ui.add_space(8.0);
                if action_button(ui, Glyph::Backup, crate::i18n::tr("Создать копию"), true)
                    .clicked()
                {
                    create = true;
                }
            });
        });
        if create {
            self.request(Action::Backup);
        }
        ui.add_space(22.0);
        ui.label(
            RichText::new(match crate::i18n::current() {
                crate::i18n::Language::Ru => {
                    format!("История резервных копий ({})", self.backups.len())
                }
                crate::i18n::Language::De => format!("Sicherungsverlauf ({})", self.backups.len()),
                crate::i18n::Language::En => format!("Backup history ({})", self.backups.len()),
            })
            .size(18.0)
            .strong()
            .color(INK),
        );
        ui.add_space(12.0);
        if self.backups.is_empty() {
            empty_state(
                ui,
                Glyph::Backup,
                crate::i18n::tr("Копий пока нет"),
                crate::i18n::tr("Создайте первую копию, чтобы сохранить вашу картотеку."),
            );
        }
        let mut restore = None;
        for folder in &self.backups {
            let incomplete = folder.join("WARNING.txt").is_file();
            card(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    icon_badge(
                        ui,
                        Glyph::Backup,
                        40.0,
                        if incomplete { RED_SOFT } else { SOFT_BLUE },
                        if incomplete { DANGER } else { BLUE },
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(folder.file_name().unwrap_or_default().to_string_lossy())
                                .strong()
                                .color(INK),
                        );
                        ui.label(
                            RichText::new(if incomplete {
                                crate::i18n::tr(
                                    "Неполная защитная копия · восстановление недоступно",
                                )
                            } else {
                                crate::i18n::tr("База и прикреплённые файлы")
                            })
                            .size(14.0)
                            .color(if incomplete {
                                DANGER
                            } else {
                                MUTED
                            }),
                        );
                    });
                    ui.add_space(20.0);
                    if ui
                        .add_enabled(
                            !incomplete,
                            secondary_button(crate::i18n::tr("Восстановить")),
                        )
                        .clicked()
                    {
                        restore = Some(folder.clone());
                    }
                    if ui
                        .add(secondary_button(crate::i18n::tr("Открыть папку")))
                        .clicked()
                    {
                        if let Err(e) = open::that(folder) {
                            self.error = e.to_string();
                        }
                    }
                });
            });
            ui.add_space(10.0);
        }
        if let Some(path) = restore {
            self.restore_confirm = Some(path);
        }
    }
    fn settings_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        title(
            ui,
            crate::i18n::tr("Настройки"),
            crate::i18n::tr("Внешний вид и хранение вашей картотеки"),
        );
        ui.add_space(20.0);
        let width = ui.available_width();
        let stacked = width < 780.0;
        ui.with_layout(if stacked{egui::Layout::top_down(egui::Align::Min)}else{egui::Layout::left_to_right(egui::Align::Min)},|ui|{
            ui.allocate_ui_with_layout(egui::vec2(if stacked{width}else{270.0},0.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                card(ui,|ui|{for (i,label) in [crate::i18n::tr("Общие"),crate::i18n::tr("Внешний вид"),crate::i18n::tr("Папка хранения"),crate::i18n::tr("О программе")].iter().enumerate(){
                    if nav_item(ui,[Glyph::Settings,Glyph::Appearance,Glyph::Folder,Glyph::Info][i],label,self.settings_topic==i,false).clicked(){self.settings_topic=i;}
                }});
            });
            ui.allocate_ui_with_layout(egui::vec2(if stacked{width}else{(width-286.0).max(210.0)},0.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                card(ui,|ui|{match self.settings_topic{
                    0|1=>{
                        section_title(ui,Glyph::Settings,crate::i18n::tr("Внешний вид"));
 ui.label(crate::i18n::tr("Язык интерфейса"));
 ui.horizontal_wrapped(|ui|{for language in crate::i18n::Language::ALL {if ui.selectable_label(self.prefs.language==language,language.name()).clicked(){self.prefs.language=language;crate::i18n::set(language);self.status=crate::i18n::tr("Готово").to_owned();self.error.clear();self.save_preferences();ctx.request_repaint();}}});

                        ui.add_space(12.0);ui.label(crate::i18n::tr("Размер интерфейса"));
                        ui.horizontal(|ui| {
                            let changed=ui.scope(|ui| {
                                ui.spacing_mut().interact_size.y=22.0;
                                ui.spacing_mut().slider_width=180.0;
                                ui.visuals_mut().widgets.inactive.bg_fill=BLUE;
                                ui.visuals_mut().widgets.hovered.bg_fill=BLUE;
                                ui.visuals_mut().widgets.active.bg_fill=BLUE;
                                ui.visuals_mut().widgets.inactive.bg_stroke=egui::Stroke::NONE;
                                ui.visuals_mut().widgets.inactive.fg_stroke=egui::Stroke::NONE;
                                ui.visuals_mut().widgets.hovered.fg_stroke=egui::Stroke::NONE;
                                ui.visuals_mut().widgets.active.fg_stroke=egui::Stroke::NONE;
                                ui.visuals_mut().slider_trailing_fill=false;
                                ui.add(egui::Slider::new(&mut self.prefs.zoom,0.8..=1.5).step_by(0.05).show_value(false)).changed()
                            }).inner;
                            ui.label(format!("{} %",(self.prefs.zoom*100.0).round()));
                            if changed{ctx.set_zoom_factor(self.prefs.zoom);self.save_preferences();}
                        });
                        ui.label(RichText::new("80 % — 150 %").size(14.0).color(MUTED));
                        ui.add_space(20.0);ui.label(crate::i18n::tr("Левое меню"));ui.horizontal_wrapped(|ui|{
                            for (label,mode) in [(crate::i18n::tr("Полное"),Sidebar::Full),(crate::i18n::tr("Иконки"),Sidebar::Icons),(crate::i18n::tr("Скрыто"),Sidebar::Hidden)]{
                                if ui.selectable_label(self.prefs.sidebar==mode,label).clicked(){self.prefs.sidebar=mode;self.save_preferences();}
                            }
                        });ui.add_space(12.0);ui.label(RichText::new(crate::i18n::tr("Шрифт: Segoe UI · синий акцент #2E6BFF")).size(14.0).color(MUTED));
                        ui.add_space(12.0);if ui.add(secondary_button(crate::i18n::tr("Восстановить оформление по умолчанию"))).clicked(){let language=self.prefs.language;self.prefs=Preferences::default();self.prefs.language=language;ctx.set_zoom_factor(1.0);self.save_preferences();}
                    },
                    2=>{
                        section_title(ui,Glyph::Folder,crate::i18n::tr("Место хранения"));
                        if let Some(db)=&self.db{ui.label(RichText::new(db.root.display().to_string()).color(MUTED));
                            if ui.add(secondary_button(crate::i18n::tr("Открыть папку данных"))).clicked(){if let Err(e)=open::that(&db.root){self.error=e.to_string();}}
                        }
                        ui.add_space(12.0);ui.label(crate::i18n::tr("База, фотографии и документы хранятся локально. Для переноса используйте резервную копию."));
                    },
                    _=>{
                        section_title(ui,Glyph::Person,crate::i18n::tr("Одноклассники — Картотека"));ui.label(crate::i18n::tr("Версия 0.5.0 · Quiet Premium Workspace"));
                        ui.label(crate::i18n::tr("Программа работает без интернета. Данные сохраняются на компьютере без шифрования."));
                        ui.add_space(12.0);ui.label(RichText::new(crate::i18n::tr("Ctrl+S — сохранить · Ctrl+N — новая запись · Ctrl+F — поиск")).color(MUTED));
                    }
                }});
            });
        });
    }
    fn help_screen(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            crate::i18n::tr("Справка"),
            crate::i18n::tr("Всё для уверенной работы с картотекой"),
        );
        ui.add_space(20.0);
        let topics=[
            (crate::i18n::tr("Начало работы"),crate::i18n::tr("Создайте первую карточку, заполните данные и сохраните запись. Переключайтесь между карточками и общей таблицей через левое меню.")),
            (crate::i18n::tr("Добавление записей"),crate::i18n::tr("Нажмите «Новая запись», заполните фамилию и имя, затем нажмите «Сохранить» в нижней панели. Обязательные поля отмечены звёздочкой.")),
            (crate::i18n::tr("Работа с таблицей"),crate::i18n::tr("Нажмите на строку для выделения, затем «Открыть карточку». Двойной щелчок также открывает запись. Ширину столбцов можно менять мышью. CSV доступен через кнопку экспорта.")),
            (crate::i18n::tr("Поиск записей"),crate::i18n::tr("Введите часть имени, телефона или адреса. Поиск просматривает все поля выбранной таблицы. Фильтр над таблицей работает по загруженным строкам.")),
            (crate::i18n::tr("Фото и документы"),crate::i18n::tr("В карточке нажмите «Выбрать фото» или «Выбрать файл». Файл копируется в папку картотеки. Нажмите «Сохранить», чтобы закрепить вложение за записью.")),
            (crate::i18n::tr("Собственные поля"),crate::i18n::tr("Откройте «Конструктор», выберите таблицу и нажмите «Добавить поле». Задайте название и тип. Видимость настраивается в разделе «Формы», порядок — стрелками в конструкторе.")),
            (crate::i18n::tr("Резервные копии"),crate::i18n::tr("Нажмите «Создать копию». В копию войдут база и вложения. Восстановление заменяет текущие данные; программа сначала создаёт защитный снимок.")),
            (crate::i18n::tr("Настройки и клавиши"),crate::i18n::tr("Знак приложения или разделитель меню переключает полное меню, иконки и скрытый режим. Масштаб 80–150 % находится в настройках. Ctrl+S — сохранить, Ctrl+N — новая запись, Ctrl+F — поиск.")),
        ];
        let width = ui.available_width();
        let stacked = width < 820.0;
        ui.with_layout(
            if stacked {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Min)
            },
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(if stacked { width } else { 304.0 }, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            for (i, (label, _)) in topics.iter().enumerate() {
                                if nav_item(
                                    ui,
                                    [
                                        Glyph::Home,
                                        Glyph::Plus,
                                        Glyph::Table,
                                        Glyph::Search,
                                        Glyph::Folder,
                                        Glyph::Tools,
                                        Glyph::Backup,
                                        Glyph::Settings,
                                    ][i],
                                    label,
                                    self.help_topic == i,
                                    false,
                                )
                                .clicked()
                                {
                                    self.help_topic = i;
                                }
                            }
                        });
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(
                        if stacked {
                            width
                        } else {
                            (width - 320.0).max(200.0)
                        },
                        0.0,
                    ),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        card(ui, |ui| {
                            ui.label(
                                RichText::new(topics[self.help_topic].0)
                                    .size(22.0)
                                    .strong()
                                    .color(INK),
                            );
                            ui.add_space(12.0);
                            ui.label(topics[self.help_topic].1);
                            if self.help_topic == 0 {
                                for (i, (label, body)) in [
                                    (
                                        crate::i18n::tr("Добавьте запись"),
                                        crate::i18n::tr("Заполните основные данные ученика."),
                                    ),
                                    (
                                        crate::i18n::tr("Прикрепите фото"),
                                        crate::i18n::tr(
                                            "Фотография поможет быстрее найти человека.",
                                        ),
                                    ),
                                    (
                                        crate::i18n::tr("Посмотрите таблицу"),
                                        crate::i18n::tr(
                                            "Все записи будут доступны в одном списке.",
                                        ),
                                    ),
                                    (
                                        crate::i18n::tr("Создайте копию"),
                                        crate::i18n::tr("Сохраните базу вместе с вложениями."),
                                    ),
                                ]
                                .iter()
                                .enumerate()
                                {
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(
                                            egui::vec2(34.0, 48.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(r.center(), 17.0, BLUE);
                                        paint_centered_text(
                                            ui.painter(),
                                            r.center(),
                                            &(i + 1).to_string(),
                                            egui::FontId::proportional(15.0),
                                            Color32::WHITE,
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(*label).strong().color(INK));
                                            ui.label(RichText::new(*body).size(14.0).color(MUTED));
                                        });
                                    });
                                }
                            }
                        });
                    },
                );
            },
        );
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.pending.is_some() {
            egui::Window::new(crate::i18n::tr("Несохранённые изменения"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .default_width(440.0)
                .max_width((ctx.screen_rect().width() - 48.0).min(560.0))
                .max_height((ctx.screen_rect().height() - 72.0).max(180.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("modal-content-scroll")
                        .max_height((ctx.screen_rect().height() - 160.0).max(140.0))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
                            ui.set_min_width((ctx.screen_rect().width() - 80.0).min(520.0));
                            ui.label(crate::i18n::tr(
                                "В карточке есть изменения, которые ещё не сохранены.",
                            ));
                            ui.label(crate::i18n::tr("Что сделать перед переходом?"));
                            ui.add_space(12.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .add(primary_button(crate::i18n::tr("Сохранить и продолжить")))
                                    .clicked()
                                {
                                    if self.save_record() {
                                        if let Some(action) = self.pending.take() {
                                            self.finish_pending(action, ctx);
                                        }
                                    }
                                }
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Не сохранять")))
                                    .clicked()
                                {
                                    self.discard_changes();
                                    if let Some(action) = self.pending.take() {
                                        self.finish_pending(action, ctx);
                                    }
                                }
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Отмена")))
                                    .clicked()
                                {
                                    self.pending = None;
                                }
                            });
                        })
                        .inner
                });
        }
        if self.delete_record_confirm {
            egui::Window::new(crate::i18n::tr("Удалить запись?"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .default_width(440.0)
                .max_width((ctx.screen_rect().width() - 48.0).min(560.0))
                .max_height((ctx.screen_rect().height() - 72.0).max(180.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("modal-content-scroll")
                        .max_height((ctx.screen_rect().height() - 160.0).max(140.0))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
                            ui.label(crate::i18n::tr(
                                "Запись будет удалена из базы. Файлы останутся в папке программы.",
                            ));
                            ui.label(crate::i18n::tr(
                                "Рекомендуем сначала создать резервную копию.",
                            ));
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .add(danger_button(crate::i18n::tr("Удалить запись")))
                                    .clicked()
                                {
                                    let id = self.selected.id;
                                    match self
                                        .db()
                                        .and_then(|db| db.delete_record(self.active_table, id))
                                    {
                                        Ok(()) => {
                                            self.selected =
                                                Record::new(self.active_table, &self.fields);
                                            self.dirty = false;
                                            self.refresh_data();
                                            self.notify(crate::i18n::tr("Запись удалена"));
                                        }
                                        Err(e) => self.fail(e),
                                    }
                                    self.delete_record_confirm = false;
                                }
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Отмена")))
                                    .clicked()
                                {
                                    self.delete_record_confirm = false;
                                }
                            });
                        })
                        .inner
                });
        }
        if self.create_table_dialog {
            egui::Window::new(crate::i18n::tr("Новая таблица"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .default_width(440.0)
                .max_width((ctx.screen_rect().width() - 48.0).min(560.0))
                .max_height((ctx.screen_rect().height() - 72.0).max(180.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("modal-content-scroll")
                        .max_height((ctx.screen_rect().height() - 160.0).max(140.0))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
                            ui.label(crate::i18n::tr("Как назвать новую таблицу?"));
                            ui.add_sized(
                                [ui.available_width(), 40.0],
                                egui::TextEdit::singleline(&mut self.new_table_name)
                                    .margin(egui::vec2(12.0, 8.0))
                                    .vertical_align(egui::Align::Center)
                                    .hint_text(crate::i18n::tr("Например, учителя")),
                            );
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .add(primary_button(crate::i18n::tr("Создать таблицу")))
                                    .clicked()
                                {
                                    let result = self
                                        .db()
                                        .and_then(|db| db.create_table(&self.new_table_name));
                                    match result {
                                        Ok(id) => {
                                            self.new_table_name.clear();
                                            self.create_table_dialog = false;
                                            self.active_table = id;
                                            self.selected = Record::new(id, &[]);
                                            self.refresh_tables();
                                            self.notify(crate::i18n::tr(
                                                "Таблица создана. Теперь добавьте поля.",
                                            ));
                                        }
                                        Err(e) => self.fail(e),
                                    }
                                }
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Отмена")))
                                    .clicked()
                                {
                                    self.create_table_dialog = false;
                                }
                            });
                        })
                        .inner
                });
        }
        if self.add_field_dialog {
            let is_new = self.field_editor.id == 0;
            egui::Window::new(if is_new {
                crate::i18n::tr("Добавить поле")
            } else {
                crate::i18n::tr("Настроить поле")
            })
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .default_width(440.0)
            .max_width((ctx.screen_rect().width() - 48.0).min(560.0))
            .max_height((ctx.screen_rect().height() - 72.0).max(180.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("modal-content-scroll")
                    .max_height((ctx.screen_rect().height() - 160.0).max(140.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
                        ui.set_min_width((ctx.screen_rect().width() - 80.0).min(440.0));
                        ui.label(crate::i18n::tr("Название поля"));
                        ui.add_sized(
                            [ui.available_width(), 40.0],
                            egui::TextEdit::singleline(&mut self.field_editor.name)
                                .margin(egui::vec2(12.0, 8.0))
                                .vertical_align(egui::Align::Center)
                                .hint_text(crate::i18n::tr("Например, любимый предмет")),
                        );
                        ui.add_space(8.0);
                        ui.label(crate::i18n::tr("Что будем здесь хранить?"));
                        egui::ComboBox::from_id_salt("field-kind")
                            .width(ui.available_width())
                            .selected_text(self.field_editor.kind.label())
                            .show_ui(ui, |ui| {
                                for kind in FieldKind::ALL {
                                    ui.selectable_value(
                                        &mut self.field_editor.kind,
                                        kind,
                                        kind.label(),
                                    );
                                }
                            });
                        ui.add_space(8.0);
                        quiet_checkbox(
                            ui,
                            &mut self.field_editor.required,
                            crate::i18n::tr("Обязательно заполнять"),
                        );
                        quiet_checkbox(
                            ui,
                            &mut self.field_editor.visible,
                            crate::i18n::tr("Показывать в карточке и таблице"),
                        );
                        ui.label(crate::i18n::tr("Значение по умолчанию (необязательно)"));
                        ui.text_edit_singleline(&mut self.field_editor.default_value);
                        ui.label(crate::i18n::tr("Ширина столбца таблицы"));
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 40.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(180.0, 22.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().interact_size.y = 22.0;
                                        ui.spacing_mut().slider_width = 180.0;
                                        ui.visuals_mut().widgets.inactive.bg_fill = BLUE;
                                        ui.visuals_mut().widgets.hovered.bg_fill = BLUE;
                                        ui.visuals_mut().widgets.active.bg_fill = BLUE;
                                        ui.visuals_mut().widgets.inactive.bg_stroke =
                                            egui::Stroke::NONE;
                                        ui.visuals_mut().widgets.inactive.fg_stroke =
                                            egui::Stroke::NONE;
                                        ui.visuals_mut().widgets.hovered.fg_stroke =
                                            egui::Stroke::NONE;
                                        ui.visuals_mut().widgets.active.fg_stroke =
                                            egui::Stroke::NONE;
                                        ui.visuals_mut().slider_trailing_fill = false;
                                        ui.add(
                                            egui::Slider::new(
                                                &mut self.field_editor.width,
                                                100..=600,
                                            )
                                            .show_value(false),
                                        );
                                    },
                                );
                                ui.add_sized(
                                    [80.0, 36.0],
                                    egui::DragValue::new(&mut self.field_editor.width)
                                        .range(100..=600)
                                        .suffix(" px"),
                                );
                            },
                        );
                        ui.add_space(12.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui
                                .add(primary_button(if is_new {
                                    crate::i18n::tr("Добавить поле")
                                } else {
                                    crate::i18n::tr("Сохранить поле")
                                }))
                                .clicked()
                            {
                                self.save_field();
                            }
                            if !is_new
                                && ui
                                    .add(danger_button(crate::i18n::tr("Удалить поле")))
                                    .clicked()
                            {
                                self.delete_field_confirm = true;
                            }
                            if ui
                                .add(secondary_button(crate::i18n::tr("Отмена")))
                                .clicked()
                            {
                                self.add_field_dialog = false;
                            }
                        });
                    })
                    .inner
            });
        }
        if self.delete_field_confirm {
            egui::Window::new(crate::i18n::tr("Удалить поле и его данные?")).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0])
                .collapsible(false).resizable(false).auto_sized()
                .default_width(440.0)
                .max_width((ctx.screen_rect().width()-48.0).min(560.0)).show(ctx,|ui|{
                    egui::ScrollArea::vertical().id_salt("modal-content-scroll")
                        .max_height((ctx.screen_rect().height()-160.0).max(140.0))
                        .auto_shrink([false,true]).show(ui,|ui|{ui.spacing_mut().item_spacing=egui::vec2(16.0,12.0);
                    ui.label(match crate::i18n::current(){crate::i18n::Language::Ru=>format!("Поле «{}» будет удалено из всех записей. Это нельзя отменить без резервной копии.",self.field_editor.name),crate::i18n::Language::De=>format!("Feld „{}“ wird aus allen Datensätzen gelöscht. Ohne Sicherung ist das nicht rückgängig zu machen.",self.field_editor.name),crate::i18n::Language::En=>format!("Field “{}” will be deleted from all records. This cannot be undone without a backup.",self.field_editor.name)});
                    ui.horizontal_wrapped(|ui|{
                        if ui.add(danger_button(crate::i18n::tr("Удалить навсегда"))).clicked(){
                            let result=self.db().and_then(assets::create_backup)
                                .and_then(|_|self.db().and_then(|db|db.delete_field(self.field_editor.id,self.active_table)));
                            match result {
                                Ok(())=>{self.add_field_dialog=false;self.delete_field_confirm=false;self.selected_field=None;self.refresh_data();self.notify(crate::i18n::tr("Поле удалено"));},
                                Err(e)=>self.fail(e),
                            }
                        }
                        if ui.add(secondary_button(crate::i18n::tr("Отмена"))).clicked(){self.delete_field_confirm=false;}
                    });

                        }).inner
                });
        }
        if let Some(path) = self.restore_confirm.clone() {
            egui::Window::new(crate::i18n::tr("Восстановить резервную копию?"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .default_width(440.0)
                .max_width((ctx.screen_rect().width() - 48.0).min(560.0))
                .max_height((ctx.screen_rect().height() - 72.0).max(180.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("modal-content-scroll")
                        .max_height((ctx.screen_rect().height() - 160.0).max(140.0))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
                            ui.label(crate::i18n::tr(
                                "Текущая база будет заменена данными из выбранной копии.",
                            ));
                            ui.label(crate::i18n::tr(
                                "Перед заменой программа автоматически сохранит защитную копию.",
                            ));
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .add(primary_button(crate::i18n::tr("Восстановить")))
                                    .clicked()
                                {
                                    self.restore_confirm = None;
                                    self.request(Action::Restore(path.clone()));
                                }
                                if ui
                                    .add(secondary_button(crate::i18n::tr("Отмена")))
                                    .clicked()
                                {
                                    self.restore_confirm = None;
                                }
                            });
                        })
                        .inner
                });
        }
    }
    fn finish_pending(&mut self, action: Action, ctx: &egui::Context) {
        if matches!(action, Action::Exit) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            self.apply_action(action);
        }
    }
}

// Read modifiers from the key event: Ctrl may already be released this frame.
fn shortcut_pressed(input: &egui::InputState, wanted: egui::Key) -> bool {
    input.events.iter().any(|event| {
        matches!(event,
            egui::Event::Key { key, physical_key, pressed: true, repeat: false, modifiers }
                if (modifiers.command || modifiers.ctrl) && !modifiers.alt && !modifiers.shift
                    && (*key == wanted || *physical_key == Some(wanted))
        )
    })
}

fn display_data_path(path: &std::path::Path) -> String {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        if let Ok(relative) = path.strip_prefix(PathBuf::from(local)) {
            return format!("%LOCALAPPDATA%\\{}", relative.display());
        }
    }
    path.display().to_string()
}

fn sidebar_after_drag(initial: Sidebar, delta: f32) -> Sidebar {
    match initial {
        Sidebar::Full if delta < -136.0 => Sidebar::Hidden,
        Sidebar::Full if delta < -48.0 => Sidebar::Icons,
        Sidebar::Icons if delta > 64.0 => Sidebar::Full,
        Sidebar::Icons if delta < -32.0 => Sidebar::Hidden,
        Sidebar::Hidden if delta > 144.0 => Sidebar::Full,
        Sidebar::Hidden if delta > 32.0 => Sidebar::Icons,
        _ => initial,
    }
}

impl eframe::App for Kartoteka {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(BG).to_array()
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("interface-language"),
                self.prefs.language.code(),
            )
        });
        crate::i18n::set(self.prefs.language);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            crate::i18n::tr("Одноклассники — Картотека").to_owned(),
        ));
        // Optional local QA capture: the native renderer supplies the actual
        // framebuffer. No capture or file write occurs in ordinary launches.
        if let Some(folder) = std::env::var_os("KARTOTEKA_REVIEW_DIR") {
            let folder = PathBuf::from(folder);
            let stamp = format!(
                "{:?}-{}-{}-{}-{}-{}-{}-{}-{}-{:?}-{}-{:?}",
                self.screen,
                self.pending.is_some(),
                self.delete_record_confirm,
                self.delete_field_confirm,
                self.restore_confirm.is_some(),
                self.create_table_dialog,
                self.add_field_dialog,
                self.field_editor.id,
                self.settings_topic,
                self.effective_sidebar(),
                self.prefs.zoom,
                ctx.screen_rect().size()
            );
            let stamp = format!("{}-{}", stamp, self.prefs.language.code());
            let changed = ctx.data_mut(|d| {
                let id = egui::Id::new("review-last-frame-state");
                if d.get_temp::<String>(id).as_ref() != Some(&stamp) {
                    d.insert_temp(id, stamp);
                    true
                } else {
                    false
                }
            });
            if changed || ctx.input(|i| i.key_pressed(egui::Key::F12)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            let screenshots = ctx.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| {
                        if let egui::Event::Screenshot { image, .. } = e {
                            Some(image.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            });
            for image in screenshots {
                let name = if self.delete_field_confirm {
                    "26-dialog-delete-field"
                } else if self.pending.is_some() {
                    "24-dialog-unsaved"
                } else if self.delete_record_confirm {
                    "23-dialog-delete-record"
                } else if self.restore_confirm.is_some() {
                    "22-dialog-restore"
                } else if self.add_field_dialog {
                    if self.field_editor.id == 0 {
                        "21-dialog-add-field"
                    } else {
                        "25-dialog-edit-field"
                    }
                } else if self.create_table_dialog {
                    "20-dialog-new-table"
                } else {
                    match self.screen {
                        Screen::Home => "01-home",
                        Screen::Card => "02-card",
                        Screen::Table => "03-table",
                        Screen::Search => "04-search",
                        Screen::Designer => "05-designer",
                        Screen::Forms => "06-forms",
                        Screen::Backups => "07-backups",
                        Screen::Settings => "08-settings",
                        Screen::Help => "09-help",
                    }
                };
                let bytes = image
                    .pixels
                    .iter()
                    .flat_map(|p| p.to_array())
                    .collect::<Vec<_>>();
                let result = std::fs::create_dir_all(&folder)
                    .map_err(|e| e.to_string())
                    .and_then(|_| {
                        let frame = image::RgbaImage::from_raw(
                            image.size[0] as u32,
                            image.size[1] as u32,
                            bytes,
                        )
                        .ok_or(crate::i18n::tr("Неверный размер снимка"))?;
                        let mut path = folder.join(format!("{name}.png"));
                        let mut index = 2;
                        while path.exists() {
                            path = folder.join(format!("{name}-{index}.png"));
                            index += 1;
                        }
                        frame.save(path).map_err(|e| e.to_string())
                    });
                if let Err(error) = result {
                    self.fail(match crate::i18n::current() {
                        crate::i18n::Language::Ru => {
                            format!("Не удалось сохранить снимок проверки: {error}")
                        }
                        crate::i18n::Language::De => {
                            format!("Prüfbild konnte nicht gespeichert werden: {error}")
                        }
                        crate::i18n::Language::En => {
                            format!("Could not save review capture: {error}")
                        }
                    });
                }
            }
        }
        // Клавиатурные сочетания дублируют, а не заменяют видимые кнопки.
        let (save, create, find) = ctx.input(|input| {
            (
                shortcut_pressed(input, egui::Key::S),
                shortcut_pressed(input, egui::Key::N),
                shortcut_pressed(input, egui::Key::F),
            )
        });
        if self.pending.is_none()
            && !self.add_field_dialog
            && !self.create_table_dialog
            && !self.delete_field_confirm
            && !self.delete_record_confirm
            && self.restore_confirm.is_none()
        {
            if save && self.screen == Screen::Card {
                self.save_record();
            } else if create {
                self.request(Action::NewRecord);
            } else if find {
                self.focus_search = true;
                self.request(Action::Screen(Screen::Search));
            }
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.pending.is_none() {
                self.pending = Some(Action::Exit);
            }
        }
        self.update_responsive_shell(ctx);
        self.topbar(ctx);
        self.statusbar(ctx);
        self.sidebar(ctx);
        self.main_area(ctx);
        self.dialogs(ctx);
    }
}
