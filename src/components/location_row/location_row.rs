/* selector_row.rs
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

use crate::services::weather::LocationInfo;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/idevecore/Commanda/components/location_row/location_row.ui")]
    pub struct LocationRow {
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        pub location: RefCell<Option<LocationInfo>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocationRow {
        const NAME: &'static str = "LocationRow";
        type Type = super::LocationRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LocationRow {}
    impl WidgetImpl for LocationRow {}
    impl ListBoxRowImpl for LocationRow {}
}

glib::wrapper! {
    pub struct LocationRow(ObjectSubclass<imp::LocationRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl LocationRow {
    pub fn new(loc: &LocationInfo) -> Self {
        let obj: Self = glib::Object::builder().build();

        let name = format!("{}, {}, {}", loc.name, loc.admin1, loc.country);

        obj.imp().name.get().set_label(&name);

        *obj.imp().location.borrow_mut() = Some(loc.clone());
        obj
    }
    pub fn location(&self) -> Option<LocationInfo> {
        self.imp().location.borrow().clone()
    }
}
