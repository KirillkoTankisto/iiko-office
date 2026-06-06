use gtk4::glib::{DateTime, TimeZone};

pub fn reformat_date(some_str: &Option<String>) -> String
{
    if let Some(s) = some_str
        && let Ok(datetime) = DateTime::from_iso8601(s, Some(&TimeZone::local()))
        && let Ok(gstr) = datetime.format("%d.%m.%Y %H:%M")
    {
        gstr.into()
    }
    else
    {
        "...".into()
    }
}
