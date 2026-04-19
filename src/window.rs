/* window.rs
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

use crate::components::home::Home;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/idevecore/Commanda/window.ui")]
    pub struct CommandaWindow {
        // Template widgets
        #[template_child]
        pub content: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandaWindow {
        const NAME: &'static str = "CommandaWindow";
        type Type = super::CommandaWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CommandaWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let home_page = Home::new();
            self.obj().set_content(&home_page);
        }
    }

    impl WidgetImpl for CommandaWindow {}
    impl WindowImpl for CommandaWindow {}
    impl ApplicationWindowImpl for CommandaWindow {}
    impl AdwApplicationWindowImpl for CommandaWindow {}
}

glib::wrapper! {
    pub struct CommandaWindow(ObjectSubclass<imp::CommandaWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl CommandaWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    pub fn set_content<P: IsA<gtk::Widget>>(&self, page: &P) {
        let imp = self.imp();
        let content = imp.content.get();
        content.set_child(Some(page));
    }
}
