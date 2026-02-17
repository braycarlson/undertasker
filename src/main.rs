#![windows_subsystem = "windows"]

slint::include_modules!();

mod command;
mod model;

use rfd::FileDialog;
use slint::{Model, ModelRc, StandardListViewItem, VecModel};
use slint::winit_030::{WinitWindowAccessor, winit};
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use crate::command::Commands;
use crate::model::CustomListViewItem;

pub struct Undertasker {
    app: Rc<App>,
    model: Rc<RefCell<ModelRc<StandardListViewItem>>>,
    path: PathBuf,
    state: Rc<RefCell<ModelRc<CustomListViewItem>>>
}

impl Undertasker {
    pub fn new(
        app: Rc<App>,
        model: Rc<RefCell<ModelRc<StandardListViewItem>>>,
        path: PathBuf,
        state: Rc<RefCell<ModelRc<CustomListViewItem>>>,
    ) -> Self {
        Self { app, model, path, state }
    }

    fn add(&self, path: slint::SharedString) {
        let item = CustomListViewItem::from(path.clone());

        if let Some(model) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
            model.push(item.clone());
        }

        if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
            let item = item.into();
            model.push(item);

            self.app.set_is_not_empty(model.row_count() > 0);

            let index = model.row_count() - 1;
            self.app.set_index(index as i32);

            let height = 30;
            let position = (index * height) as f32;
            self.app.set_scroll(-position);

            self.app.set_path(String::new().into());
        }
    }

    fn browse(&self) {
        if let Some(file) = FileDialog::new().pick_file() {
            if let Some(path) = file.to_str() {
                let item = CustomListViewItem::from(path);

                if let Some(model) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
                    model.push(item.clone());
                }

                if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                    let item = StandardListViewItem::from(path);
                    model.push(item);

                    self.app.set_is_not_empty(model.row_count() > 0);

                    let index = model.row_count() - 1;
                    self.app.set_index(index as i32);

                    let height = 30;
                    let position = (index * height) as f32;
                    self.app.set_scroll(-position);

                    self.app.set_path(String::new().into());
                }
            }
        }
    }

    fn center(&self) {
        let weak = self.app.as_weak();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = weak.upgrade() {
                let monitor_info = app.window().with_winit_window(|winit_window| {
                    winit_window.primary_monitor()
                        .or_else(|| winit_window.current_monitor())
                        .map(|monitor| (monitor.size(), monitor.position()))
                });

                if let Some(Some((screen, monitor_position))) = monitor_info {
                    let size = app.window().size();

                    let x = monitor_position.x
                        + ((screen.width as i32 - size.width as i32) / 2);
                    let y = monitor_position.y
                        + ((screen.height as i32 - size.height as i32) / 2);

                    app.window().set_position(
                        slint::WindowPosition::Physical(
                            slint::PhysicalPosition::new(x, y)
                        )
                    );
                }
            }
        });
    }

    fn load(&self) {
        let commands = Commands::from_file(&self.path).unwrap();

        for command in commands.file() {
            let item = CustomListViewItem::from(command);

            if let Some(model) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
                let item = CustomListViewItem::from(slint::SharedString::from(command));
                model.push(item);
            }

            if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                model.push(item.into());
            }
        }

        for command in commands.windows() {
            let item = CustomListViewItem::from(command);

            if let Some(state) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
                let item = CustomListViewItem::from(slint::SharedString::from(command));
                state.push(item);
            }

            if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                model.push(item.into());
            }
        }

        for (command, quiet) in commands.terminal() {
            let command = slint::SharedString::from(command);

            if let Some(model) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
                let item = CustomListViewItem {
                    item: StandardListViewItem::from(command.clone()),
                    quiet: *quiet,
                };

                model.push(item);
            }

            let item = StandardListViewItem::from(command);

            if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                model.push(item);
            }
        }
    }

    fn remove(&self) {
        let index = self.app.get_index();

        if index >= 0 {
            if let Some(model) = &self.model.borrow().as_any().downcast_ref::<VecModel<StandardListViewItem>>() {
                model.remove(index as usize);
                self.app.set_is_not_empty(model.row_count() > 0);
            }

            if let Some(model) = self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
                model.remove(index as usize);
            }
        }
    }

    fn execute(&self) {
       if let Some(model) = &self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
           let commands = command::Commands::from_state(model);
           commands.execute();
       }
    }

    fn save(&self) {
        if let Some(model) = &self.state.borrow().as_any().downcast_ref::<VecModel<CustomListViewItem>>() {
            let commands = command::Commands::from_state(model);
            let result = commands.to_file(self.path.clone());

            if result.is_ok() {
                self.app.invoke_show_success();
            } else {
                self.app.invoke_show_error();
            }
        }
    }

    fn register(self: Rc<Self>) {
        let app = Rc::clone(&self);

        self.app.on_add(move |path| {
            app.add(path.into());
        });

        let app = Rc::clone(&self);

        self.app.on_browse(move || {
            app.browse();
        });

        let app = Rc::clone(&self);

        self.app.on_remove(move || {
            app.remove();
        });

        let app = Rc::clone(&self);

        self.app.on_run(move || {
            app.execute();
        });

        let app = Rc::clone(&self);

        self.app.on_save(move || {
            app.save();
        });
    }

    fn run(&self) -> Result<(), slint::PlatformError> {
        let is_empty = &self.model
            .borrow()
            .as_any().downcast_ref::<VecModel<StandardListViewItem>>()
            .map_or(true, |x| x.row_count() == 0);

        self.app.set_is_not_empty(!is_empty);

        self.app.show()?;
        self.center();
        slint::run_event_loop()?;
        self.app.hide()?;

        Ok(())
    }
}

fn main() -> Result<(), slint::PlatformError> {
    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(|attributes| {
            attributes.with_position(
                winit::dpi::PhysicalPosition::new(-10000i32, -10000i32)
            )
        })
        .select()?;

    let app = Rc::new(App::new()?);

    let model = VecModel::<StandardListViewItem>::default();
    let state = VecModel::<CustomListViewItem>::default();

    let state_rc = Rc::new(RefCell::new(ModelRc::new(state)));
    let model_rc = Rc::new(RefCell::new(ModelRc::new(model)));

    app.set_list(model_rc.borrow().clone());

    let executable = env::current_exe().expect("Failed to get current executable path");

    let path = executable
        .parent()
        .expect("Failed to get executable's parent directory")
        .join("command.json");

    let undertasker = Rc::new(
        Undertasker::new(
            app.clone(),
            model_rc,
            path,
            state_rc
        )
    );

    undertasker.load();
    undertasker.clone().register();

    undertasker.run()
}
