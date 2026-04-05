use std::collections::HashSet;
use std::sync::MutexGuard;
use std::{thread, time::Duration};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetSystemMetrics, GetWindowRect, IsZoomed, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos,
};

use crate::app_state;

const WINDOW_SETTLE_DELAY_MS: u64 = 50;

fn is_system_window(hwnd: HWND) -> bool {
    unsafe {
        let mut class_name: [u16; 256] = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class_name);

        if len == 0 {
            return false;
        }

        let name = String::from_utf16_lossy(&class_name[..len as usize]);

        // Filter out Windows system UI
        name.contains("Windows.UI.Core") || name.contains("Shell_TrayWnd") || name.contains("Start")
    }
}

pub fn handle_window(hwnd: HWND) {
    if hwnd.0 == 0 {
        return;
    }

    if is_system_window(hwnd) {
        return;
    }

    let state: &app_state::AppState = app_state::get();

    // Check enabled
    let enabled: MutexGuard<'_, bool> = state.enabled.lock().unwrap();
    if !*enabled {
        return;
    }
    drop(enabled);

    unsafe {
        if IsZoomed(hwnd).as_bool() {
            return;
        }
    }

    let mut seen: MutexGuard<'_, HashSet<isize>> = state.seen_windows.lock().unwrap();
    if seen.contains(&hwnd.0) {
        return;
    }

    thread::sleep(Duration::from_millis(WINDOW_SETTLE_DELAY_MS));

    centre_window(hwnd);

    seen.insert(hwnd.0);
}

fn centre_window(hwnd: HWND) {
    unsafe {
        let mut rect: RECT = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width: i32 = rect.right - rect.left;
        let height: i32 = rect.bottom - rect.top;

        if width < 200 || height < 200 {
            return;
        }

        let screen_width: i32 = GetSystemMetrics(SM_CXSCREEN);
        let screen_height: i32 = GetSystemMetrics(SM_CYSCREEN);

        let x: i32 = (screen_width - width) / 2;
        let y: i32 = (screen_height - height) / 2;

        let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
    }
}
