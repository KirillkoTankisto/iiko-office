use gtk4::glib::{DateTime, TimeZone};

pub fn reformat_date(value: Option<&str>) -> String {
    if let Some(s) = value
        && let Ok(datetime) = DateTime::from_iso8601(s, Some(&TimeZone::local()))
        && let Ok(gstr) = datetime.format("%d.%m.%Y %H:%M")
    {
        gstr.into()
    } else {
        "...".into()
    }
}
