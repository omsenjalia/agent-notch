//! Windows desktop tools for the agent brain: mouse, keyboard, windows,
//! clipboard and launching apps.
//!
//! All input goes through Win32 calls (`SetCursorPos`, `mouse_event`,
//! `keybd_event`). Each tool returns a short human-readable output string
//! describing what it did, for the brain's conversation.

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use serde_json::Value;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse as KBM;
use windows::Win32::UI::Shell;
use windows::Win32::UI::WindowsAndMessaging as WM;

use crate::agent::vision;

/// `mouse_event` flags we use.
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0001;
const MOUSEEVENTF_LEFTUP: u32 = 0x0002;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0004;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0008;
const MOUSEEVENTF_WHEEL: u32 = 0x0800;
const WHEEL_DELTA: i32 = 120;
/// `keybd_event` flag for key-up.
const KEYEVENTF_KEYUP: u32 = 0x0002;

/// Virtual-key codes we use.
const VK_CONTROL: u16 = 0x11;
const VK_SHIFT: u16 = 0x10;
const VK_MENU: u16 = 0x12;
const VK_LWIN: u16 = 0x5B;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0D;
const VK_SPACE: u16 = 0x20;
const VK_0: u16 = 0x30;
const VK_9: u16 = 0x39;
const VK_A: u16 = 0x41;
const VK_Z: u16 = 0x5A;
const VK_OEM_PERIOD: u16 = 0xBE;
const VK_OEM_COMMA: u16 = 0xBC;
const VK_OEM_SEMICOLON: u16 = 0xBA;
const VK_OEM_SLASH: u16 = 0xBF;

fn mouse(flags: u32, data: i32) {
    unsafe { KBM::mouse_event(KBM::MOUSE_EVENT_FLAGS(flags), 0, 0, data, 0) }
}

fn send_key(vk: u16, down: bool) {
    unsafe {
        KBM::keybd_event(
            vk as u8,
            0,
            KBM::KEYBD_EVENT_FLAGS(if down { 0 } else { KEYEVENTF_KEYUP }),
            0,
        )
    }
}

pub fn click(x: i32, y: i32) {
    unsafe {
        let _ = WM::SetCursorPos(x, y);
    }
    mouse(MOUSEEVENTF_LEFTDOWN, 0);
    mouse(MOUSEEVENTF_LEFTUP, 0);
}

pub fn right_click(x: i32, y: i32) {
    unsafe {
        let _ = WM::SetCursorPos(x, y);
    }
    mouse(MOUSEEVENTF_RIGHTDOWN, 0);
    mouse(MOUSEEVENTF_RIGHTUP, 0);
}

pub fn double_click(x: i32, y: i32) {
    click(x, y);
    sleep(Duration::from_millis(70));
    click(x, y);
}

pub fn drag(x1: i32, y1: i32, x2: i32, y2: i32) {
    unsafe {
        let _ = WM::SetCursorPos(x1, y1);
    }
    mouse(MOUSEEVENTF_LEFTDOWN, 0);
    // Move in small steps so a long drag looks the same as a user dragging.
    let steps = 12;
    for i in 1..=steps {
        let x = x1 + (x2 - x1) * i / steps;
        let y = y1 + (y2 - y1) * i / steps;
        unsafe {
            let _ = WM::SetCursorPos(x, y);
        }
    }
    mouse(MOUSEEVENTF_LEFTUP, 0);
}

pub fn scroll(x: i32, y: i32, up: bool, amount: u32) {
    unsafe {
        let _ = WM::SetCursorPos(x, y);
    }
    let delta = if up { amount as i32 } else { -(amount as i32) };
    mouse(MOUSEEVENTF_WHEEL, delta * WHEEL_DELTA);
}

/// A virtual-key code from a printable character or common symbol.
fn vk_for_char(c: char) -> Option<u16> {
    let u = c.to_ascii_uppercase() as u16;
    if (0x41..=0x5A).contains(&u) {
        return Some(VK_A + u - 0x41);
    }
    if (0x30..=0x39).contains(&u) {
        return Some(VK_0 + u - 0x30);
    }
    Some(match c {
        ' ' => VK_SPACE,
        '\t' => VK_TAB,
        '\n' | '\r' => VK_RETURN,
        '.' => VK_OEM_PERIOD,
        ',' => VK_OEM_COMMA,
        ';' => VK_OEM_SEMICOLON,
        '/' => VK_OEM_SLASH,
        _ => return None,
    })
}

/// Type text into whatever has focus. Non-ASCII characters are skipped.
pub fn type_text(text: &str) {
    for c in text.chars() {
        let Some(vk) = vk_for_char(c) else {
            continue;
        };
        if c.is_uppercase() {
            send_key(VK_SHIFT, true);
        }
        send_key(vk, true);
        send_key(vk, false);
        if c.is_uppercase() {
            send_key(VK_SHIFT, false);
        }
    }
}

/// Press a chord like `ctrl+shift+s`. Names: ctrl, shift, alt, win, plus any
/// single printable character.
pub fn press_chord(chord: &str) {
    let mut keys: Vec<u16> = Vec::new();
    for part in chord.split('+') {
        let name = part.trim().to_ascii_lowercase();
        let vk = match name.as_str() {
            "ctrl" | "control" => Some(VK_CONTROL),
            "shift" => Some(VK_SHIFT),
            "alt" => Some(VK_MENU),
            "win" => Some(VK_LWIN),
            _ => name.chars().next().and_then(vk_for_char),
        };
        let Some(vk) = vk else {
            continue;
        };
        keys.push(vk);
    }
    for vk in &keys {
        send_key(*vk, true);
    }
    for vk in keys.iter().rev() {
        send_key(*vk, false);
    }
}

/// One window: title and class name.
#[derive(Clone, Debug, Default)]
pub struct WindowInfo {
    pub title: String,
    pub class_name: String,
}

struct WindowFind {
    needle: String,
    found: Option<HWND>,
}

unsafe extern "system" fn window_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let find: &mut WindowFind = &mut *(lparam.0 as *mut WindowFind);
    if find.found.is_none() {
        let title = window_title(hwnd);
        if !title.is_empty() && title.to_lowercase().contains(&find.needle) {
            find.found = Some(hwnd);
            return BOOL(0);
        }
    }
    BOOL(1)
}

/// Bring a window to the front: raise it and restore it.
pub fn activate_window(title_substr: &str) -> bool {
    let needle = title_substr.to_lowercase();
    let mut find = WindowFind {
        needle,
        found: None,
    };
    let lparam = LPARAM((&mut find as *mut WindowFind) as isize);
    let _ = unsafe { WM::EnumWindows(Some(window_cb), lparam) };
    match find.found {
        Some(hwnd) => {
            unsafe {
                let _ = WM::SetForegroundWindow(hwnd);
                let _ = WM::ShowWindow(hwnd, WM::SW_RESTORE);
            }
            true
        }
        None => false,
    }
}

/// All windows with a visible title.
pub fn list_windows() -> Vec<WindowInfo> {
    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let list: &mut Vec<WindowInfo> = &mut *(lparam.0 as *mut Vec<WindowInfo>);
        let title = window_title(hwnd);
        if !title.is_empty() {
            let mut class = [0u16; 260];
            let len = WM::GetClassNameW(hwnd, &mut class);
            list.push(WindowInfo {
                title,
                class_name: String::from_utf16_lossy(&class[..len.max(0) as usize]),
            });
        }
        BOOL(1)
    }
    let mut list = Vec::new();
    let lparam = LPARAM((&mut list as *mut Vec<WindowInfo>) as isize);
    let _ = unsafe { WM::EnumWindows(Some(cb), lparam) };
    list
}

/// The window's title (empty when the window has none).
pub fn window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { WM::GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

/// The foreground window's title and class name.
pub fn foreground_info() -> (String, String) {
    unsafe {
        let hwnd = WM::GetForegroundWindow();
        let mut class = [0u16; 260];
        let len = WM::GetClassNameW(hwnd, &mut class);
        (
            window_title(hwnd),
            String::from_utf16_lossy(&class[..len.max(0) as usize]),
        )
    }
}

/// Launch a path/URL with the system handler (ShellExecuteW "open").
pub fn launch(path: &str) -> bool {
    let operation = windows::core::HSTRING::from("open");
    let target = windows::core::HSTRING::from(path);
    unsafe {
        let _ = Shell::ShellExecuteW(
            HWND::default(),
            windows::core::PCWSTR(operation.as_ptr()),
            windows::core::PCWSTR(target.as_ptr()),
            windows::core::PCWSTR::null(),
            windows::core::PCWSTR::null(),
            WM::SW_SHOW,
        );
    }
    true
}

/// Read the current clipboard text.
pub fn clipboard_read() -> String {
    crate::notch::clipboard::latest().unwrap_or_default()
}

/// Write text to the clipboard.
pub fn clipboard_write(text: &str) -> bool {
    crate::notch::clipboard::copy_text(text)
}

/// Run a shell command via `cmd /c`.
fn execute_command(command: &str) -> (bool, String) {
    match Command::new("cmd.exe").args(["/c", command]).output() {
        Ok(output) => {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if output.status.success() {
                (
                    true,
                    if out.is_empty() {
                        "command completed".to_owned()
                    } else {
                        out
                    },
                )
            } else {
                (false, out)
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

fn arg_i32(args: &Value, name: &str) -> i32 {
    args.get(name).and_then(Value::as_i64).unwrap_or(0) as i32
}

/// Run a tool by name; returns `(ok, output)`. Assumes the guard already
/// approved the action.
pub fn run_tool(tool: &str, args: &Value) -> (bool, String) {
    match tool {
        "screenshot" => match vision::capture_screenshot(arg_i32(args, "monitor") as usize) {
            Some(img) => (true, format!("{}x{} pixels", img.width(), img.height())),
            None => (false, "no screen at that index".to_owned()),
        },
        "list_windows" => {
            let windows = list_windows();
            let names: Vec<&str> = windows.iter().take(5).map(|w| w.title.as_str()).collect();
            (
                true,
                format!("{} windows: {}", windows.len(), names.join(", ")),
            )
        }
        "get_foreground" => {
            let (title, class) = vision::foreground_info();
            (true, format!("{title} ({class})"))
        }
        "clipboard_read" => {
            let text = clipboard_read();
            (
                true,
                if text.is_empty() {
                    "clipboard is empty".to_owned()
                } else {
                    format!("{} chars", text.chars().count())
                },
            )
        }
        "clipboard_write" => {
            let text = args.get("text").and_then(Value::as_str).unwrap_or_default();
            (clipboard_write(text), "copied to clipboard".to_owned())
        }
        "scroll" => {
            let x = arg_i32(args, "x");
            let y = arg_i32(args, "y");
            let direction = args
                .get("direction")
                .and_then(Value::as_str)
                .unwrap_or("down");
            let amount = args.get("amount").and_then(Value::as_u64).unwrap_or(3) as u32;
            scroll(x, y, direction == "up", amount);
            (true, format!("scrolled {direction}"))
        }
        "click" => {
            click(arg_i32(args, "x"), arg_i32(args, "y"));
            (
                true,
                format!("clicked ({}, {})", arg_i32(args, "x"), arg_i32(args, "y")),
            )
        }
        "right_click" => {
            right_click(arg_i32(args, "x"), arg_i32(args, "y"));
            (true, "right clicked".to_owned())
        }
        "double_click" => {
            double_click(arg_i32(args, "x"), arg_i32(args, "y"));
            (true, "double clicked".to_owned())
        }
        "drag" => {
            let (x1, y1) = (arg_i32(args, "x1"), arg_i32(args, "y1"));
            let (x2, y2) = (arg_i32(args, "x2"), arg_i32(args, "y2"));
            drag(x1, y1, x2, y2);
            (true, format!("dragged ({x1}, {y1}) to ({x2}, {y2})"))
        }
        "type_text" => {
            let text = args.get("text").and_then(Value::as_str).unwrap_or_default();
            type_text(text);
            (true, format!("typed {} chars", text.chars().count()))
        }
        "press_chord" => {
            let chord = args
                .get("chord")
                .and_then(Value::as_str)
                .unwrap_or_default();
            press_chord(chord);
            (true, format!("pressed {chord}"))
        }
        "activate_window" => {
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default();
            (activate_window(title), format!("activated {title:?}"))
        }
        "launch" => {
            let path = args.get("path").and_then(Value::as_str).unwrap_or_default();
            (launch(path), format!("launched {path}"))
        }
        "execute_command" => {
            let command = args
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default();
            execute_command(command)
        }
        other => (false, format!("unknown tool: {other}")),
    }
}

/// JSON schemas telling providers what tools exist and how to call them.
pub fn tool_schemas() -> Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "screenshot",
                "description": "Capture a screenshot of one monitor (the primary screen when monitor is 0 or omitted) so you can see what is on screen.",
                "parameters": { "type": "object", "properties": { "monitor": { "type": "integer", "description": "Monitor index, 0 = primary" } } }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "click",
                "description": "Click at screen coordinates (x, y) — left button.",
                "parameters": { "type": "object", "properties": { "x": { "type": "integer" }, "y": { "type": "integer" } }, "required": ["x", "y"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "right_click",
                "description": "Right-click at screen coordinates (x, y).",
                "parameters": { "type": "object", "properties": { "x": { "type": "integer" }, "y": { "type": "integer" } }, "required": ["x", "y"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "double_click",
                "description": "Double-click at screen coordinates (x, y).",
                "parameters": { "type": "object", "properties": { "x": { "type": "integer" }, "y": { "type": "integer" } }, "required": ["x", "y"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "drag",
                "description": "Drag the left button from (x1, y1) to (x2, y2).",
                "parameters": { "type": "object", "properties": { "x1": { "type": "integer" }, "y1": { "type": "integer" }, "x2": { "type": "integer" }, "y2": { "type": "integer" } }, "required": ["x1", "y1", "x2", "y2"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "type_text",
                "description": "Type text into whatever has keyboard focus.",
                "parameters": { "type": "object", "properties": { "text": { "type": "string", "description": "Text to type" } }, "required": ["text"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "press_chord",
                "description": "Press a keyboard chord like ctrl+s or ctrl+shift+p.",
                "parameters": { "type": "object", "properties": { "chord": { "type": "string" } }, "required": ["chord"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "scroll",
                "description": "Scroll at screen coordinates (x, y) up or down by a number of notches.",
                "parameters": { "type": "object", "properties": { "x": { "type": "integer" }, "y": { "type": "integer" }, "direction": { "type": "string", "enum": ["up", "down"] }, "amount": { "type": "integer", "default": 3 } } }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "clipboard_read",
                "description": "Read the current clipboard text.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "clipboard_write",
                "description": "Write text to the clipboard.",
                "parameters": { "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_windows",
                "description": "List open windows with visible titles.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_foreground",
                "description": "Get the title and class name of the foreground (active) window.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "activate_window",
                "description": "Raise and focus a window by title substring.",
                "parameters": { "type": "object", "properties": { "title": { "type": "string" } }, "required": ["title"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "launch",
                "description": "Launch a path or URL with the system handler.",
                "parameters": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "execute_command",
                "description": "Run a shell command (cmd /c) and return its output. Irreversible: always needs approval.",
                "parameters": { "type": "object", "properties": { "command": { "type": "string" } }, "required": ["command"] }
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_mapping_covers_letters_digits_common() {
        assert_eq!(vk_for_char('a'), Some(VK_A));
        assert_eq!(vk_for_char('Z'), Some(VK_Z));
        assert_eq!(vk_for_char('7'), Some(VK_0 + 7));
        assert_eq!(vk_for_char(' '), Some(VK_SPACE));
        assert_eq!(vk_for_char('\n'), Some(VK_RETURN));
        assert_eq!(vk_for_char('\u{00e9}'), None);
    }

    #[test]
    fn run_tool_unknown_returns_error_string() {
        let (ok, output) = run_tool("not_a_tool", &Value::Null);
        assert!(!ok);
        assert_eq!(output, "unknown tool: not_a_tool");
    }
}
