use serde::{Deserialize, Serialize};
use slint::{Model, VecModel};
use std::io;
use std::io::{BufRead, BufReader};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{thread, time};
use winapi::um::winbase::{CREATE_NEW_CONSOLE, CREATE_NO_WINDOW, DETACHED_PROCESS};

use crate::model::CustomListViewItem;


#[derive(Serialize, Deserialize)]
pub struct Commands {
    file: Vec<String>,
    windows: Vec<String>,
    terminal: Vec<(String, bool)>,
}

impl Commands {
    pub fn file(&self) -> &[String] {
        &self.file
    }

    pub fn windows(&self) -> &[String] {
        &self.windows
    }

    pub fn terminal(&self) -> &[(String, bool)] {
        &self.terminal
    }

    pub fn execute(&self) {
        for command in &self.file {
            Command::new(command)
                .creation_flags(DETACHED_PROCESS)
                .spawn()
                .expect("Process could not be spawned.");
        }

        for (command, quiet) in &self.terminal {
            if *quiet {
                Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
                    .expect("Terminal process could not be spawned.");
            } else {
                Command::new("cmd")
                    .arg("/K")
                    .arg(command)
                    .creation_flags(CREATE_NEW_CONSOLE)
                    .spawn()
                    .expect("Terminal process could not be spawned.");
            }
        }

        for (iteration, command) in self.windows.iter().enumerate() {
            if self.windows.len() == 1 || iteration == self.windows.len() - 1 {
                Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .creation_flags(DETACHED_PROCESS)
                    .spawn()
                    .expect("Process could not be spawned.");
            } else {
                if let Ok(mut child) = Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .creation_flags(DETACHED_PROCESS)
                    .spawn() {

                    let _ = child.wait();

                    loop {
                        let pid = Command::new("cmd")
                            .arg("/C")
                            .arg("wmic process where name='SystemSettings.exe'")
                            .creation_flags(CREATE_NO_WINDOW)
                            .output()
                            .expect("Process could not be spawned.");

                        let mut instance = true;
                        let reader = BufReader::new(&*pid.stderr).lines();

                        for line in reader {
                            if let Ok(line) = line {
                               if !line.is_empty() {
                                   instance = false;
                               }
                            }
                        }

                        if instance == false {
                            break;
                        }

                        let time = time::Duration::from_millis(500);
                        thread::sleep(time);
                    }

                    let _ = child.kill();
                }
            }
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if !path.as_ref().exists() {
            let commands = Commands {
                file: Vec::new(),
                windows: Vec::new(),
                terminal: Vec::new(),
            };
            commands.to_file(path.as_ref().to_path_buf())?;
            Ok(commands)
        } else {
            let json = fs::read_to_string(path)
                .map_err(|e| io::Error::new(e.kind(), format!("Failed to read file: {}", e)))?;

            let mut commands: Commands = serde_json::from_str(&json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON: {}", e)))?;

            commands.file.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
            commands.windows.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
            commands.terminal.sort_by(|(x, _), (y, _)| x.to_lowercase().cmp(&y.to_lowercase()));

            Ok(commands)
        }
    }

    pub fn to_file(&self, path: PathBuf) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to serialize JSON: {}", e)))?;

        fs::write(path, json.as_bytes())
            .map_err(|e| io::Error::new(e.kind(), format!("Failed to write file: {}", e)))
    }

    pub fn from_state(model: &VecModel<CustomListViewItem>) -> Self {
        let mut file = Vec::new();
        let mut terminal = Vec::new();
        let mut windows = Vec::new();

        for index in 0..model.row_count() {
            if let Some(item) = model.row_data(index) {
                let command = item.item.text;
                let quiet = item.quiet;

                if fs::metadata(&*command).is_ok() {
                    file.push(command.to_string());
                } else if command.starts_with("start") {
                    windows.push(command.to_string());
                } else {
                    let pair = (command.to_string(), quiet);
                    terminal.push(pair);
                }
            }
        }

        file.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
        windows.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
        terminal.sort_by(|(x, _), (y, _)| x.to_lowercase().cmp(&y.to_lowercase()));

        Commands { file, windows, terminal }
    }
}
