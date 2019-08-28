use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{
    BufRead,
    BufReader,
    Read,
    Write
};
use std::path::PathBuf;
use std::ptr::null_mut;

use winapi::um::wincon::FreeConsole;
use winapi::um::winuser;
use winapi::um::winuser::MessageBoxW;


pub fn get_file_contents() -> Vec<String> {
    let path = get_path();

    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .expect("File could not be opened.");

    let mut content: Vec<String> = Vec::new();
    let buffer = BufReader::new(&file);

    for line in buffer.lines() {
        match line {
            Ok(line) => content.push(line),
            Err(error) => eprintln!("Line could not be read. {}", error),
        }
    }

    content
}

pub fn parse_file() -> Vec<String>  {
    let path = get_path();
    let lines = get_file_contents();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .expect("File could not be opened.");

    let mut processes: Vec<String> = Vec::new();

    for mut line in lines {
        if !line.is_empty() {
            line = line.trim().to_string();

            if line.starts_with('"') || line.ends_with('"') {
                line.retain(|c| !r#"""#.contains(c));
            }

            if let Err(error) = writeln!(file, "{}", &line) {
                eprintln!("Line could not be added. {}.", error);
            }

            processes.push(line);
        }
    }

    processes
}

pub fn check_or_create_file() {
    let path = get_path();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(path);

    match file {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Content could not be read.");

            if content.trim().is_empty() {
                no_content();
                return;
            }
        },
        Err(e) => eprintln!("File could not be created. {}", e),
    };
}

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn get_path() -> std::path::PathBuf {
    let mut path = PathBuf::new();
    let directory = env::current_exe();

    match directory {
        Ok(directory) => {
            path.push(directory);
            path.set_file_name("command");
            path.set_extension("txt");
        }
        Err(error) => eprint!("Directory not found. {:?}", error)
    };

    path
}

pub fn no_content() {
    let title: Vec<u16> = "undertasker\0".encode_utf16().collect();
    let message: Vec<u16> = "You must specify a command \
                            before running undertasker.\0"
                            .encode_utf16().collect();

    unsafe {
        FreeConsole();

        MessageBoxW(
            null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            winuser::MB_OK | winuser::MB_ICONINFORMATION
        );
    }
}
