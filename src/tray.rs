use std::sync::MutexGuard;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyWindow, GetCursorPos, HMENU, InsertMenuW, MENU_ITEM_FLAGS,
    PostQuitMessage, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu,
    WM_RBUTTONUP,
};
use windows::core::PCWSTR;

use crate::app_state;

pub const ID_TRAY_EXIT: usize = 1;
pub const ID_TRAY_TOGGLE: usize = 2;

pub fn handle_tray_event(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    unsafe {
        if lparam.0 as u32 == WM_RBUTTONUP {
            let menu: HMENU = CreatePopupMenu().unwrap();

            let state: &app_state::AppState = app_state::get();
            let enabled: MutexGuard<'_, bool> = state.enabled.lock().unwrap();
            let toggle_text: &str = if *enabled { "Disable" } else { "Enable" };
            drop(enabled);

            let toggle_w: Vec<u16> = format!("{}\0", toggle_text).encode_utf16().collect();
            let exit_w: Vec<u16> = "Exit\0".encode_utf16().collect();

            let _ = InsertMenuW(
                menu,
                0,
                MENU_ITEM_FLAGS(0),
                ID_TRAY_TOGGLE,
                PCWSTR(toggle_w.as_ptr()),
            );

            let _ = InsertMenuW(
                menu,
                1,
                MENU_ITEM_FLAGS(0),
                ID_TRAY_EXIT,
                PCWSTR(exit_w.as_ptr()),
            );

            let mut point: POINT = Default::default();
            let _ = GetCursorPos(&mut point);

            let _ = SetForegroundWindow(hwnd);

            let _ = TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                point.x,
                point.y,
                0,
                hwnd,
                None,
            );
        }

        LRESULT(0)
    }
}

pub fn handle_command(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    unsafe {
        match wparam.0 {
            ID_TRAY_EXIT => {
                let _ = DestroyWindow(hwnd);
            }

            ID_TRAY_TOGGLE => {
                let state: &app_state::AppState = app_state::get();
                let mut enabled: MutexGuard<'_, bool> = state.enabled.lock().unwrap();
                *enabled = !*enabled;
            }

            _ => {}
        }

        LRESULT(0)
    }
}

pub fn handle_destroy() -> LRESULT {
    unsafe {
        PostQuitMessage(0);
        LRESULT(0)
    }
}
