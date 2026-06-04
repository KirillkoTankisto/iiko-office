type Translations = &'static [&'static [&'static str]];

#[derive(Clone)]
pub enum CurrentLanguage {
    EN,
    RU,
}

#[allow(nonstandard_style)]
pub enum Line {
    ADDRESS,
    USERNAME,
    PASSWORD,
    LOGIN,
    LOGOUT,
    FILE,
    CLOSE,
    DATE_FROM,
    DATE_TO,
    CASH_SHIFTS,
}

const TRANSLATIONS: Translations = &[
    &[
        "Server Address",
        "Username",
        "Password",
        "Login",
        "Logout",
        "File",
        "Close",
        "Date From",
        "Date To",
        "Cash Shifts",
    ],
    &[
        "Адрес Сервера",
        "Имя Пользователя",
        "Пароль",
        "Войти",
        "Выйти",
        "Файл",
        "Закрыть",
        "Дата От",
        "Дата До",
        "Кассовые Смены",
    ],
];

pub fn translate(language: CurrentLanguage, line: Line) -> &'static str {
    TRANSLATIONS[language as usize][line as usize]
}
