mod utils;

use std::os::windows::process::CommandExt;
use std::process::{
    Command,
    Stdio
};
use std::{
    thread,
    time
};

use tokio::{
    prelude::*,
    runtime::Runtime
};
use tokio_process::CommandExt as extension;
use utils::{
    check_or_create_file,
    parse_file,
    path_exists,
};


fn main() {
    // Create a command.txt file, if it doesn't exist or
    // display a modal dialog box if the file is empty.
    check_or_create_file();

    let mut processes = parse_file();
    let mut settings: Vec<String> = Vec::new();

    for process in &mut processes {
        println!("{:?}", process);

        // "Windows Settings" can have one instance
        // at a time, so execute these commands last.
        if process.starts_with("start ms-settings") {
            let string = process.to_string();
            settings.push(string);
        }

        if path_exists(&process) {
            Command::new(process)
                .creation_flags(0x00000008) // DETACHED_PROCESS
                .spawn()
                .expect("Process could not be spawned.");
        } else {
            if process.starts_with("start") {
                Command::new("cmd")
                    .arg("/C")
                    .arg(&process)
                    .creation_flags(0x00000008) // DETACHED_PROCESS
                    .output()
                    .expect("Process could not be spawned.");
            } else {
                Command::new("cmd")
                    .arg("/K")
                    .arg(&process)
                    .creation_flags(0x00000010) // CREATE_NEW_CONSOLE
                    .spawn()
                    .expect("Process could not be spawned.");
            }
        }
    }

    for (iteration, process) in settings.iter().enumerate() {
        // If there is only one item in the vector or the loop
        // is on the final iteration, just detach the process.
        if settings.len() == 1 || iteration == settings.len() - 1 {
            Command::new("cmd")
                .arg("/C")
                .arg(&process)
                .creation_flags(0x00000008) // DETACHED_PROCESS
                .spawn()
                .expect("Process could not be spawned.");
        } else {
            if let Ok(mut child) = Command::new("cmd")
                .arg("/C")
                .arg(&process)
                .creation_flags(0x00000008) // DETACHED_PROCESS
                .spawn() {

                let _ = child.wait();

                // Poll the "Windows Settings" process, and wait for the
                // user to exit before opening another instance.
                loop {
                    // If the process ID is needed:
                    // wmic process where name='SystemSettings.exe' get processid | findstr /v ProcessId
                    let mut pid = Command::new("cmd")
                        .arg("/C")
                        .arg("wmic process where name='SystemSettings.exe'")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn_async()
                        .expect("Process could not be spawned.");

                    let stderr = pid
                        .stderr()
                        .take()
                        .expect("No standard error.");

                    let codec = tokio::codec::LinesCodec::new();

                    let line = tokio::codec::FramedRead::new(stderr, codec)
                        .take(1)
                        .collect()
                        .map(|l| l.first().cloned()
                    );

                    let mut runtime = Runtime::new().expect("No runtime was created.");

                    let result = runtime.block_on(line);

                    if let Ok(error) = result {
                        if let Some(line) = error {
                            // Windows Management Interface Command (wmic) writes to
                            // stderr if there is no instance of the process found.
                            if !line.is_empty() {
                                break;
                            }
                        }
                    }
                    // Sleeping drastically reduces CPU usage.
                    let time = time::Duration::from_millis(500);
                    thread::sleep(time);
                }

               let _ = child.kill();
            }
        }
    }
}
