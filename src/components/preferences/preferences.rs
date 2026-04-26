/* preferences.rs
 *
 * Copyright 2026 Ideve Core
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::components::location_selector::LocationSelector;
use crate::services::weather::WeatherService;
use adw::subclass::prelude::*;
use gtk::{
    gio::{self, prelude::SettingsExtManual},
    glib::{self, object::ObjectExt},
};

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/idevecore/Commanda/components/preferences/preferences.ui")]
    pub struct Preferences {
        #[template_child]
        pub location_selector: TemplateChild<LocationSelector>,
        #[template_child]
        pub ip_location_switch: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Preferences {
        const NAME: &'static str = "Preferences";
        type Type = super::Preferences;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Preferences {}
    impl WidgetImpl for Preferences {}
    impl AdwDialogImpl for Preferences {}
    impl PreferencesDialogImpl for Preferences {}
}

glib::wrapper! {
    pub struct Preferences(ObjectSubclass<imp::Preferences>)
        @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog;
}

impl Preferences {
    pub fn new(app_id: &str) -> Self {
        let obj: Self = glib::Object::builder().build();

        obj.bind_weather_properties(&app_id);
        obj
    }

    pub fn set_weather_service(&self, service: WeatherService, handle: tokio::runtime::Handle) {
        self.imp()
            .location_selector
            .set_weather_service(service, handle);
    }

    pub fn bind_weather_properties(&self, app_id: &str) {
        let web_app_id = format!("{}.Weather", app_id);
        let settings = gio::Settings::new(&web_app_id);

        settings
            .bind(
                "use-ip-location",
                &self.imp().ip_location_switch.get(),
                "active",
            )
            .build();
    }
}
