use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::{
    thread,
    time::{Duration, Instant},
};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, GetMessageW, GetSystemMetrics, GetWindowRect, IsZoomed, MSG,
    SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, WINEVENT_OUTOFCONTEXT,
};
use windows::core::Result;

static SEEN_WINDOWS: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Callback function triggered when a window event occurs
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

        // Ignore early events (existing windows)
        let start = START_TIME.get().unwrap();
        if start.elapsed() < std::time::Duration::from_secs(2) {
            return;
        }

        // Avoid repeated centering
        let seen = SEEN_WINDOWS.get().unwrap();
        let mut seen = seen.lock().unwrap();

        if seen.contains(&hwnd.0) {
            return;
        }

        // Small delay to allow window to stabilise
        thread::sleep(Duration::from_millis(50));

        // Get window dimensions
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Ignore tiny windows
        if width < 200 || height < 200 {
            return;
        }

        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // Calculate centre position
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        // Move window (ignore result silently)
        let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

        seen.insert(hwnd.0);
    }
}

fn main() -> Result<()> {
    unsafe {
        println!("Window centring daemon running...");

        SEEN_WINDOWS.set(Mutex::new(HashSet::new())).unwrap();
        START_TIME.set(Instant::now()).unwrap();

        // Set ip the event hook
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.0 == 0 {
            println!("Failed to set event hook.");
            return Ok(());
        }

        // Message loop (keeps program alive)
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            // Do nothing, just keep running
        }

        // Cleanup (usually never reached)
        let _ = UnhookWinEvent(hook);
    }

    Ok(())
}
