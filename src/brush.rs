use std::ptr::null_mut;
use winapi::shared::windef::{HBRUSH, HGDIOBJ, HPEN};
use winapi::um::wingdi::{RGB, CreatePen, CreateSolidBrush, DeleteObject, PS_SOLID};


#[allow(dead_code)]
pub enum Black
{
    B0, B1
}

#[allow(dead_code)]
pub enum Purple
{
    P0, P1, P2
}

#[allow(dead_code)]
pub enum White
{
    W0
}


pub static mut BRUSH_BLACK_0: HBRUSH = null_mut();
pub static mut BRUSH_BLACK_1: HBRUSH = null_mut();

pub static mut BRUSH_PURPLE_0: HBRUSH = null_mut();
pub static mut BRUSH_PURPLE_1: HBRUSH = null_mut();
pub static mut BRUSH_PURPLE_2: HBRUSH = null_mut();

pub static mut BRUSH_WHITE_0: HBRUSH = null_mut();

pub static mut PEN_PURPLE_0: HPEN = null_mut();
pub static mut PEN_PURPLE_1: HPEN = null_mut();
pub static mut PEN_WHITE_0: HPEN = null_mut();


pub fn load_brushes()
{
    unsafe
    {
        BRUSH_BLACK_0 = CreateSolidBrush(RGB(11, 15, 16));
        BRUSH_BLACK_1 = CreateSolidBrush(RGB(19, 21, 23));

        BRUSH_PURPLE_0 = CreateSolidBrush(RGB(98, 97, 171));
        BRUSH_PURPLE_1 = CreateSolidBrush(RGB(75, 74, 139));
        BRUSH_PURPLE_2 = CreateSolidBrush(RGB(47, 46, 87));

        BRUSH_WHITE_0 = CreateSolidBrush(RGB(216, 211, 225));

        PEN_PURPLE_0 = CreatePen(PS_SOLID as i32, 2, RGB(98, 97, 171));
        PEN_PURPLE_1 = CreatePen(PS_SOLID as i32, 2, RGB(75, 74, 139));

        PEN_WHITE_0 = CreatePen(PS_SOLID as i32, 1, RGB(216, 211, 225));
    }
}

pub fn unload_brushes()
{
    unsafe
    {
        DeleteObject(BRUSH_BLACK_0 as HGDIOBJ);
        DeleteObject(BRUSH_BLACK_1 as HGDIOBJ);

        DeleteObject(BRUSH_PURPLE_0 as HGDIOBJ);
        DeleteObject(BRUSH_PURPLE_1 as HGDIOBJ);
        DeleteObject(BRUSH_PURPLE_2 as HGDIOBJ);

        DeleteObject(BRUSH_WHITE_0 as HGDIOBJ);

        DeleteObject(PEN_PURPLE_0 as HGDIOBJ);
        DeleteObject(PEN_PURPLE_1 as HGDIOBJ);

        DeleteObject(PEN_WHITE_0 as HGDIOBJ);
    }
}
