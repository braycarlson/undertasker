use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::{thread, time};
use winapi::um::winbase::{CREATE_NEW_CONSOLE, CREATE_NO_WINDOW, DETACHED_PROCESS};

use crate::window::Command;


pub fn execute(commands: Command)
{
    for command in commands.file
    {
        std::process::Command::new(command)
            .creation_flags(DETACHED_PROCESS)
            .spawn()
            .expect("Process could not be spawned.");
    }

    for command in commands.terminal
    {
        std::process::Command::new("cmd")
            .arg("/K")
            .arg(&command)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .expect("Process could not be spawned.");
    }

    for (iteration, command) in commands.windows.iter().enumerate()
    {
        if commands.windows.len() == 1 || iteration == commands.windows.len() - 1
        {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(&command)
                .creation_flags(DETACHED_PROCESS)
                .spawn()
                .expect("Process could not be spawned.");
        }
        else
        {
            if let Ok(mut child) = std::process::Command::new("cmd")
                .arg("/C")
                .arg(&command)
                .creation_flags(DETACHED_PROCESS)
                .spawn() {

                let _ = child.wait();

                loop
                {
                    let pid = std::process::Command::new("cmd")
                        .arg("/C")
                        .arg("wmic process where name='SystemSettings.exe'")
                        .creation_flags(CREATE_NO_WINDOW)
                        .output()
                        .expect("Process could not be spawned.");

                    let mut instance = true;
                    let reader = BufReader::new(&*pid.stderr).lines();

                    for line in reader
                    {
                        if let Ok(line) = line
                        {
                           if !line.is_empty()
                           {
                               instance = false;
                           }
                        }
                    }

                    if instance == false
                    {
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
