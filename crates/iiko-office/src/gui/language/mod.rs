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
    let locale = env::var("LANGUAGE")
        .or_else(|_| env::var("LC_ALL"))
        .or_else(|_| env::var("LC_MESSAGES"))
        .or_else(|_| env::var("LANG"))
        .ok()
        .or_else(sys_locale::get_locale)
        .unwrap_or_default();

    let primary = locale
        .split(['-', '_', '.', ':'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match primary.as_str() {
        "ru" => CurrentLanguage::RU,
        _ => CurrentLanguage::EN,
    }
}
