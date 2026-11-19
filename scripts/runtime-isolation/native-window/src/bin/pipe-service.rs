use kraqel_window_probe::bootstrap::{Endpoint, Role};
fn main() {
    std::process::exit(
        Endpoint::open(Role::Data)
            .and_then(Endpoint::service)
            .unwrap_or(65),
    );
}
