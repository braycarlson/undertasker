use std::alloc::{self, Layout};
use std::ptr::null_mut;
use winapi::shared::minwindef::{FALSE, LPARAM, LRESULT, TRUE, UINT, WPARAM};
use winapi::shared::windef::{HDC, HGDIOBJ, HMENU, HWND, POINT};
use winapi::um::wingdi::{LineTo, MoveToEx, SelectObject};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::winuser::*;

use crate::brush::*;
use crate::HINSTANCE;
use crate::util::to_wstr;


enum ButtonState {
    Idle,
    Hover,
    Active,
}

unsafe fn paint(hwnd: HWND, hdc: HDC, state: &ButtonState) {
    let mut rect = std::mem::MaybeUninit::uninit().assume_init();
    GetClientRect(hwnd, &mut rect);

    FillRect(hdc, &rect, BRUSH_BLACK_0);

    let old_pen = SelectObject(hdc, match state {
        ButtonState::Idle => PEN_PURPLE_0 as HGDIOBJ,
        ButtonState::Hover => PEN_PURPLE_1 as HGDIOBJ,
        ButtonState::Active => PEN_PURPLE_1 as HGDIOBJ,
    });

    MoveToEx(hdc, rect.left + 8, rect.top + 8, null_mut());
    LineTo(hdc, rect.right - 8, rect.bottom - 8);
    MoveToEx(hdc, rect.right - 8, rect.top + 8, null_mut());
    LineTo(hdc, rect.left + 8, rect.bottom - 8);
    SelectObject(hdc, old_pen);
}

unsafe extern "system" fn close_button_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, 0) as *mut u8;
    let button_state = ptr as *mut ButtonState;

    match msg {
        WM_NCCREATE => {
            let layout = Layout::new::<ButtonState>();
            let new_ptr = alloc::alloc(layout);

            if new_ptr as isize == 0 {
                return FALSE as isize;
            }

            *(new_ptr as *mut ButtonState) = ButtonState::Idle;

            if SetWindowLongPtrW(hwnd, 0, new_ptr as isize) != 0 {
                return FALSE as isize;
            }

            return TRUE as isize;
        },

        WM_NCDESTROY => {
            if ptr as isize != 0 {
                let layout = Layout::new::<ButtonState>();
                alloc::dealloc(ptr, layout);
            }

            0 as LRESULT
        },

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = PAINTSTRUCT {
                hdc: null_mut(),
                fErase: FALSE,
                rcPaint: std::mem::MaybeUninit::uninit().assume_init(),
                fRestore: FALSE,
                fIncUpdate: FALSE,
                rgbReserved: [0u8; 32],
            };

            let hdc = BeginPaint(hwnd, &mut ps);
            paint(hwnd, hdc, &*button_state);
            EndPaint(hwnd, &mut ps);

            0 as LRESULT
        }

        WM_MOUSEMOVE => {
            let position = POINT {
                x: GET_X_LPARAM(lparam),
                y: GET_Y_LPARAM(lparam)
            };

            let mut rect = std::mem::MaybeUninit::uninit().assume_init();
            GetClientRect(hwnd, &mut rect);

            let mut track = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: HOVER_DEFAULT,
            };

            TrackMouseEvent(&mut track);

            *button_state = if PtInRect(&rect, position) != 0 {
                match *button_state {
                    ButtonState::Idle => ButtonState::Hover,
                    ButtonState::Hover => ButtonState::Hover,
                    ButtonState::Active => ButtonState::Active,
                }
            }
            else {
                ButtonState::Idle
            };

            InvalidateRect(hwnd, &rect, FALSE);
            0 as LRESULT
        },

        WM_MOUSELEAVE => {
            *button_state = ButtonState::Idle;
            InvalidateRect(hwnd, null_mut(), FALSE);
            0 as LRESULT
        },

        WM_LBUTTONDOWN => {
            *button_state = ButtonState::Active;
            InvalidateRect(hwnd, null_mut(), FALSE);
            0 as LRESULT
        },

        WM_LBUTTONUP => {
            let position = POINT {
                x: GET_X_LPARAM(lparam),
                y: GET_Y_LPARAM(lparam)
            };

            let mut rect = std::mem::MaybeUninit::uninit().assume_init();
            GetClientRect(hwnd, &mut rect);

            if let ButtonState::Active = *button_state {
                let parent = GetParent(hwnd);
                PostMessageW(parent, WM_CLOSE, 0, 0);
            }

            *button_state = if PtInRect(&rect, position) != 0 { ButtonState::Hover } else { ButtonState::Idle };

            InvalidateRect(hwnd, null_mut(), FALSE);
            0 as LRESULT
        },

        _ => return DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub fn register_close_button() {
    let close_button_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(close_button_proc),
        cbClsExtra: 0,
        cbWndExtra: std::mem::size_of::<*const i32>() as i32,
        hInstance: unsafe { HINSTANCE },
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: to_wstr("close_button").as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe {
        RegisterClassExW(&close_button_class)
    };
}

pub fn unregister_close_button() {
    unsafe {
        UnregisterClassW(
            to_wstr("close_button").as_ptr(),
            HINSTANCE
        )
    };
}

pub fn create_close_button(parent: HWND, id: i32) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            to_wstr("close_button").as_ptr(),
            null_mut(),
            WS_CHILD | WS_VISIBLE,
            0, 0, 0, 0,
            parent,
            id as HMENU,
            HINSTANCE,
            null_mut(),
        );

        hwnd
    }
}
