use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, GetMessageW, GetSystemMetrics, GetWindowRect, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, WINEVENT_OUTOFCONTEXT,
};
use windows::core::Result;

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

        // Get window dimensions
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // Calculate centre position
        let x = (screen_width - width) / 2;
        let y = (screen_width - height) / 2;

        // Move window (ignore result silently)
        let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
    }
}

fn main() -> Result<()> {
    unsafe {
        println!("Window centring daemon running...");

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
