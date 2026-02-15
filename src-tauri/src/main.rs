#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Force X11 backend on Linux so that always-on-top works.
    // Wayland doesn't support the always-on-top window hint.
    #[cfg(target_os = "linux")]
    std::env::set_var("GDK_BACKEND", "x11");

    sticky_md_lib::run()
}
