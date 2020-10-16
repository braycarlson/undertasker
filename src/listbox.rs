use crate::window::delete_item;
use std::ptr::null_mut;
use winapi::ctypes::c_int;
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser::*;

use crate::font::FONT;
use crate::HINSTANCE;
use crate::util::to_wstr;

static mut LPFN_LISTBOX_PROC: WNDPROC = None;


unsafe extern "system" fn listbox_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN => {
            if GetKeyState(VK_DELETE) as u16 & 0x8000 != 0 {
                delete_item();
            }
        },

        _ => {}
    }

    return CallWindowProcW(LPFN_LISTBOX_PROC, hwnd, msg, wparam, lparam);
}

pub fn register_listbox() {
    let mut listbox_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(listbox_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { HINSTANCE },
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: to_wstr("LISTBOX").as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe {
        GetClassInfoExW(HINSTANCE, to_wstr("LISTBOX").as_ptr(), &mut listbox_class);
        LPFN_LISTBOX_PROC = listbox_class.lpfnWndProc;

        listbox_class.lpszClassName = to_wstr("LISTBOX").as_ptr();
        listbox_class.style &= !CS_GLOBALCLASS;
        listbox_class.hInstance = HINSTANCE;
        listbox_class.lpfnWndProc = Some(listbox_proc);

        RegisterClassExW(&listbox_class)
    };
}

pub fn unregister_listbox() {
    unsafe {
        UnregisterClassW(
            to_wstr("LISTBOX").as_ptr(),
            HINSTANCE
        )
    };
}

pub fn create_listbox(parent: HWND, id: i32) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            to_wstr("LISTBOX").as_ptr(),
            null_mut(),
            WS_VISIBLE | WS_CHILD | WS_BORDER | LBS_SORT | LBS_NOTIFY | WS_VSCROLL | LBS_NOINTEGRALHEIGHT | LBS_OWNERDRAWFIXED | LBS_HASSTRINGS,
            0, 0, 0, 0,
            parent,
            id as HMENU,
            HINSTANCE,
            null_mut(),
        );

        SendMessageW(hwnd, WM_SETFONT, FONT as WPARAM, 0) as DWORD as c_int;
        SendMessageW(hwnd, LB_SETITEMHEIGHT, 0, 25) as DWORD as c_int;

        hwnd
    }
}
