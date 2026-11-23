use kraqel_window_probe::bootstrap::{Endpoint, Role};
#[used]
#[unsafe(link_section = ".kqcmp")]
static COMPARE_ABI: [u8; 32] = kraqel_window_probe::window_receipt::COMPARE_ABI;

#[used]
#[unsafe(link_section = ".kqmerge")]
static MERGE_ABI: [u8; 32] = kraqel_window_probe::window_receipt::MERGE_ABI;
#[used]
#[unsafe(link_section = ".kqfresh")]
static REFRESH_ABI: [u8; 32] = kraqel_window_probe::window_receipt::REFRESH_ABI;
#[used]
#[unsafe(link_section = ".kqgap")]
static GAP_ABI: [u8; 32] = kraqel_window_probe::window_receipt::GAP_ABI;
#[used]
#[unsafe(link_section = ".kqagain")]
static REPEAT_ABI: [u8; 32] = kraqel_window_probe::window_receipt::REPEAT_ABI;
fn main() {
    std::hint::black_box(&REPEAT_ABI);
    std::hint::black_box(&GAP_ABI);
    std::hint::black_box(&REFRESH_ABI);
    std::hint::black_box(&MERGE_ABI);
    let mut arg = std::env::args_os().nth(1);
    use kraqel_native_scene_model::channel::ReviewedWrite;
    let repeat_arg = arg
        .as_deref()
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_suffix("-repeat"))
        .map(std::ffi::OsString::from);
    let repeat = repeat_arg.is_some();
    if repeat {
        arg = repeat_arg;
    }
    let gap_arg = arg
        .as_deref()
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_suffix("-gap"))
        .map(std::ffi::OsString::from);
    let gap = gap_arg.is_some();
    if gap {
        arg = gap_arg;
    }

    let reviewed_write = if gap && repeat {
        ReviewedWrite::Repeat
    } else if gap {
        ReviewedWrite::Once
    } else {
        ReviewedWrite::Unavailable
    };
    let stripped = arg
        .as_deref()
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_suffix("-merge"));

    std::hint::black_box(&COMPARE_ABI);
    if let Some(mode) = kraqel_window_probe::window_receipt::SceneOperation::from_argument(
        stripped.map(std::ffi::OsStr::new).or(arg.as_deref()),
    ) {
        let code = std::env::args()
            .nth(2)
            .and_then(|profile| {
                kraqel_window_probe::managed_scene::serve(
                    &profile,
                    mode,
                    stripped.is_some(),
                    reviewed_write,
                )
                .ok()
            })
            .unwrap_or(65);
        std::process::exit(code as i32);
    }
    std::process::exit(
        Endpoint::open(Role::Data)
            .and_then(Endpoint::service)
            .unwrap_or(65),
    );
}
