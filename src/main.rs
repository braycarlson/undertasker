#![allow(non_snake_case)]

mod brush;
mod button;
mod core;
mod close_button;
mod edit;
mod font;
mod listbox;
mod util;
mod window;

use std::ptr::null_mut;
use winapi::shared::minwindef::HINSTANCE;

use crate::util::{check_or_create_file, hide_console};
use crate::window::{create_window, message_loop, register_window};

const BTN_ADD: i32 = 1001;
const BTN_CLOSE: i32 = 1002;
const BTN_REMOVE: i32 = 1003;
const BTN_BROWSE: i32 = 1004;
const BTN_RUN: i32 = 1005;
const BTN_SAVE: i32 = 1006;
const EDIT_COMMAND: i32 = 1007;
const LB_COMMAND: i32 = 1008;
static mut HINSTANCE: HINSTANCE = null_mut();


fn main() {
    hide_console();
    check_or_create_file();

    register_window();
    create_window();
    message_loop();
}
