//! Bounded, synthetic AppContainer experiments, not a production launcher.
#[cfg(any(windows, test))]
mod policy;
#[cfg(any(windows, test))]
mod transfer;
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
