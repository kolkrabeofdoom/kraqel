use kraqel_window_probe::bootstrap::{Endpoint, Role};
fn main() {
    if let Some(mode) = kraqel_window_probe::window_receipt::SceneOperation::from_argument(
        std::env::args_os().nth(1).as_deref(),
    ) {
        let code = std::env::args()
            .nth(2)
            .and_then(|profile| kraqel_window_probe::managed_scene::serve(&profile, mode).ok())
            .unwrap_or(65);
        std::process::exit(code as i32);
    }
    std::process::exit(
        Endpoint::open(Role::Data)
            .and_then(Endpoint::service)
            .unwrap_or(65),
    );
}
