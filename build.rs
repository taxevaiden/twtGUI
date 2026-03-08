fn main() {
    let now = chrono::Utc::now();
    let ver = now.format("%Y.%-m%d").to_string();
    let date = now
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        .to_string();

    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    let hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=BUILD_VERSION={}-{}", ver, hash.trim());
    println!("cargo:rustc-env=BUILD_DATE={}", date);
    println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
}
