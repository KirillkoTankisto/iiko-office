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
        common::table::{AnyTable, AnyTableColumn},
        main::menu::{
            tabs::{AnyTab, build_box},
            view::MainView,
        },
        translation::{
            Line::{OLAP_FIELDS, OLAP_REPORTS},
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
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), OLAP_FIELDS),
            Align::Start,
            false,
            |p: &(String, OlapColumn)| p.1.name.clone(),
        ));

        olap_box.append(table.present());

        let (sender, receiver) =
            async_channel::bounded::<Result<HashMap<String, OlapColumn>, String>>(1);

        std::thread::spawn(move || {
            if let Some((address, token)) = {
                if let Ok(locked) = gdata.user_data.lock() {
                    Some((locked.address.clone(), locked.token.clone()))
                } else {
                    None
                }
            } {
                let _ = sender
                    .send_blocking(OlapColumnRequest::new(address, token, ReportType::SALES).run());
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
