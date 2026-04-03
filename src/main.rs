#![windows_subsystem = "windows"]

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::{thread, time::Duration};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, EnumWindows, GetMessageW, GetSystemMetrics, GetWindowRect, IsZoomed,
    MSG, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, WINEVENT_OUTOFCONTEXT,
};
use windows::core::Result;

static SEEN_WINDOWS: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();

/// Enumerates all existing windows at startup
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let seen_ptr: *mut Mutex<HashSet<isize>> = lparam.0 as *mut Mutex<HashSet<isize>>;

        if !seen_ptr.is_null() {
            let seen: &Mutex<HashSet<isize>> = &*seen_ptr;
            let mut seen: MutexGuard<'_, HashSet<isize>> = seen.lock().unwrap();
            seen.insert(hwnd.0);
        }

        true.into()
    }
}

/// Callback function triggered when a window gains focus
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread_id: u32,
    _time: u32,
) {
    unsafe {
        if hwnd.0 == 0 {
            return;
        }

        // Ignore maximised windows
        if IsZoomed(hwnd).as_bool() {
            return;
        }

        let seen: &Mutex<HashSet<isize>> = SEEN_WINDOWS.get().unwrap();
        let mut seen: MutexGuard<'_, HashSet<isize>> = seen.lock().unwrap();

        // Ignore already processed windows
        if seen.contains(&hwnd.0) {
            return;
        }

        // Small delay to allow window to stabilise
        thread::sleep(Duration::from_millis(50));

        // Get window dimensions
        let mut rect: RECT = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width: i32 = rect.right - rect.left;
        let height: i32 = rect.bottom - rect.top;

        // Ignore tiny windows
        if width < 200 || height < 200 {
            return;
        }

        // Get screen dimensions
        let screen_width: i32 = GetSystemMetrics(SM_CXSCREEN);
        let screen_height: i32 = GetSystemMetrics(SM_CYSCREEN);

        // Calculate centre position
        let x: i32 = (screen_width - width) / 2;
        let y: i32 = (screen_height - height) / 2;

        // Move window (ignore result silently)
        let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

        // Mark as processed
        seen.insert(hwnd.0);
    }
}

fn main() -> Result<()> {
    unsafe {
        // Initialise global state
        SEEN_WINDOWS.set(Mutex::new(HashSet::new())).unwrap();

        // Populate existing windows ONCE
        let seen: &Mutex<HashSet<isize>> = SEEN_WINDOWS.get().unwrap();
        let seen_ptr: *mut Mutex<HashSet<isize>> =
            seen as *const Mutex<HashSet<isize>> as *mut Mutex<HashSet<isize>>;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(seen_ptr as isize));

        // Set up the event hook
        let hook: HWINEVENTHOOK = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.0 == 0 {
            return Ok(());
        }

        // Message loop (keeps program alive)
        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {}

        // Cleanup (usually never reached)
        let _ = UnhookWinEvent(hook);
    }

    Ok(())
}
