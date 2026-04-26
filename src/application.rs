/* application.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::components::preferences::Preferences;
use crate::config::VERSION;
use crate::server::{spawn_server, SharedState};
use crate::CommandaWindow;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct CommandaApplication {
        pub state: std::cell::OnceCell<SharedState>,
    }

    impl Default for CommandaApplication {
        fn default() -> Self {
            Self {
                state: std::cell::OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandaApplication {
        const NAME: &'static str = "CommandaApplication";
        type Type = super::CommandaApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for CommandaApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for CommandaApplication {
        fn activate(&self) {
            let application = self.obj();
            let app_id = application.application_id().expect("");

            self.state.get_or_init(|| spawn_server(8080, &app_id));

            let window = application.active_window().unwrap_or_else(|| {
                let window = CommandaWindow::new(&*application);
                let gsettings = gio::Settings::new(&app_id);
                gsettings.bind("width", &window, "default-width").build();
                gsettings.bind("height", &window, "default-height").build();
                gsettings.bind("is-maximized", &window, "maximized").build();
                gsettings
                    .bind("is-fullscreen", &window, "fullscreened")
                    .build();
                window.upcast()
            });
            window.present();
        }
    }

    impl GtkApplicationImpl for CommandaApplication {}
    impl AdwApplicationImpl for CommandaApplication {}
}

glib::wrapper! {
    pub struct CommandaApplication(ObjectSubclass<imp::CommandaApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CommandaApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/io/github/idevecore/Commanda")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| app.show_preferences())
            .build();
        self.set_accels_for_action("app.preferences", &["<control>comma"]);
        self.add_action_entries([quit_action, about_action, preferences_action]);
    }

    fn show_preferences(&self) {
        let window = self.active_window().unwrap();
        let state = self.imp().state.get().expect("state not initialized");
        let app_id = self.application_id().expect("");
        let preferences = Preferences::new(&app_id);
        preferences.set_weather_service(state.weather.clone(), state.runtime_handle.clone());
        preferences.present(Some(&window));
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("Commanda")
            .application_icon("io.github.idevecore.Commanda")
            .developer_name("Francisco")
            .version(VERSION)
            .developers(vec!["Francisco"])
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2026 Francisco")
            .build();
        about.present(Some(&window));
    }
}
