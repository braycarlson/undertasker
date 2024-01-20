#![windows_subsystem = "windows"]

slint::include_modules!();

mod command;

use i_slint_backend_winit::WinitWindowAccessor;
use rfd::FileDialog;
use slint::{Model, ModelRc, StandardListViewItem, VecModel, Window};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use winit::dpi::PhysicalPosition;

use crate::command::Commands;


fn center(window_option: Option<&Window>) {
    if let Some(window) = window_option {
        window.with_winit_window(|winit_window| {
            let monitor_size = winit_window.current_monitor().unwrap().size();

            let x = (monitor_size.width - 600) / 2;
            let y = (monitor_size.height - 350) / 2;

            winit_window.set_outer_position(PhysicalPosition::new(x, y));
        });
    }
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-skia");

    let app = App::new()?;

    let handle = app.as_weak();
    let browse_handle = handle.clone();
    let save_handle = handle.clone();

    let window = app.window();
    center(Some(window));

    let model = VecModel::<StandardListViewItem>::default();
    let command = Rc::new(RefCell::new(ModelRc::new(model)));
    app.set_list(command.borrow().clone());

    let executable = env::current_exe().expect("Failed to get current executable path");

    let path = executable
        .parent()
        .expect("Failed to get executable's parent directory")
        .join("command.json");

    let commands = Commands::from_file(path.clone()).unwrap();

    for path in commands.file().iter().chain(commands.windows().iter()).chain(commands.terminal().iter()) {
        if let Some(model) = command.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            let item = StandardListViewItem::from(slint::SharedString::from(path.as_str()));
            model.push(item);
        }
    }

    let is_empty = command
        .borrow()
        .as_any().downcast_ref::<VecModel<StandardListViewItem>>()
        .map_or(true, |model| model.row_count() == 0);

    app.set_is_not_empty(!is_empty);

    let add = command.clone();
    let clone = handle.clone();

    app.on_add(move |path| {
        if let Some(model) = add.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            let item = StandardListViewItem::from(StandardListViewItem::from(path));
            model.push(item);

            if let Some(app) = clone.upgrade() {
                app.set_is_not_empty(model.row_count() > 0);

                let index = model.row_count() - 1;
                app.set_index(index as i32);

                let height = 30;
                let position = (index * height) as f32;
                app.set_scroll(-position);

                app.set_path(String::new().into());
            }
        }

    });

    let browse = command.clone();

    app.on_browse(move || {
        if let Some(file_path) = FileDialog::new().pick_file() {
            if let Some(model) = browse.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                if let Some(path_str) = file_path.to_str() {
                    let item = StandardListViewItem::from(slint::SharedString::from(path_str));
                    model.push(item);

                    if let Some(app) = browse_handle.upgrade() {
                        app.set_is_not_empty(model.row_count() > 0);

                        let index = model.row_count() - 1;
                        app.set_index(index as i32);

                        let height = 30;
                        let position = (index * height) as f32;
                        app.set_scroll(-position);

                        app.set_path(String::new().into());
                    }
                }
            }
        }
    });

    let remove = command.clone();

    app.on_remove(move || {
        if let Some(model) = remove.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            let app = handle.unwrap();
            let index = app.get_index();

            if index >= 0 {
                model.remove(index as usize);
            }

            app.set_is_not_empty(model.row_count() > 0);
        }
    });

    let save = command.clone();

    app.on_save(move || {
        let model_rc = save.borrow();

        if let Some(model) = model_rc.as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            if let Some(app) = save_handle.upgrade() {
                let commands = command::Commands::from_ui(model);

                let result = commands.to_disk(path.clone());

                if result.is_ok() {
                    app.invoke_show_success();
                } else {
                    app.invoke_show_error();
                }
            }
        }
    });

    let run = command.clone();

    app.on_run(move || {
        let model_rc = run.borrow();

        if let Some(model) = model_rc.as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            let commands = command::Commands::from_ui(model);
            commands.execute();
        }
    });

    app.run()
}
