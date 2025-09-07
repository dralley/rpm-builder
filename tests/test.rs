use std::env;
use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

#[test]
fn test_package_compression() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("rpm-builder-test-gzipped");
    fs::create_dir_all(&tmp_dir)?;
    let mut out_file = tmp_dir.clone();
    out_file.push("test.rpm");

    let work_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cargo_toml = work_dir.clone();
    cargo_toml.push("Cargo.toml");

    for compression in ["gzip", "zstd"] {
        let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .args(vec![
                "--exec-file",
                "tests/assets/multiplication_tables.py:/usr/bin/multiplication_tables",
                "--doc-file",
                &format!("{}:/foo/bar", &cargo_toml.to_string_lossy()),
                "--config-file",
                &format!("{}:/bar/bazz", &cargo_toml.to_string_lossy()),
                "--version",
                "1.0.0",
                "--dir",
                &format!("{}/tests/assets:/src", &work_dir.to_string_lossy()),
                "--compression",
                compression,
                "rpm-builder",
                "-o",
                &out_file.to_string_lossy(),
                "--release",
                "foo-bar",
                "--pre-install-script",
                &format!("{}/tests/assets/preinst.sh", &work_dir.to_string_lossy()),
            ])
            .assert()
            .success();
    }

    std::fs::remove_dir_all(tmp_dir)?;
    Ok(())
}

#[test]
fn test_not_compressed() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("rpm-builder-test-not-compressed");
    fs::create_dir_all(&tmp_dir)?;
    let mut out_file = tmp_dir.clone();
    out_file.push("test.rpm");

    let work_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("multiplication-tables")
        .arg("--exec-file")
        .arg(&format!(
            "{}/tests/assets/multiplication_tables.py:/usr/bin/multiplication_tables",
            work_dir.clone().to_string_lossy()
        ))
        .arg("--version")
        .arg("3.0.0")
        .arg("--epoch")
        .arg("5")
        .arg("--release")
        .arg("2")
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("failed to execute process");

    let pkg = rpm::Package::open(&out_file)?;
    assert_eq!(pkg.metadata.get_version()?, "3.0.0");
    assert_eq!(pkg.metadata.get_payload_compressor()?, rpm::CompressionType::None);

    std::fs::remove_dir_all(tmp_dir)?;
    Ok(())
}

#[test]
fn test_signature() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("rpm-builder-test-signature");
    fs::create_dir_all(&tmp_dir)?;
    let mut out_file = tmp_dir.clone();
    out_file.push("test.rpm");

    let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut private_key_path = workspace_path.clone();
    private_key_path.push("tests/assets/package-manager.key");
    let mut public_key_path = workspace_path.clone();
    public_key_path.push("tests/assets/package-manager.key.pub");

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(vec![
            "--exec-file",
            &format!(
                "{}/tests/assets/multiplication_tables.py:/usr/bin/rpm-builder",
                workspace_path.clone().to_string_lossy()
            ),
            "--version",
            "3.0.0",
            "--epoch",
            "5",
            "rpm-builder",
            "-o",
            &out_file.to_string_lossy(),
            "--sign-with-pgp-asc",
            &private_key_path.to_string_lossy(),
        ])
        .output()
        .expect("failed to execute process");

    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    let pkg = rpm::Package::open(&out_file)?;

    let raw_public_key = fs::read(public_key_path)?;
    let verifier = rpm::signature::pgp::Verifier::load_from_asc_bytes(&raw_public_key)?;
    pkg.verify_signature(verifier)?;

    std::fs::remove_dir_all(tmp_dir)?;
    Ok(())
}
