//! Bounded, synthetic AppContainer experiments, not a production launcher.
// P1 is a pure, test-only contract model. No CLI, transport or launcher integration.
#[cfg(any(windows, test))]
mod policy;
#[cfg(test)]
mod product_start;
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
