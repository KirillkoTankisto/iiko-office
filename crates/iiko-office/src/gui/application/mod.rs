const APP_ID: &str = "org.fargo.iiko-office-libre";

pub struct IikoOffice;

impl IikoOffice {
    pub fn build() -> gtk4::Application {
        gtk4::Application::builder().application_id(APP_ID).build()
    }
}
