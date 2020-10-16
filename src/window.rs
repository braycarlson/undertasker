use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::fs::OpenOptions;
use std::ptr::{null, null_mut};
use std::thread;
use winapi::shared::minwindef::{FALSE, LPARAM, LRESULT, MAX_PATH, UINT, WPARAM};
use winapi::shared::ntdef::LPWSTR;
use winapi::shared::windef::{HDC, HWND};
use winapi::um::commdlg::{GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY, OFN_PATHMUSTEXIST, OPENFILENAMEW};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{RGB, SetBkMode, SetBkColor, SetTextColor, TRANSPARENT};
use winapi::um::winuser::*;

use crate::{BTN_ADD, BTN_CLOSE, BTN_BROWSE, BTN_REMOVE, BTN_RUN, BTN_SAVE, EDIT_COMMAND, HINSTANCE, LB_COMMAND};
use crate::brush::*;
use crate::button::create_button;
use crate::close_button::create_close_button;
use crate::core::execute;
use crate::edit::create_edit;
use crate::listbox::create_listbox;
use crate::font::{load_fonts, unload_fonts};
use crate::util::{from_wstr, get_path, register_custom_classes, path_exists, to_wstr, to_utf16, unregister_custom_classes};


#[derive(PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Command {
    pub file: Vec<String>,
    pub windows: Vec<String>,
    pub terminal: Vec<String>
}

unsafe fn run() {
    let commands = get_commands();

    if commands.file.is_empty() && commands.windows.is_empty() && commands.terminal.is_empty() {
        MessageBoxW(
             null_mut(),
             to_wstr("Please add a command.").as_ptr(),
             to_wstr("undertasker").as_ptr(),
             MB_ICONINFORMATION | MB_OK,
         );

        return
    }

    execute(commands);
}

unsafe fn get_file() -> Option<Command> {
    let path = get_path();

    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .expect("File could not be opened.");

    let reader = BufReader::new(file);
    let commands = serde_json::from_reader(reader).unwrap_or(None);

    commands
}

unsafe fn get_commands() -> Command {
    let buffer: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];
    let mut file: Vec<String> = Vec::new();
    let mut terminal: Vec<String> = Vec::new();
    let mut windows: Vec<String> = Vec::new();

    let hwnd = FindWindowW(
        to_wstr("WINDOW").as_ptr(),
        to_wstr("undertasker").as_ptr()
    );

    let listbox = GetDlgItem(hwnd, LB_COMMAND);
    let count = SendMessageW(listbox, LB_GETCOUNT, 0, 0);

    for index in 0..count {
        SendMessageW(listbox, LB_GETTEXT, index as WPARAM, buffer.as_ptr() as LPARAM);

        let command = from_wstr(&buffer);

        if let Some(command) = command {
            if path_exists(&command) {
                file.push(command);
            } else if command.starts_with("start") {
                windows.push(command);
            } else {
                terminal.push(command);
            }
        }
    }

    file.sort_by(
        |x, y| x.to_lowercase().cmp(
            &y.to_lowercase()
        )
    );

    windows.sort_by(
        |x, y| x.to_lowercase().cmp(
            &y.to_lowercase()
        )
    );

    terminal.sort_by(
        |x, y| x.to_lowercase().cmp(
            &y.to_lowercase()
        )
    );

    let commands = Command {
        file: file,
        windows: windows,
        terminal: terminal
    };

    commands
}

unsafe fn save_file() {
    let commands = get_commands();
    let path = get_path();

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .expect("File could not be opened.");

    serde_json::to_writer_pretty(file, &commands).expect("File could not be saved.");
}

fn add_item(item: Vec::<u16>) {
    unsafe {
        let value = item.as_ptr();

        let hwnd = FindWindowW(
            to_wstr("WINDOW").as_ptr(),
            to_wstr("undertasker").as_ptr()
        );

        let listbox = GetDlgItem(hwnd, LB_COMMAND);

        let index = SendMessageW(listbox, LB_ADDSTRING, 0, value as LPARAM);
        SetFocus(listbox);
        SendMessageW(listbox, LB_SETCURSEL, index as WPARAM, 0);
    }
}

pub fn delete_item() {
    unsafe {
        let hwnd = FindWindowW(
            to_wstr("WINDOW").as_ptr(),
            to_wstr("undertasker").as_ptr()
        );

        let listbox = GetDlgItem(hwnd, LB_COMMAND);
        let count = SendMessageW(listbox, LB_GETCOUNT, 0, 0);

        if count == 0 {
            return
        }

        let mut index = SendMessageW(listbox, LB_GETCURSEL, 0, 0);
        SendMessageW(listbox, LB_DELETESTRING, index as WPARAM, 0);
        SetFocus(listbox);

        if index != 0 {
            index = index - 1;
            SendMessageW(listbox, LB_SETCURSEL, index as WPARAM, 0);
        } else {
            SendMessageW(listbox, LB_SETCURSEL, index as WPARAM, 0);
        }
    }
}

unsafe fn populate_listbox() {
    let commands = get_file();

    if let Some(commands) = commands {
        for command in commands.file {
            add_item(
                to_utf16(&command)
            );
        }

        for command in commands.windows {
            add_item(
                to_utf16(&command)
            );
        }

        for command in commands.terminal {
            add_item(
                to_utf16(&command)
            );
        }
    }
}

pub unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            let commands = get_commands();
            let content = get_file();

            if let Some(content) = content {
                if commands != content {
                    let prompt = MessageBoxW(
                         null_mut(),
                         to_wstr("Do you want to save changes?").as_ptr(),
                         to_wstr("undertasker").as_ptr(),
                         MB_ICONINFORMATION | MB_YESNO | MB_APPLMODAL,
                     );

                    if prompt == 6 {
                        save_file();
                    }
                }
            }

            unload_fonts();
            unload_brushes();
            unregister_custom_classes();
            DestroyWindow(hwnd);
            0 as LRESULT
        },

        WM_COMMAND => {
            match wparam as i32 {
                BTN_ADD => {
                    let edit = GetDlgItem(hwnd, EDIT_COMMAND);
                    let length = GetWindowTextLengthW(edit);
                    let mut text: Vec<u16> = vec![0; (length+1) as usize];

                    let listbox = GetDlgItem(hwnd, LB_COMMAND);

                    if GetWindowTextW(edit, text.as_mut_ptr() as LPWSTR, length + 1) != 0 {
                        add_item(text);
                        SetWindowTextW(edit, null_mut());
                        InvalidateRect(listbox, null_mut(), FALSE);
                    } else {
                        MessageBoxW(
                             null_mut(),
                             to_wstr("Please enter a command.").as_ptr(),
                             to_wstr("undertasker").as_ptr(),
                             MB_ICONINFORMATION | MB_OK,
                         );
                    }

                    SetFocus(edit);
                },

                BTN_BROWSE => {
                    thread::spawn(|| {
                        let mut buffer = vec![0u16; 1024];

                        let mut filename = OPENFILENAMEW {
                            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
                            hwndOwner: null_mut(),
                            hInstance: HINSTANCE,
                            lpstrFilter: null_mut(),
                            lpstrCustomFilter: null_mut(),
                            nMaxCustFilter: 0,
                            nFilterIndex: 1,
                            lpstrFile: buffer.as_mut_ptr(),
                            nMaxFile: buffer.len() as u32,
                            lpstrFileTitle: null_mut(),
                            nMaxFileTitle: 0,
                            lpstrInitialDir: null_mut(),
                            lpstrTitle: null_mut(),
                            Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST,
                            nFileOffset: 0,
                            nFileExtension: 0,
                            lpstrDefExt: null_mut(),
                            lCustData: 0,
                            lpfnHook: None,
                            lpTemplateName: null_mut(),
                            pvReserved: null_mut(),
                            dwReserved: 0,
                            FlagsEx: 0
                        };

                        let result = GetOpenFileNameW(&mut filename);

                        if result != 0 {
                            if let Some(path) = from_wstr(&buffer) {
                                add_item(
                                    to_wstr(&path)
                                );
                            }
                        }
                    });
                },

                BTN_REMOVE => {
                    let listbox = GetDlgItem(hwnd, LB_COMMAND);
                    let count = SendMessageW(listbox, LB_GETCOUNT, 0, 0);

                    if count != 0 {
                        let mut rect = std::mem::MaybeUninit::uninit().assume_init();
                        GetClientRect(hwnd, &mut rect);
                        delete_item();
                    } else {
                        MessageBoxW(
                             null_mut(),
                             to_wstr("No command to remove.").as_ptr(),
                             to_wstr("undertasker").as_ptr(),
                             MB_ICONINFORMATION | MB_OK,
                         );
                    }
                },

                BTN_RUN => {
                    thread::spawn(|| {
                        run();
                    });
                },

                BTN_SAVE => {
                    save_file();
                },

                _ => {}
            }
            0 as LRESULT
        },

        WM_CTLCOLOREDIT => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, RGB(216, 211, 225));

            return BRUSH_BLACK_1 as isize;
        },

        WM_CTLCOLORLISTBOX => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);

            return BRUSH_BLACK_1 as isize;
        },

        WM_DRAWITEM => {
            match wparam as i32 {
                LB_COMMAND => {
                    let listbox = GetDlgItem(hwnd, LB_COMMAND);
                    let count = SendMessageW(listbox, LB_GETCOUNT, 0, 0);

                    if count == 0 {
                        SendMessageW(listbox, LB_SETSEL, FALSE as WPARAM, -1);
                    } else {
                        let mut pdis = *(lparam as LPDRAWITEMSTRUCT);
                        let buffer: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];

                        SendMessageW(
                            pdis.hwndItem,
                            LB_GETTEXT,
                            pdis.itemID as WPARAM,
                            buffer.as_ptr() as LPARAM
                        );

                        let active = GetFocus();

                        if pdis.itemState & ODA_DRAWENTIRE != 0 || pdis.itemState & ODA_SELECT != 0 {
                            FillRect(pdis.hDC, &mut pdis.rcItem, BRUSH_BLACK_1);
                            SetBkColor(pdis.hDC, RGB(229, 225, 222));
                            SetTextColor(pdis.hDC, RGB(216, 211, 225));
                        }

                        if pdis.itemState & ODS_FOCUS != 0 || active != listbox && pdis.itemState & ODS_SELECTED != 0 {
                            FillRect(pdis.hDC, &mut pdis.rcItem, BRUSH_PURPLE_0);
                            SetBkMode(pdis.hDC, TRANSPARENT as i32);
                            SetTextColor(pdis.hDC, RGB(229, 225, 222));
                        } else {
                            FillRect(pdis.hDC, &mut pdis.rcItem, BRUSH_BLACK_1);
                            SetBkColor(pdis.hDC, RGB(98, 97, 171));
                            SetTextColor(pdis.hDC, RGB(229, 225, 222));
                        }

                        pdis.rcItem.left = 3;

                        DrawTextW(
                            pdis.hDC,
                            buffer.as_ptr(),
                            -1,
                            &mut pdis.rcItem,
                            DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOFULLWIDTHCHARBREAK | DT_NOCLIP
                        );

                        if pdis.itemState == ODS_FOCUS {
                            DrawFocusRect(pdis.hDC, &pdis.rcItem);
                        }
                    }
                },

                _ => {}
            };
            0 as LRESULT
        },

        WM_CREATE => {
            let icon = LoadIconW(
                HINSTANCE,
                MAKEINTRESOURCEW(1000)
            );

            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, icon as isize);
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, icon as isize);

            register_custom_classes();

            let listbox = create_listbox(hwnd, LB_COMMAND);
            populate_listbox();

            SendMessageW(listbox, LB_SETCURSEL, FALSE as WPARAM, 0);
            SendMessageW(listbox, LB_SETTOPINDEX, 0, 0);

            let close_button = create_close_button(hwnd, BTN_CLOSE);
            let edit = create_edit(hwnd, EDIT_COMMAND);

            let add = create_button(hwnd, BTN_ADD, "Add");
            let remove = create_button(hwnd, BTN_REMOVE, "Remove");
            let browse = create_button(hwnd, BTN_BROWSE, "Browse");
            let run = create_button(hwnd, BTN_RUN, "Run");
            let save = create_button(hwnd, BTN_SAVE, "Save");

            SetWindowPos(
                close_button,
                HWND_TOP,
                521,
                2,
                24,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                edit,
                HWND_TOP,
                20,
                60,
                422,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                listbox,
                HWND_TOP,
                20,
                100,
                500,
                140,
                SWP_NOZORDER
            );

            SetWindowPos(
                run,
                HWND_TOP,
                456,
                256,
                64,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                save,
                HWND_TOP,
                378,
                256,
                64,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                add,
                HWND_TOP,
                456,
                60,
                64,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                remove,
                HWND_TOP,
                98,
                256,
                64,
                24,
                SWP_NOZORDER
            );

            SetWindowPos(
                browse,
                HWND_TOP,
                20,
                256,
                64,
                24,
                SWP_NOZORDER
            );

            0 as LRESULT
        },

        WM_DESTROY => {
            PostQuitMessage(0);
            0 as LRESULT
        },

        WM_NCHITTEST => {
            let mut hit = DefWindowProcW(hwnd, msg, wparam, lparam);

            if hit == HTCLIENT {
                hit = HTCAPTION;
            }

            return hit;
        },

        _ => DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub fn register_window() {
    unsafe {
        HINSTANCE = GetModuleHandleW(null());
        load_fonts();
        load_brushes();

        let wndclassex = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE,
            hIcon: LoadIconW(null_mut(), MAKEINTRESOURCEW(1000)),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: BRUSH_BLACK_0,
            lpszClassName: to_wstr("WINDOW").as_ptr(),
            hIconSm: null_mut(),
            lpszMenuName: null(),
        };

        let atom = RegisterClassExW(&wndclassex);

        if atom == 0 {
            MessageBoxW(
                null_mut(),
                to_wstr("This program requires Windows NT!").as_ptr(),
                to_wstr("undertasker").as_ptr(),
                MB_ICONERROR,
            );

            return;
        }
    }
}

pub fn create_window() {
    unsafe {
        let desktop = GetDesktopWindow();

        let mut rect = std::mem::MaybeUninit::uninit().assume_init();
        GetClientRect(desktop, &mut rect);

        let hwnd = CreateWindowExW(
            0,
            to_wstr("WINDOW").as_ptr(),
            to_wstr("undertasker").as_ptr(),
            WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_BORDER | WS_POPUP,
            (rect.right - 512) / 2,
            (rect.bottom - 275) / 2,
            550,
            300,
            null_mut(),
            null_mut(),
            HINSTANCE,
            null_mut()
        );

        if hwnd.is_null() {
            return;
        }

        ShowWindow(hwnd, SW_SHOW);

        if UpdateWindow(hwnd) == 0 {
            return;
        }
    }
}

pub fn message_loop() -> WPARAM {
    unsafe {
        let mut msg: MSG = std::mem::MaybeUninit::uninit().assume_init();

        loop {
            let queue = GetMessageW(&mut msg, null_mut(), 0, 0);

            if queue == -1 {
                return 0;
            } else if queue == 0 {
                break;
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        return msg.wParam;
    }
}
