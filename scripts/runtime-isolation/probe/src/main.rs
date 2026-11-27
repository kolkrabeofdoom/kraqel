//! Bounded, synthetic AppContainer experiments, not a production launcher.
// Pure rules; scoped Windows P2a records use them after native control checks.
#[cfg(windows)]
#[path = "../../native-window/src/package.rs"]
mod native_win32_package;
#[cfg(any(windows, test))]
mod policy;
#[cfg(any(windows, test))]
mod product_start;
#[cfg(any(windows, test))]
mod product_start_p2c;
#[cfg(any(windows, test))]
mod transfer;
#[cfg(windows)]
#[allow(
    dead_code,
    reason = "The probe consumes only the research subset of the shared window protocol"
)]
#[path = "../../native-window/src/window_receipt.rs"]
mod window_receipt;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
fn main() {
    windows::main();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("AppContainer diagnostics require Windows; no isolation was applied.");
    std::process::exit(2);
}
