// registry.rs - Windows registry operations for settings persistence

use crate::AUTOSTART_ENABLED;
use crate::constants::*;
use std::sync::atomic::Ordering;
use windows::{Win32::System::Registry::*, core::*};

pub unsafe fn is_autostart_enabled() -> bool {
    let key_path = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let mut hkey = HKEY::default();

    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_READ, &mut hkey) }.is_ok() {
        let value_name = HSTRING::from(APP_NAME);
        let result = unsafe { RegQueryValueExW(hkey, &value_name, None, None, None, None) };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        result.is_ok()
    } else {
        false
    }
}

pub unsafe fn load_country_selection() -> u32 {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();
    let mut country: u32 = 0;

    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_READ, &mut hkey) }.is_ok() {
        let mut data: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let value_name = HSTRING::from(COUNTRY_KEY);

        if unsafe {
            RegQueryValueExW(
                hkey,
                &value_name,
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut size),
            )
        }
        .is_ok()
        {
            country = data;
        }
        unsafe {
            let _ = RegCloseKey(hkey);
        }
    }
    country
}

pub unsafe fn save_country_selection(country: u32) {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();

    if unsafe { RegCreateKeyW(HKEY_CURRENT_USER, key_path, &mut hkey) }.is_ok() {
        let value_name = HSTRING::from(COUNTRY_KEY);
        unsafe {
            let _ = RegSetValueExW(
                hkey,
                &value_name,
                Some(0),
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &country as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            );
            let _ = RegCloseKey(hkey);
        }
    }
}

pub unsafe fn load_position() -> (i32, i32) {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();
    let mut x: i32 = -1;
    let mut y: i32 = -1;

    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_READ, &mut hkey) }.is_ok() {
        let mut data_x: u32 = 0;
        let mut data_y: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;

        let value_x = HSTRING::from(POS_KEY_X);
        let value_y = HSTRING::from(POS_KEY_Y);

        if unsafe {
            RegQueryValueExW(
                hkey,
                &value_x,
                None,
                None,
                Some(&mut data_x as *mut u32 as *mut u8),
                Some(&mut size),
            )
        }
        .is_ok()
        {
            x = data_x as i32;
        }

        size = std::mem::size_of::<u32>() as u32;
        if unsafe {
            RegQueryValueExW(
                hkey,
                &value_y,
                None,
                None,
                Some(&mut data_y as *mut u32 as *mut u8),
                Some(&mut size),
            )
        }
        .is_ok()
        {
            y = data_y as i32;
        }

        unsafe {
            let _ = RegCloseKey(hkey);
        }
    }

    (x, y)
}

pub unsafe fn save_position(x: i32, y: i32) {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();

    if unsafe { RegCreateKeyW(HKEY_CURRENT_USER, key_path, &mut hkey) }.is_ok() {
        let value_x = HSTRING::from(POS_KEY_X);
        let value_y = HSTRING::from(POS_KEY_Y);
        let x_u32 = x as u32;
        let y_u32 = y as u32;

        unsafe {
            let _ = RegSetValueExW(
                hkey,
                &value_x,
                Some(0),
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &x_u32 as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            );

            let _ = RegSetValueExW(
                hkey,
                &value_y,
                Some(0),
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &y_u32 as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            );

            let _ = RegCloseKey(hkey);
        }
    }
}

pub unsafe fn toggle_autostart(enable: bool) {
    let key_path = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let mut hkey = HKEY::default();

    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_WRITE, &mut hkey) }.is_ok()
    {
        let value_name = HSTRING::from(APP_NAME);

        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path_str = exe_path.to_string_lossy();
                let path_wide: Vec<u16> =
                    path_str.encode_utf16().chain(std::iter::once(0)).collect();
                let path_bytes = unsafe {
                    std::slice::from_raw_parts(path_wide.as_ptr() as *const u8, path_wide.len() * 2)
                };
                unsafe {
                    let _ = RegSetValueExW(hkey, &value_name, Some(0), REG_SZ, Some(path_bytes));
                }
                AUTOSTART_ENABLED.store(true, Ordering::SeqCst);
            }
        } else {
            unsafe {
                let _ = RegDeleteValueW(hkey, &value_name);
            }
            AUTOSTART_ENABLED.store(false, Ordering::SeqCst);
        }
        unsafe {
            let _ = RegCloseKey(hkey);
        }
    }
}

/// Check if the app has run before
pub unsafe fn has_run_before() -> bool {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();

    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_READ, &mut hkey) }.is_ok() {
        let mut data: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let value_name = HSTRING::from("HasRun");

        let result = unsafe {
            RegQueryValueExW(
                hkey,
                &value_name,
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut size),
            )
        };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        result.is_ok() && data == 1
    } else {
        false
    }
}

/// Mark that the app has run before
pub unsafe fn mark_has_run() {
    let key_path = w!("Software\\BongoWidget");
    let mut hkey = HKEY::default();

    if unsafe { RegCreateKeyW(HKEY_CURRENT_USER, key_path, &mut hkey) }.is_ok() {
        let value_name = HSTRING::from("HasRun");
        let data: u32 = 1;
        unsafe {
            let _ = RegSetValueExW(
                hkey,
                &value_name,
                Some(0),
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &data as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            );
            let _ = RegCloseKey(hkey);
        }
    }
}
