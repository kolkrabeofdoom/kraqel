use kraqel_window_probe::bootstrap::{Endpoint, Role};
fn main() {
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new(
            kraqel_window_probe::window_receipt::SCENE_ARG,
        ))
    {
        let ok = std::env::args()
            .nth(2)
            .is_some_and(|profile| kraqel_window_probe::managed_scene::serve(&profile).is_ok());
        std::process::exit(if ok { 0 } else { 65 });
    }
    std::process::exit(
        Endpoint::open(Role::Data)
            .and_then(Endpoint::service)
            .unwrap_or(65),
    );
}
