use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempdir::TempDir;

/// Test default behavior w/ minimum possible input provided, e.g. version, release auto-fill
#[test]
fn test_basic_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-basic-defaults")?;
    let out_file = tmp_dir.path().join("test-1.0.0-1.noarch.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test")
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file).expect("couldn't find package");
    assert_eq!(pkg.metadata.get_name()?, "test");
    assert_eq!(pkg.metadata.get_epoch()?, 0);
    assert_eq!(pkg.metadata.get_version()?, "1.0.0");
    assert_eq!(pkg.metadata.get_release()?, "1");
    assert_eq!(pkg.metadata.get_arch()?, "noarch");
    assert_eq!(pkg.metadata.get_license()?, "MIT"); // the default (todo: maybe shouldn't have a default?)
    assert_eq!(pkg.metadata.get_summary()?, "");
    assert_eq!(pkg.metadata.get_description()?, ""); // should be a copy of the summary
    assert_eq!(
        pkg.metadata.get_payload_compressor()?,
        rpm::CompressionType::default()
    );

    // provides itself by default
    assert_eq!(
        pkg.metadata.get_provides()?,
        vec![
            rpm::Dependency::eq("test", "1.0.0"),
            rpm::Dependency::eq("test(noarch)", "1.0.0"),
        ]
    );
    // has no requires by default, except for rpmlib() ones
    assert_eq!(
        pkg.metadata
            .get_requires()?
            .into_iter()
            .filter(|r| !r.flags.contains(rpm::DependencyFlags::RPMLIB))
            .collect::<Vec<rpm::Dependency>>(),
        vec![]
    );
    // no other deps by default
    assert_eq!(pkg.metadata.get_obsoletes()?, vec![]);
    assert_eq!(pkg.metadata.get_conflicts()?, vec![]);
    assert_eq!(pkg.metadata.get_suggests()?, vec![]);
    assert_eq!(pkg.metadata.get_recommends()?, vec![]);
    assert_eq!(pkg.metadata.get_supplements()?, vec![]);
    assert_eq!(pkg.metadata.get_enhances()?, vec![]);
    // no filelists or changelog entries by default
    assert_eq!(pkg.metadata.get_file_entries()?, vec![]);
    assert_eq!(pkg.metadata.get_changelog_entries()?, vec![]);

    Ok(())
}

/// Assert that the command fails if no package name was provided
#[test]
fn test_no_name_provided() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-no-name-provided")?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .failure();

    Ok(())
}

/// Test adding basic metadata (version, epoch, release, arch, license, summary) to the package
#[test]
fn test_set_basic_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-set-metadata")?;
    let out_file = tmp_dir
        .path()
        .join("test-set-metadata-2.3.4-5.fc46.x86_64.rpm");

    assert!(!fs::exists(&out_file).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-set-metadata")
        .arg("--epoch")
        .arg("1")
        .arg("--version")
        .arg("2.3.4")
        .arg("--release")
        .arg("5.fc46")
        .arg("--arch")
        .arg("x86_64")
        .arg("--license")
        .arg("MPL-2.0")
        .arg("--summary")
        .arg("blah blah blah")
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success();
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    assert_eq!(pkg.metadata.get_name()?, "test-set-metadata");
    assert_eq!(pkg.metadata.get_epoch()?, 1);
    assert_eq!(pkg.metadata.get_version()?, "2.3.4");
    assert_eq!(pkg.metadata.get_release()?, "5.fc46");
    assert_eq!(pkg.metadata.get_arch()?, "x86_64");
    assert_eq!(pkg.metadata.get_license()?, "MPL-2.0");
    assert_eq!(pkg.metadata.get_summary()?, "blah blah blah");
    assert_eq!(pkg.metadata.get_description()?, "blah blah blah"); // should be a copy of the summary

    Ok(())
}

/// Test that the output option behaves as intended in various circumstances
#[test]
fn test_output_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("rpm-builder-test-outputs")?;

    // test using an explicit filename as output
    let explicit_filename = &tmp_dir.path().join("explicit-filename.rpm");
    assert!(!fs::exists(&explicit_filename).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-output")
        .arg("-o")
        .arg(&explicit_filename)
        .assert()
        .success();
    assert!(fs::exists(&explicit_filename).unwrap());

    let filename = Path::new("test-output-1.0.0-1.noarch.rpm");
    let file_in_tmp = &tmp_dir.path().join(&filename);

    // test using a directory as output, no provided filename
    assert!(!fs::exists(&file_in_tmp).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-output")
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .success();
    assert!(fs::exists(&file_in_tmp).unwrap());

    // test no output option provided at all - no provided filename, current working directory
    let orig_cwd = env::current_dir()?;
    env::set_current_dir(&tmp_dir.path())?;

    let expected_filename = Path::new("test-no-output-1.0.0-1.noarch.rpm");
    assert!(!fs::exists(&expected_filename).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-no-output")
        .assert()
        .success();
    assert!(fs::exists(&expected_filename).unwrap());

    env::set_current_dir(orig_cwd)?;

    Ok(())
}

/// Test providing the compression option
#[test]
fn test_package_compression() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = TempDir::new("rpm-builder-test-compression-option")?;

    for compression in ["gzip", "zstd", "none"] {
        let out_file = tmp_dir.path().join(format!(
            "test-compression-{}-1.0.0-1.noarch.rpm",
            &compression
        ));
        assert!(!fs::exists(&out_file).unwrap());
        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .arg(&format!("test-compression-{}", &compression))
            .arg("--exec-file")
            .arg(&format!(
                "{}/tests/assets/multiplication_tables.py:/usr/bin/multiplication_tables",
                workspace_path.to_string_lossy()
            ))
            .arg("--compression")
            .arg(&compression)
            .arg("-o")
            .arg(&out_file)
            .assert()
            .success();
        assert!(fs::exists(&out_file).unwrap());

        let pkg = rpm::Package::open(&out_file)?;
        assert_eq!(
            pkg.metadata.get_payload_compressor()?,
            match compression {
                "none" => rpm::CompressionType::None,
                "zstd" => rpm::CompressionType::Zstd,
                "gzip" => rpm::CompressionType::Gzip,
                _ => unreachable!(),
            }
        );
    }

    // Test an invalid value for the compression option
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-compression")
        .arg("--compression")
        .arg("invalid")
        .arg("-o")
        .arg(&tmp_dir.path())
        .assert()
        .failure();

    Ok(())
}

/// Test using the signing options
#[test]
fn test_signature() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = TempDir::new("rpm-builder-test-signature")?;
    let out_file = tmp_dir.path().join("test-signature-1.0.0-1.noarch.rpm");

    let private_key_path = workspace_path.join("tests/assets/package-manager.key");
    let public_key_path = workspace_path.join("tests/assets/package-manager.key.pub");

    assert!(!fs::exists(&out_file).unwrap());
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("test-signature")
        .arg("--sign-with-pgp-asc")
        .arg(&private_key_path)
        .arg("-o")
        .arg(&out_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    assert!(fs::exists(&out_file).unwrap());

    let pkg = rpm::Package::open(&out_file)?;
    let raw_public_key = fs::read(public_key_path)?;
    let verifier = rpm::signature::pgp::Verifier::load_from_asc_bytes(&raw_public_key)?;
    pkg.verify_signature(verifier)?;

    Ok(())
}
