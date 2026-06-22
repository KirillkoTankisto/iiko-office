use std::{collections::HashMap, sync::Arc};

use gtk4::{
    Align,
    glib::{BoxedAnyObject, object::Cast},
    prelude::BoxExt,
};

use crate::{
    api::olap_columns::{OlapColumn, OlapColumnRequest, ReportType},
    gui::{
        GlobalData,
        common::{datepicker::DatePicker, table::{AnyTable, AnyTableColumn}},
        main::menu::{
            tabs::{AnyTab, build_box, build_hbox},
            view::MainView,
        },
        translation::{
            Line::{DATE_FROM, DATE_TO, OLAP_FIELDS, OLAP_REPORTS},
            translate,
        },
    },
};

pub struct OlapReportsTab;

impl AnyTab for OlapReportsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), OLAP_REPORTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let olap_box = build_box();

        let table = AnyTable::new();

        let grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let date_from = DatePicker::new(translate(gdata.language(), DATE_FROM), gdata.language());
        let date_to = DatePicker::new(translate(gdata.language(), DATE_TO), gdata.language());

        date_from.attach_to(&grid, 0);
        date_to.attach_to(&grid, 1);

        olap_box.append(&grid);

        let content = build_hbox();

        olap_box.append(&content);

        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), OLAP_FIELDS),
            Align::Start,
            false,
            |p: &(String, OlapColumn)| p.1.name.clone(),
        ));

        table.set_row_drag(|p: &(String, OlapColumn)| p.0.clone());

        content.append(table.present());

        let (sender, receiver) =
            async_channel::bounded::<Result<HashMap<String, OlapColumn>, String>>(1);

        std::thread::spawn(move || {
            if let Some(udata) = gdata.get_credentials() {
                let _ = sender.send_blocking(
                    OlapColumnRequest::new(udata.address, udata.token, ReportType::SALES).run(),
                );
            } else {
                let _ = sender.send_blocking(Err("Couldn't lock gdata".to_string()));
            }
        });

        gtk4::glib::spawn_future_local(async move {
            if let Ok(received) = receiver.recv().await {
                match received {
                    Ok(columns) => {
                        for column in columns {
                            table.add_object(&BoxedAnyObject::new(column));
                        }
                    }
                    Err(err) => eprintln!("{err}"),
                }
            }
        });

        olap_box.upcast()
    }
}
