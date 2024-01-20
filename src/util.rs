use winapi::um::wincon::{GetConsoleWindow};
use winapi::um::winuser::{ShowWindow, SW_HIDE};

use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::iter::once;
use std::os::windows::ffi::{OsStrExt};
use std::path::PathBuf;
use std::ptr::null_mut;


pub fn register_custom_classes()
{
    crate::close_button::register_close_button();
    crate::edit::register_edit();
    crate::listbox::register_listbox();
    crate::button::register_button();
}

pub fn unregister_custom_classes()
{
    crate::close_button::unregister_close_button();
    crate::edit::unregister_edit();
    crate::listbox::unregister_listbox();
    crate::button::unregister_button();
}

pub fn to_wstr(value: &str) -> Vec<u16>
{
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(once(0))
        .collect()
}

pub fn from_wstr(value: &[u16]) -> Option<String>
{
    let mut result = String::new();

    for character in value {
        if *character == 0 {
            return Some(result);
        }
        result.push(std::char::from_u32(*character as u32)?);
    }

    Some(result)
}

pub fn to_utf16(string: &str) -> Vec<u16>
{
    let mut vector: Vec<u16> = string.encode_utf16().collect();
    vector.push(0);

    vector
}

pub fn hide_console()
{
    let window = unsafe
    {
        GetConsoleWindow()
    };

    if window != null_mut()
    {
        unsafe
        {
            ShowWindow(window, SW_HIDE);
        }
    }
}

pub fn check_or_create_file()
{
    let path = get_path();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(path);

    match file
    {
        Ok(mut file) =>
        {
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Content could not be read.");

            if content.trim().is_empty()
            {
                return;
            }
        },
        Err(e) => eprintln!("File could not be created. {}", e),
    };
}

pub fn path_exists(path: &str) -> bool
{
    fs::metadata(path).is_ok()
}

pub fn get_path() -> std::path::PathBuf
{
    let mut path = PathBuf::new();
    let directory = env::current_exe();

    match directory
    {
        Ok(directory) =>
        {
            path.push(directory);
            path.set_file_name("command");
            path.set_extension("json");
        }
        Err(error) => eprint!("Directory not found. {:?}", error)
    };

    path
}
