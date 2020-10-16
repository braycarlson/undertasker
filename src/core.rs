use std::os::windows::process::CommandExt;
use std::process::Stdio;
use std::{thread, time};
use tokio::io::{BufReader, AsyncBufReadExt};
use winapi::um::winbase::{CREATE_NEW_CONSOLE, CREATE_NO_WINDOW, DETACHED_PROCESS};

use crate::window::Command;


#[tokio::main]
pub async fn execute(commands: Command) {
    for command in commands.file {
        std::process::Command::new(command)
            .creation_flags(DETACHED_PROCESS)
            .spawn()
            .expect("Process could not be spawned.");
    }

    for command in commands.terminal {
        std::process::Command::new("cmd")
            .arg("/K")
            .arg(&command)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .expect("Process could not be spawned.");
    }

    for (iteration, command) in commands.windows.iter().enumerate() {
        if commands.windows.len() == 1 || iteration == commands.windows.len() - 1 {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(&command)
                .creation_flags(DETACHED_PROCESS)
                .spawn()
                .expect("Process could not be spawned.");
        } else {
            if let Ok(mut child) = std::process::Command::new("cmd")
                .arg("/C")
                .arg(&command)
                .creation_flags(DETACHED_PROCESS)
                .spawn() {

                let _ = child.wait();

                loop {
                    let mut pid = tokio::process::Command::new("cmd")
                        .arg("/C")
                        .arg("wmic process where name='SystemSettings.exe'")
                        .creation_flags(CREATE_NO_WINDOW)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Process could not be spawned.");

                    let stderr = pid.stderr.take().expect("No stderr.");
                    let mut reader = BufReader::new(stderr).lines();

                    tokio::spawn(
                        async {
                            pid.await.expect("Process encountered an error");
                        }
                    );

                    let line = reader
                        .next_line()
                        .await
                        .expect("The reader could not fetch the next line.");

                    if let Some(line) = line {
                       if !line.is_empty() {
                           break;
                       }
                    }

                    let time = time::Duration::from_millis(500);
                    thread::sleep(time);
                }

                let _ = child.kill();
            }
        }
    }
}
