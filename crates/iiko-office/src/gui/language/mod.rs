use crate::gui::translation::CurrentLanguage;
use std::env;

/// Should always be called before anything
pub fn set_language() {
    if env::var_os("LANG").is_none()
        && env::var_os("LC_ALL").is_none()
        && let Some(locale) = sys_locale::get_locale()
    {
        unsafe {
            env::set_var("LC_ALL", &locale);
            env::set_var("LANG", &locale);
        }
    }
}

pub fn get_language() -> CurrentLanguage {
    let locale_full = gtk4::default_language().to_string();

    let primary = locale_full
        .split(['-', '_', '.'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match primary.as_str() {
        "ru" => CurrentLanguage::RU,
        _ => CurrentLanguage::EN,
    }
}
