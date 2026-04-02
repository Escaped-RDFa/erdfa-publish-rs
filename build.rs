fn main() {
    // Capture git commit at build time
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    if let Ok(o) = output {
        let commit = String::from_utf8_lossy(&o.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_COMMIT={}", commit);
    }
    println!("cargo:rerun-if-changed=.git/HEAD");
}
