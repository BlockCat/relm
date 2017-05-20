/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    CellLayoutExt,
    CellRendererText,
    Inhibit,
    ListStore,
    ToValue,
    TreeSelection,
    TreeViewColumn,
    Type,
    WidgetExt,
};
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    SelectionChanged(TreeSelection),
    Quit,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let columns = vec![Type::String];
        let model = ListStore::new(&columns);
        let row = model.append();
        model.set_value(&row, 0, &"String".to_value());
        let row = model.append();
        model.set_value(&row, 0, &"Text".to_value());

        let view_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", 0);
        self.tree_view.append_column(&view_column);

        self.tree_view.set_model(Some(&model));
    }

    fn model() -> () {
    }

    fn update(&mut self, event: Msg) {
        match event {
            SelectionChanged(_selection) => println!("selection changed"),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                #[name="tree_view"]
                gtk::TreeView {
                    selection.changed(selection) => SelectionChanged(selection.clone()),
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}