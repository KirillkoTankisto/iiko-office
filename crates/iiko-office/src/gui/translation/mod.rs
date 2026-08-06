use gtk4::glib;

#[derive(Clone, Copy, glib::Downgrade)]
pub enum CurrentLanguage {
    EN,
    RU,
}

/// Declares the line along with its translation
macro_rules! lines {
    ($($name:ident => $en:literal, $ru:literal;)*) => {
        #[derive(Clone, Copy)]
        #[allow(nonstandard_style)]
        pub enum Line {
            $($name,)*
        }

        const TRANSLATIONS: &[&[&str]] = &[&[$($en,)*], &[$($ru,)*]];
    };
}

lines! {
    ABOUT_COMMENT         => "iikoOffice Open-Source alternative for Linux and macOS", "Альтернатива iikoOffice для Linux и macOS с открытым исходным кодом";
    ABOUT_SOURCE_CODE     => "Source Code", "Исходный Код";
    LOGIN_ADD_SERVER      => "Add Server", "Добавить Сервер";
    LOGIN_REMOVE_SERVER   => "Remove Server", "Убрать Сервер";
    LOGIN_ADDRESS         => "Server Address", "Адрес Сервера";
    LOGIN_USERNAME        => "Username", "Имя Пользователя";
    LOGIN_PASSWORD        => "Password", "Пароль";
    LOGIN                 => "Login", "Войти";
    MENUBAR_LOGOUT        => "Logout", "Выйти";
    MENUBAR_ABOUT         => "About", "О Программе";
    MENUBAR_FILE          => "File", "Файл";
    CLOSE                 => "Close", "Закрыть";
    DATE_FROM             => "Date From", "Дата От";
    DATE_TO               => "Date To", "Дата До";
    CASH_SHIFTS           => "Cash Shifts", "Кассовые Смены";
    REFRESH               => "Refresh", "Обновить";
    OPEN_DATE             => "Open Date", "Дата Открытия";
    CLOSE_DATE            => "Close Date", "Дата Закрытия";
    ACCEPT_DATE           => "Accept Date", "Дата Принятия";
    SALES_SUM             => "Sales Summary", "Сумма";
    SALES_CARD            => "Sales Card", "Оплачено Картой";
    SALES_CASH            => "Sales Cash", "Оплачено Наличными";
    SALES_CREDIT          => "Sales Credit", "Кредит";
    SHIFT_NUMBER          => "Shift Number", "Номер Смены";
    PAYMENTS              => "Payments", "Платежи";
    DATE                  => "Date", "Дата";
    GROUP                 => "Group", "Группа";
    SUM                   => "Sum", "Сумма";
    OLAP_REPORTS          => "OLAP Reports", "Отчёты OLAP";
    OLAP_FIELDS           => "OLAP Fields", "Поля OLAP";
    OLAP_ROW_FIELDS       => "Row Fields", "Поля Строк";
    OLAP_COLUMN_FIELDS    => "Column Fields", "Поля Столбцов";
    OLAP_AGGREGATE_FIELDS => "Aggregate Fields", "Поля Агрегации";
    ERROR_ADDRESS         => "Invalid Address", "Неверный Адрес";
    ERROR_INTERNAL        => "Internal Error", "Внутренняя Ошибка";
    ERROR_REQUEST         => "Request Failed", "Ошибка Соединения с Сервером";
    ERROR_RESPONSE        => "Failed to Parse Response", "Не Удалось Обработать Ответ от Сервера";
    ERROR_UNAUTHORIZED    => "Failed to Authorize", "Не Удалось Авторизоваться";
    PERIOD_CUSTOM         => "Custom Period", "Свой Период";
    PERIOD_OPEN           => "Open Period", "Период Открытия";
    PERIOD_TODAY          => "Today", "Сегодня";
    PERIOD_YESTERDAY      => "Yesterday", "Вчера";
    PERIOD_CURRENT_WEEK   => "Current Week", "Текущая Неделя";
    PERIOD_CURRENT_MONTH  => "Current Month", "Текущий Месяц";
    PERIOD_CURRENT_YEAR   => "Current Year", "Текущий Год";
    PERIOD_LAST_WEEK      => "Last Week", "Предыдущая Неделя";
    PERIOD_LAST_MONTH     => "Last Month", "Предыдущий Месяц";
    PERIOD_LAST_YEAR      => "Last Year", "Предыдущий Год";
    TOTAL                 => "Total", "Итого";
}

pub fn translate(language: CurrentLanguage, line: Line) -> &'static str {
    TRANSLATIONS[language as usize][line as usize]
}
