use std::{env, path::PathBuf, process::Command};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../scripts/runtime-isolation/native-window");
        for entry in [
            "src",
            "Cargo.toml",
            "Cargo.lock",
            "../native-scene-model/src",
            "../native-scene-model/Cargo.toml",
        ] {
            println!("cargo:rerun-if-changed={}", manifest.join(entry).display());
        }
        let target = env::var("TARGET").expect("Cargo target");
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output"));
        let build = out
            .ancestors()
            .nth(4)
            .expect("Cargo target directory")
            .join("manuscript-review-roles");
        let mut cargo = Command::new(env::var_os("CARGO").expect("Cargo"));
        // This nested build produces executable resources, including when the parent is linted.
        if env::var_os("RUSTC_WORKSPACE_WRAPPER").is_some_and(|wrapper| {
            PathBuf::from(wrapper)
                .file_stem()
                .is_some_and(|name| name == "clippy-driver")
        }) {
            cargo.env_remove("RUSTC_WORKSPACE_WRAPPER");
        }
        let status = cargo
            .arg("build")
            .args([
                "--release",
                "--locked",
                "--features",
                "manuscript-window",
                "--bin",
                "native-leaf",
                "--bin",
                "pipe-service",
                "--target",
                &target,
                "--manifest-path",
            ])
            .arg(manifest.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(&build)
            .status()
            .expect("build isolated review roles");
        assert!(status.success(), "isolated review roles failed to build");
        println!(
            "cargo:rustc-env=KRAQEL_REVIEW_IMAGES={}",
            build.join(target).join("release").display()
        );
    }
    tauri_build::build();
}
