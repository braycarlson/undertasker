use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser::*;

use crate::font::FONT;
use crate::{BTN_ADD, HINSTANCE};
use crate::util::{to_wstr};

static mut LPFN_EDIT_PROC: WNDPROC = None;


unsafe extern "system" fn edit_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN => {
            if GetKeyState(VK_CONTROL) as u16 & 0x8000 != 0 && wparam == 'A' as usize {
                SendMessageW(hwnd, EM_SETSEL as u32, 0, -1);
            }

            if GetKeyState(VK_RETURN) as u16 & 0x8000 != 0 {
                let parent = GetParent(hwnd);
                let add = GetDlgItem(parent, BTN_ADD);
                SendMessageW(add, BM_CLICK, 0, 0);
            }

            0 as LRESULT
        },

        _ => return CallWindowProcW(LPFN_EDIT_PROC, hwnd, msg, wparam, lparam)
    }
}

pub fn register_edit() {
    let mut edit_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(edit_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { HINSTANCE },
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: to_wstr("EDIT").as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe {
        GetClassInfoExW(HINSTANCE, to_wstr("EDIT").as_ptr(), &mut edit_class);
        LPFN_EDIT_PROC = edit_class.lpfnWndProc;

        edit_class.lpszClassName = to_wstr("EDIT").as_ptr();
        edit_class.style &= !CS_GLOBALCLASS;
        edit_class.hInstance = HINSTANCE;
        edit_class.lpfnWndProc = Some(edit_proc);

        RegisterClassExW(&edit_class)
    };
}

pub fn unregister_edit() {
    unsafe {
        UnregisterClassW(
            to_wstr("EDIT").as_ptr(),
            HINSTANCE
        )
    };
}

pub fn create_edit(parent: HWND, id: i32) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            to_wstr("EDIT").as_ptr(),
            null_mut(),
            WS_VISIBLE | WS_CHILD | WS_BORDER | ES_AUTOHSCROLL | ES_LEFT,
            0, 0, 0, 0,
            parent,
            id as HMENU,
            HINSTANCE,
            null_mut(),
        );

        SendMessageW(hwnd, WM_SETFONT, FONT as WPARAM, 0);

        hwnd
    }
}
