use std::ptr::null_mut;
use winapi::shared::minwindef::FALSE;
use winapi::shared::windef::{HFONT, HGDIOBJ};
use winapi::um::wingdi::{ANSI_CHARSET, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_PITCH, DEFAULT_QUALITY, DeleteObject, FF_DONTCARE, FW_DONTCARE, OUT_TT_PRECIS};

use crate::util::to_wstr;

pub static mut FONT: HFONT = null_mut();

pub unsafe fn load_fonts()
{
    FONT = CreateFontW(
        18,
        0,
        0,
        0,
        FW_DONTCARE,
        FALSE as u32,
        FALSE as u32,
        FALSE as u32,
        ANSI_CHARSET,
        OUT_TT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        DEFAULT_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE,
        to_wstr("Tahoma").as_ptr()
    );
}

pub unsafe fn unload_fonts()
{
    DeleteObject(FONT as HGDIOBJ);
}
