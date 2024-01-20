use std::alloc::{self, Layout};
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{FALSE, LPARAM, LRESULT, TRUE, UINT, WPARAM};
use winapi::shared::windef::{HDC, HGDIOBJ, HMENU, HWND, POINT, RECT};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::wingdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, RGB, SelectObject, SetBkMode, SetTextColor, SRCCOPY, TRANSPARENT};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::*;

use crate::brush::*;
use crate::font::FONT;
use crate::HINSTANCE;
use crate::util::to_wstr;

enum ButtonState
{
    Idle,
    Hover,
    Active,
}

struct ButtonData
{
    state: ButtonState,
    colour: Purple,
}

static BUTTON_CLASS_NAME: &str = "_button_";
static mut LPFN_BUTTON_PROC: WNDPROC = None;
static mut CB_WND_EXTRA: i32 = 0;


fn paint(hwnd: HWND, hdc: HDC, rect: &mut RECT, data: &ButtonData)
{
    unsafe
    {
        let hdc_mem = CreateCompatibleDC(hdc);
        let hbm_mem = CreateCompatibleBitmap(hdc, rect.right - rect.left, rect.bottom - rect.top);

        let hbm_old = SelectObject(hdc_mem, hbm_mem as HGDIOBJ);

        FillRect(hdc_mem, rect, match data.state
        {
            ButtonState::Idle => BRUSH_PURPLE_0,
            ButtonState::Hover => BRUSH_PURPLE_1,
            ButtonState::Active => BRUSH_PURPLE_1
        });

        SetBkMode(hdc_mem, TRANSPARENT as i32);
        SetTextColor(hdc_mem, RGB(255, 255, 255));

        let length = GetWindowTextLengthW(hwnd) + 1;
        let mut text: Vec<u16> = vec![0u16; length as usize];
        GetWindowTextW(hwnd, text.as_mut_ptr(), length);

        let old_font = SelectObject(hdc_mem, FONT as HGDIOBJ);
        DrawTextW(hdc_mem, text.as_ptr(), -1, rect, DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOCLIP);

        BitBlt(
            hdc,
            rect.left, rect.top,
            rect.right - rect.left, rect.bottom - rect.top,
            hdc_mem,
            0, 0,
            SRCCOPY
        );

        SelectObject(hdc_mem, old_font);
        SelectObject(hdc_mem, hbm_old);
        DeleteObject(hbm_mem as *mut c_void);
        DeleteDC(hdc_mem);
    }
}

unsafe extern "system" fn button_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT
{
    let ptr = GetWindowLongPtrW(hwnd, CB_WND_EXTRA) as *mut u8;
    let button_data = ptr as *mut ButtonData;

    match msg
    {
        WM_NCCREATE =>
        {
            if CallWindowProcW(LPFN_BUTTON_PROC, hwnd, msg, wparam, lparam) == 0
            {
                return FALSE as isize;
            }

            let layout = Layout::new::<ButtonData>();
            let new_ptr = alloc::alloc(layout);

            if new_ptr as isize == 0
            {
                return FALSE as isize;
            }

            *(new_ptr as *mut ButtonData) = ButtonData
            {
                state: ButtonState::Idle,
                colour: Purple::P0
            };

            if SetWindowLongPtrW(hwnd, CB_WND_EXTRA, new_ptr as isize) != 0
            {
                return FALSE as isize;
            }

            return TRUE as isize;
        },

        WM_NCDESTROY =>
        {
            if ptr as isize != 0
            {
                let layout = Layout::new::<ButtonData>();
                alloc::dealloc(ptr, layout)
            }

            return 0;
        },

        WM_ERASEBKGND =>
        {
            return 1;
        },

        WM_PAINT =>
        {
            InvalidateRect(hwnd, null_mut(), TRUE);

            let mut ps: PAINTSTRUCT = PAINTSTRUCT
            {
                hdc: null_mut(),
                fErase: FALSE,
                rcPaint: std::mem::MaybeUninit::uninit().assume_init(),
                fRestore: FALSE,
                fIncUpdate: FALSE,
                rgbReserved: [0u8; 32],
            };

            BeginPaint(hwnd, &mut ps);
            paint(hwnd, ps.hdc, &mut ps.rcPaint, &*button_data);
            EndPaint(hwnd, &ps);

            return 0;
        },

        WM_PRINTCLIENT =>
        {
            let mut rect: RECT = std::mem::MaybeUninit::uninit().assume_init();
            GetClientRect(hwnd, &mut rect);
            paint(hwnd, wparam as HDC, &mut rect, &*button_data);

            return 0;
        },

        WM_MOUSEMOVE =>
        {
            let position = POINT
            {
                x: GET_X_LPARAM(lparam),
                y: GET_Y_LPARAM(lparam)
            };

            let mut track = TRACKMOUSEEVENT
            {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: HOVER_DEFAULT,
            };

            TrackMouseEvent(&mut track);

            let mut rect = std::mem::MaybeUninit::uninit().assume_init();
            GetClientRect(hwnd, &mut rect);

            (*button_data).state = if PtInRect(&rect, position) != 0
            {
                match (*button_data).state
                {
                    ButtonState::Idle => ButtonState::Hover,
                    ButtonState::Hover => ButtonState::Hover,
                    ButtonState::Active => ButtonState::Active,
                }
            }
            else
            {
                ButtonState::Idle
            };

            InvalidateRect(hwnd, &rect, FALSE);
        },

        WM_KILLFOCUS =>
        {
            (*button_data).state = ButtonState::Idle;
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        WM_SETFOCUS =>
        {
            (*button_data).state = ButtonState::Idle;
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        WM_MOUSELEAVE =>
        {
            (*button_data).state = ButtonState::Idle;
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        WM_LBUTTONDBLCLK =>
        {
            (*button_data).state = ButtonState::Active;
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        WM_LBUTTONDOWN =>
        {
            (*button_data).state = ButtonState::Active;
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        WM_LBUTTONUP =>
        {
            let position = POINT
            {
                x: GET_X_LPARAM(lparam),
                y: GET_Y_LPARAM(lparam)
            };

            let mut rect = std::mem::MaybeUninit::uninit().assume_init();
            GetClientRect(hwnd, &mut rect);

            (*button_data).state = if PtInRect(&rect, position) != 0 { ButtonState::Hover } else { ButtonState::Idle };
            InvalidateRect(hwnd, null_mut(), FALSE);
        },

        _ => {}
    }

    return CallWindowProcW(LPFN_BUTTON_PROC, hwnd, msg, wparam, lparam);
}

pub fn register_button()
{
    let mut button_class = WNDCLASSEXW
    {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(button_proc),
        cbClsExtra: 0,
        cbWndExtra: std::mem::size_of::<*mut ButtonData>() as i32,
        hInstance: unsafe { HINSTANCE },
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: null_mut(),
        hIconSm: null_mut(),
    };

    unsafe
    {
        GetClassInfoExW(HINSTANCE, to_wstr("BUTTON").as_ptr(), &mut button_class);
        CB_WND_EXTRA = button_class.cbWndExtra;
        LPFN_BUTTON_PROC = button_class.lpfnWndProc;

        button_class.lpszClassName = to_wstr(BUTTON_CLASS_NAME).as_ptr();
        button_class.style &= !CS_GLOBALCLASS;
        button_class.hInstance = HINSTANCE;
        button_class.cbWndExtra += std::mem::size_of::<*mut ButtonData>() as i32;
        button_class.lpfnWndProc = Some(button_proc);

        RegisterClassExW(&button_class)
    };
}

pub fn unregister_button()
{
    unsafe
    {
        UnregisterClassW(
            to_wstr(BUTTON_CLASS_NAME).as_ptr(),
            HINSTANCE
        )
    };
}

pub fn create_button(parent: HWND, id: i32, text: &str) -> HWND
{
    unsafe
    {
        let hwnd = CreateWindowExW(
            0,
            to_wstr(BUTTON_CLASS_NAME).as_ptr(),
            to_wstr(text).as_ptr(),
            WS_CHILD | WS_VISIBLE,
            0, 0, 0, 0,
            parent,
            id as HMENU,
            HINSTANCE,
            null_mut(),
        );

        let ptr = GetWindowLongPtrW(hwnd, CB_WND_EXTRA) as *mut u8;
        let button_data = ptr as *mut ButtonData;
        (*button_data).colour = Purple::P0;

        if SetWindowLongPtrW(hwnd, CB_WND_EXTRA, ptr as isize) == 0
        {
            eprintln!("Failed to set colour of button: {}", GetLastError());
        }

        hwnd
    }
}
