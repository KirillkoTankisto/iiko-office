use gtk4::Application;

const APP_ID: &str = "org.fargo.iiko-office";

pub fn create_app() -> Application {
    gtk4::Application::builder().application_id(APP_ID).build()
}
