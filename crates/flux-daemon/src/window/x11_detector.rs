use std::fs;
use std::path::Path;

use tracing::{debug, trace, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

use super::{WindowDetector, WindowInfo};

pub struct X11WindowDetector {
    connection: RustConnection,
    root_window: Window,
    active_window_atom: u32,
    wm_class_atom: u32,
    wm_pid_atom: u32,
    net_wm_name_atom: u32,
    wm_name_atom: u32,
    utf8_string_atom: u32,
}

impl X11WindowDetector {
    pub fn new() -> Option<Self> {
        let (connection, screen_number) = RustConnection::connect(None)
            .map_err(|error| {
                warn!(%error, "failed to connect to X11 display");
            })
            .ok()?;

        let screen = &connection.setup().roots[screen_number];
        let root_window = screen.root;

        let active_window_atom = connection
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .ok()?
            .reply()
            .ok()?
            .atom;

        let wm_class_atom = AtomEnum::WM_CLASS.into();

        let wm_pid_atom = connection
            .intern_atom(false, b"_NET_WM_PID")
            .ok()?
            .reply()
            .ok()?
            .atom;

        let net_wm_name_atom = connection
            .intern_atom(false, b"_NET_WM_NAME")
            .ok()?
            .reply()
            .ok()?
            .atom;

        let wm_name_atom = AtomEnum::WM_NAME.into();

        let utf8_string_atom = connection
            .intern_atom(false, b"UTF8_STRING")
            .ok()?
            .reply()
            .ok()?
            .atom;

        debug!("X11 window detector initialized");

        Some(Self {
            connection,
            root_window,
            active_window_atom,
            wm_class_atom,
            wm_pid_atom,
            net_wm_name_atom,
            wm_name_atom,
            utf8_string_atom,
        })
    }

    fn get_active_window(&self) -> Option<Window> {
        let reply = self
            .connection
            .get_property(
                false,
                self.root_window,
                self.active_window_atom,
                AtomEnum::WINDOW,
                0,
                1,
            )
            .ok()?
            .reply()
            .ok()?;

        if reply.value.len() >= 4 {
            let window_id = u32::from_ne_bytes([
                reply.value[0],
                reply.value[1],
                reply.value[2],
                reply.value[3],
            ]);
            if window_id != 0 {
                return Some(window_id);
            }
        }

        None
    }

    fn get_window_class(&self, window: Window) -> Option<String> {
        let reply = self
            .connection
            .get_property(false, window, self.wm_class_atom, AtomEnum::STRING, 0, 2048)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.is_empty() {
            return None;
        }

        let parts: Vec<&str> = std::str::from_utf8(&reply.value)
            .ok()?
            .split('\0')
            .filter(|s| !s.is_empty())
            .collect();

        parts.get(1).or(parts.first()).map(|s| s.to_string())
    }

    fn get_window_title(&self, window: Window) -> Option<String> {
        if let Some(title) = self.get_net_wm_name(window) {
            return Some(title);
        }

        self.get_wm_name(window)
    }

    fn get_net_wm_name(&self, window: Window) -> Option<String> {
        let reply = self
            .connection
            .get_property(
                false,
                window,
                self.net_wm_name_atom,
                self.utf8_string_atom,
                0,
                2048,
            )
            .ok()?
            .reply()
            .ok()?;

        if reply.value.is_empty() {
            return None;
        }

        String::from_utf8(reply.value).ok()
    }

    fn get_wm_name(&self, window: Window) -> Option<String> {
        let reply = self
            .connection
            .get_property(false, window, self.wm_name_atom, AtomEnum::STRING, 0, 2048)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.is_empty() {
            return None;
        }

        std::str::from_utf8(&reply.value)
            .ok()
            .map(|s| s.trim_end_matches('\0').to_string())
    }

    fn get_window_pid(&self, window: Window) -> Option<u32> {
        let reply = self
            .connection
            .get_property(false, window, self.wm_pid_atom, AtomEnum::CARDINAL, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.len() >= 4 {
            let pid = u32::from_ne_bytes([
                reply.value[0],
                reply.value[1],
                reply.value[2],
                reply.value[3],
            ]);
            return Some(pid);
        }

        None
    }

    fn is_claude_code_running(&self, window_pid: u32) -> bool {
        self.find_claude_in_process_tree(window_pid)
    }

    fn find_claude_in_process_tree(&self, parent_pid: u32) -> bool {
        let proc_path = Path::new("/proc");

        if let Ok(entries) = fs::read_dir(proc_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();

                if let Ok(pid) = name.parse::<u32>() {
                    if self.is_child_of(pid, parent_pid) && self.is_claude_process(pid) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn is_child_of(&self, pid: u32, parent_pid: u32) -> bool {
        let status_path = format!("/proc/{}/status", pid);

        if let Ok(content) = fs::read_to_string(&status_path) {
            for line in content.lines() {
                if let Some(ppid_str) = line.strip_prefix("PPid:\t") {
                    if let Ok(ppid) = ppid_str.trim().parse::<u32>() {
                        if ppid == parent_pid {
                            return true;
                        }
                        if ppid != 0 && ppid != 1 {
                            return self.is_child_of(ppid, parent_pid);
                        }
                    }
                }
            }
        }

        false
    }

    fn is_claude_process(&self, pid: u32) -> bool {
        let cmdline_path = format!("/proc/{}/cmdline", pid);

        if let Ok(content) = fs::read_to_string(&cmdline_path) {
            let lowercase = content.to_lowercase();
            if lowercase.contains("claude") && !lowercase.contains("claudecode") {
                trace!(pid, "found claude process");
                return true;
            }
        }

        false
    }
}

impl WindowDetector for X11WindowDetector {
    fn get_active_window_info(&self) -> Option<WindowInfo> {
        let window = self.get_active_window()?;
        let window_class = self.get_window_class(window)?;
        let window_title = self.get_window_title(window);

        let lowercase_class = window_class.to_lowercase();

        if lowercase_class.contains("cursor") || lowercase_class.contains("code") {
            if let Some(pid) = self.get_window_pid(window) {
                if self.is_claude_code_running(pid) {
                    debug!(window_class = %window_class, "detected Claude Code in editor");
                    return Some(WindowInfo::new("Claude Code".to_string(), window_title));
                }
            }
        }

        debug!(window_class = %window_class, window_title = ?window_title, "detected active window");
        Some(WindowInfo::new(window_class, window_title))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detector_can_be_created_or_fails_gracefully() {
        let detector = X11WindowDetector::new();

        match detector {
            Some(_) => println!("X11 detector created successfully"),
            None => println!("X11 not available (expected in CI)"),
        }
    }
}
