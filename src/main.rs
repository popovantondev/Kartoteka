#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod assets;
mod brand;
mod db;
mod i18n;
mod ui;

fn main() -> eframe::Result<()> {
    let language = std::fs::read(assets::default_data_dir().join("preferences.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value.get("language").cloned())
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    i18n::set(language);
    #[cfg(target_os = "windows")]
    let Some(_instance) = single_instance::acquire() else {
        return Ok(());
    };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_icon(app_icon())
            .with_title(crate::i18n::tr("Одноклассники — Картотека")),
        ..Default::default()
    };
    eframe::run_native(
        crate::i18n::tr("Одноклассники — Картотека"),
        options,
        Box::new(|cc| Ok(Box::new(ui::Kartoteka::new(cc)))),
    )
}

#[cfg(target_os = "windows")]
mod single_instance {
    use std::ffi::c_void;
    use std::hash::{Hash, Hasher};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateMutexW(
            attributes: *const c_void,
            initial_owner: i32,
            name: *const u16,
        ) -> *mut c_void;
        fn GetLastError() -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(window: *mut c_void, text: *const u16, title: *const u16, kind: u32) -> i32;
    }
    pub struct Guard(*mut c_void);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    pub fn acquire() -> Option<Guard> {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        crate::assets::default_data_dir().hash(&mut hash);
        let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let name = wide(&format!(
            "Local\\Popovantondev.Kartoteka.{:016x}",
            hash.finish()
        ));
        // Named mutex lifetime is managed by Windows, so a crash cannot leave
        // a stale lock file. Separate QA data roots do not block a user's app.
        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
            if handle.is_null() {
                return Some(Guard(handle));
            }
            if GetLastError() == 183 {
                CloseHandle(handle);
                MessageBoxW(
                    std::ptr::null_mut(),
                    wide(crate::i18n::tr(
                        "Картотека с этой базой уже открыта. Переключитесь в существующее окно.",
                    ))
                    .as_ptr(),
                    wide(crate::i18n::tr("Одноклассники — Картотека")).as_ptr(),
                    0x40,
                );
                None
            } else {
                Some(Guard(handle))
            }
        }
    }
}

fn app_icon() -> eframe::egui::IconData {
    eframe::egui::IconData {
        rgba: brand::rgba(64),
        width: 64,
        height: 64,
    }
}
