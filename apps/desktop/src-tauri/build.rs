use std::{env, path::PathBuf};

const LICENSE_ENV_VARS: [&str; 6] = [
    "MULVERSE_LICENSE_API_BASE_URL",
    "MULVERSE_LICENSE_PRODUCT_CODE",
    "MULVERSE_LICENSE_PRODUCT_ID",
    "MULVERSE_LICENSE_KEY_PREFIX",
    "MULVERSE_LICENSE_PUBLIC_KEY_ID",
    "MULVERSE_LICENSE_PUBLIC_KEY_B64",
];

fn main() {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo should set CARGO_MANIFEST_DIR"),
    );
    let workspace_env = manifest_dir.join("../../../.env");
    println!("cargo:rerun-if-changed={}", workspace_env.display());

    if workspace_env.is_file() {
        dotenvy::from_path(&workspace_env).unwrap_or_else(|error| {
            panic!(
                "Failed to load workspace .env at {}: {error}",
                workspace_env.display()
            )
        });
    }

    for name in LICENSE_ENV_VARS {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(value) = env::var(name) {
            println!("cargo:rustc-env={name}={value}");
        }
    }

    tauri_build::build()
}
