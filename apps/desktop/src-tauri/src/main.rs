#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() {
    #[cfg(not(feature = "isolation-ui-test"))]
    kraqel_desktop_lib::run();
    #[cfg(all(windows, feature = "isolation-ui-test"))]
    kraqel_desktop_lib::run_isolation_ui_test();
    #[cfg(all(not(windows), feature = "isolation-ui-test"))]
    std::process::exit(2);
}
