// fonts.rs - Cached font management for memory efficiency

use std::sync::atomic::{AtomicPtr, Ordering};
use windows::{
    core::*,
    Win32::Graphics::Gdi::*,
};
use crate::constants::MENU_FONT_SIZE;

// Embed the Ekush font directly into the executable
const EKUSH_FONT_DATA: &[u8] = include_bytes!("../fonts/Ekush-Regular.ttf");

// Thread-safe font handles using AtomicPtr
static MENU_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

// Cached fonts for widget (created once, reused)
static FONT_LINE1_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static FONT_LINE2_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static FONT_LINE3_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

// Cached fonts for calendar
static CAL_HEADER_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static CAL_NAV_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static CAL_WEEKDAY_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static CAL_DATE_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static CAL_SUB_FONT_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

#[inline]
pub fn get_menu_font() -> HFONT {
    HFONT(MENU_FONT_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_font_line1() -> HFONT {
    HFONT(FONT_LINE1_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_font_line2() -> HFONT {
    HFONT(FONT_LINE2_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_font_line3() -> HFONT {
    HFONT(FONT_LINE3_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_cal_header_font() -> HFONT {
    HFONT(CAL_HEADER_FONT_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_cal_nav_font() -> HFONT {
    HFONT(CAL_NAV_FONT_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_cal_weekday_font() -> HFONT {
    HFONT(CAL_WEEKDAY_FONT_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_cal_date_font() -> HFONT {
    HFONT(CAL_DATE_FONT_PTR.load(Ordering::Relaxed))
}

#[inline]
pub fn get_cal_sub_font() -> HFONT {
    HFONT(CAL_SUB_FONT_PTR.load(Ordering::Relaxed))
}

/// Create a font with the Ekush typeface
#[inline]
fn create_ekush_font(size: i32, weight: i32) -> HFONT {
    unsafe {
        CreateFontW(
            size, 0, 0, 0,
            weight,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Ekush"),
        )
    }
}

/// Install the embedded font and create all cached font handles
pub fn install_fonts() {
    unsafe {
        let mut num_fonts: u32 = 0;
        let _font_handle = AddFontMemResourceEx(
            EKUSH_FONT_DATA.as_ptr() as *const std::ffi::c_void,
            EKUSH_FONT_DATA.len() as u32,
            None,
            &mut num_fonts,
        );

        if num_fonts > 0 {
            // Menu font
            MENU_FONT_PTR.store(create_ekush_font(MENU_FONT_SIZE, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
            
            // Widget fonts (cached)
            FONT_LINE1_PTR.store(create_ekush_font(26, FW_SEMIBOLD.0 as i32).0, Ordering::Relaxed);
            FONT_LINE2_PTR.store(create_ekush_font(22, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
            FONT_LINE3_PTR.store(create_ekush_font(18, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
            
            // Calendar fonts (cached)
            CAL_HEADER_FONT_PTR.store(create_ekush_font(22, FW_BOLD.0 as i32).0, Ordering::Relaxed);
            CAL_NAV_FONT_PTR.store(create_ekush_font(20, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
            CAL_WEEKDAY_FONT_PTR.store(create_ekush_font(14, FW_SEMIBOLD.0 as i32).0, Ordering::Relaxed);
            CAL_DATE_FONT_PTR.store(create_ekush_font(18, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
            CAL_SUB_FONT_PTR.store(create_ekush_font(14, FW_NORMAL.0 as i32).0, Ordering::Relaxed);
        }
    }
}
