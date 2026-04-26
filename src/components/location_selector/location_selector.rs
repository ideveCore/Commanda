/* location_selector.rs
 *
 * Copyright 2026 Ideve Core
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::components::location_row::LocationRow;
use crate::services::weather::{LocationInfo, WeatherService};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(
        resource = "/io/github/idevecore/Commanda/components/location_selector/location_selector.ui"
    )]
    pub struct LocationSelector {
        #[template_child]
        pub button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub insight: TemplateChild<gtk::Label>,
        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub location_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub popover: TemplateChild<gtk::Popover>,

        pub selected: RefCell<String>,
        pub weather_service: RefCell<Option<WeatherService>>,
        pub runtime_handle: RefCell<Option<tokio::runtime::Handle>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocationSelector {
        const NAME: &'static str = "LocationSelector";
        type Type = super::LocationSelector;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LocationSelector {
        fn constructed(&self) {
            self.parent_constructed();
            self.search
                .set_key_capture_widget(Some(self.popover.upcast_ref::<gtk::Widget>()));
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: std::sync::OnceLock<Vec<glib::subclass::Signal>> =
                std::sync::OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![glib::subclass::Signal::builder("user-selection-changed").build()]
            })
        }
    }

    impl WidgetImpl for LocationSelector {}
    impl BinImpl for LocationSelector {}

    #[gtk::template_callbacks]
    impl LocationSelector {
        #[template_callback]
        fn on_row_activated(&self, row: &gtk::ListBoxRow) {
            self.popover.popdown();
            // TODO: extrair LocationInfo do row e salvar em selected
            self.obj().emit_by_name::<()>("user-selection-changed", &[]);
        }

        #[template_callback]
        fn on_popover_show(&self) {
            self.search.grab_focus();
        }

        #[template_callback]
        fn on_popover_closed(&self) {
            self.search.set_text("");
        }

        #[template_callback]
        fn on_search_changed(&self) {
            let query = self.search.text().to_string();
            if query.is_empty() {
                return;
            }

            let service = self.weather_service.borrow().clone();
            let Some(service) = service else { return };

            let handle = self.runtime_handle.borrow().clone();
            let Some(handle) = handle else { return };

            let obj = self.obj().clone();
            let list = self.location_list.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            // Roda no runtime Tokio (thread separada)
            handle.spawn(async move {
                let _ = tx.send(service.get_locations(&query).await);
            });

            // Faz polling na thread GTK até o resultado chegar
            glib::idle_add_local(move || match rx.try_recv() {
                Ok(Ok(locations)) => {
                    obj.imp().populate_list(&list, locations);
                    glib::ControlFlow::Break
                }
                Ok(Err(e)) => {
                    eprintln!("Erro ao buscar locais: {e}");
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            });
        }

        #[template_callback]
        fn on_search_activate(&self) {
            if !self.search.text().is_empty() {
                if let Some(row) = self.location_list.row_at_index(0) {
                    self.location_list
                        .emit_by_name::<()>("row-activated", &[&row]);
                }
            }
        }

        pub fn populate_list(&self, list: &gtk::ListBox, locations: Vec<LocationInfo>) {
            while let Some(row) = list.row_at_index(0) {
                list.remove(&row);
            }
            for loc in locations {
                let row = LocationRow::new(&loc);
                list.append(&row);
            }
        }
    }
}

glib::wrapper! {
    pub struct LocationSelector(ObjectSubclass<imp::LocationSelector>)
        @extends gtk::Widget, adw::Bin;
}

impl LocationSelector {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_weather_service(&self, service: WeatherService, handle: tokio::runtime::Handle) {
        *self.imp().weather_service.borrow_mut() = Some(service);
        *self.imp().runtime_handle.borrow_mut() = Some(handle);
    }

    pub fn selected_json(&self) -> String {
        self.imp().selected.borrow().clone()
    }
}

impl Default for LocationSelector {
    fn default() -> Self {
        Self::new()
    }
}
