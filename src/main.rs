use chrono;
use clap;
use clap::Parser;
use clap_derive;
use clap_derive::{Parser, ValueEnum};
use regex::Regex;
use rpm;

use std::fs;
use std::path::{Path, PathBuf};

macro_rules! app_err {
    ($format:expr $(, $x:expr )*) => {
        AppError{ cause: format!($format,$( $x ),*)}
    };
}

struct AppError {
    cause: String,
}

impl AppError {
    fn new<T: Into<String>>(cause: T) -> Self {
        return AppError {
            cause: cause.into(),
        };
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::new(format!("{}", err))
    }
}

impl From<rpm::Error> for AppError {
    fn from(err: rpm::Error) -> AppError {
        AppError::new(format!("{}", err))
    }
}

pub const NAME_ARG: &str = "name";
pub const OUT_ARG: &str = "out";
pub const VERSION_ARG: &str = "version";
pub const EPOCH_ARG: &str = "epoch";
pub const LICENSE_ARG: &str = "license";
pub const ARCH_ARG: &str = "arch";
pub const RELEASE_ARG: &str = "release";
pub const DESC_ARG: &str = "desc";
pub const FILE_ARG: &str = "file";
pub const EXEC_FILE_ARG: &str = "exec-file";
pub const DOC_FILE_ARG: &str = "doc-file";
pub const CONFIG_FILE_ARG: &str = "config-file";
pub const DIR_ARG: &str = "dir";
pub const COMPRESSION_ARG: &str = "compression";
pub const CHANGELOG_ARG: &str = "changelog";
pub const REQUIRES_ARG: &str = "requires";
pub const OBSOLETES_ARG: &str = "obsoletes";
pub const PROVIDES_ARG: &str = "provides";
pub const CONFLICTS_ARG: &str = "conflicts";
pub const PRE_INSTALL_SCRIPTLET_ARG: &str = "pre-install-script";
pub const POST_INSTALL_SCRIPTLET_ARG: &str = "post-install-script";
pub const PRE_UNINSTALL_SCRIPTLET_ARG: &str = "pre-uninstall-script";
pub const POST_UNINSTALL_SCRIPTLET_ARG: &str = "post-uninstall-script";
pub const SIGN_WITH_PGP_ASC_ARG: &str = "sign-with-pgp-asc";

#[derive(Parser, Debug)]
#[command(name = "rpm-builder", about = "Build RPMs with ease")]
pub struct Cli {
    #[arg(short = 'o', long, value_name = "OUT", help = "Specify an out file")]
    pub out: Option<PathBuf>,

    #[arg(help = "Specify the name of your package")]
    pub name: String,

    #[arg(
        long,
        value_name = "EPOCH",
        default_value = "0",
        help = "Specify an epoch"
    )]
    pub epoch: u32,

    #[arg(
        long,
        value_name = "VERSION",
        default_value = "1.0.0",
        help = "Specify a version"
    )]
    pub version: String,

    #[arg(
        long,
        value_name = "RELEASE",
        default_value = "1",
        help = "Specify release number of the package"
    )]
    pub release: String,

    #[arg(
        long,
        value_name = "ARCH",
        default_value = "noarch",
        help = "Specify the target architecture"
    )]
    pub arch: String,

    #[arg(
        long,
        value_name = "LICENSE",
        default_value = "MIT",
        help = "Specify a license"
    )]
    pub license: String,

    #[arg(
        long,
        value_name = "SUMMARY",
        default_value = "",
        help = "Give a simple description of the package"
    )]
    pub summary: String,

    #[arg(long, value_name = "FILE", help = "Add a regular file to the rpm")]
    pub file: Vec<String>,

    #[arg(
        long,
        value_name = "EXEC_FILE",
        help = "Add an executable file to the rpm"
    )]
    pub exec_file: Vec<String>,

    #[arg(
        long,
        value_name = "DOC_FILE",
        help = "Add a documentation file to the rpm"
    )]
    pub doc_file: Vec<String>,

    #[arg(
        long,
        value_name = "CONFIG_FILE",
        help = "Add a config file to the rpm"
    )]
    pub config_file: Vec<String>,

    #[arg(
        long,
        value_name = "DIR",
        help = "Add a directory and all its files to the rpm"
    )]
    pub dir: Vec<String>,

    #[arg(
        long,
        value_name = "COMPRESSION",
        value_enum,
        help = "Specify the compression algorithm."
    )]
    pub compression: Option<Compression>,

    #[arg(
        long,
        value_name = "CHANGELOG_ENTRY",
        help = "Add a changelog entry to the rpm. The entry has the form <author>:<content>:<yyyy-mm-dd> (time is in UTC)"
    )]
    pub changelog: Vec<String>,

    #[arg(
        long,
        value_name = "REQUIRES",
        help = "Indicates that the rpm requires another package. Use the format '<name> [>|>=|=|<=|< version]'"
    )]
    pub requires: Vec<String>,

    #[arg(
        long,
        value_name = "PROVIDES",
        help = "Indicates that the rpm provides another package. Use the format '<name> [>|>=|=|<=|< version]'"
    )]
    pub provides: Vec<String>,

    #[arg(
        long,
        value_name = "OBSOLETES",
        help = "Indicates that the rpm obsoletes another package. Use the format '<name> [>|>=|=|<=|< version]'"
    )]
    pub obsoletes: Vec<String>,

    #[arg(
        long,
        value_name = "CONFLICTS",
        help = "Indicates that the rpm conflicts with another package. Use the format '<name> [>|>=|=|<=|< version]'"
    )]
    pub conflicts: Vec<String>,

    #[arg(
        long,
        value_name = "PREINSTALLSCRIPT",
        help = "Path to a file that contains the pre-installation script"
    )]
    pub pre_install_script: Option<PathBuf>,

    #[arg(
        long,
        value_name = "POSTINSTALLSCRIPT",
        help = "Path to a file that contains the post-installation script"
    )]
    pub post_install_script: Option<PathBuf>,

    #[arg(
        long,
        value_name = "PRE_UNINSTALL_SCRIPT",
        help = "Path to a file that contains a pre-uninstall script"
    )]
    pub pre_uninstall_script: Option<PathBuf>,

    #[arg(
        long,
        value_name = "POST_UNINSTALL_SCRIPT",
        help = "Path to a file that contains a post-uninstall script"
    )]
    pub post_uninstall_script: Option<PathBuf>,

    #[arg(
        long,
        value_name = "SIGN_WITH_PGP_ASC",
        help = "Sign this package with the specified PGP secret key"
    )]
    pub sign_with_pgp_asc: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Compression {
    Gzip,
    Zstd,
    None,
}

fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    let compression = match args.compression {
        Some(Compression::Gzip) => rpm::CompressionType::Gzip,
        Some(Compression::Zstd) => rpm::CompressionType::Zstd,
        Some(Compression::None) => rpm::CompressionType::None,
        _ => rpm::CompressionType::default(),
    };

    let config = rpm::BuildConfig::default().compression(compression);
    let mut builder = rpm::PackageBuilder::new(
        &args.name,
        &args.version,
        &args.license,
        &args.arch,
        &args.summary,
    )
    .using_config(config)
    .release(args.release)
    .epoch(args.epoch);

    for (src, options) in parse_file_options(&args.file)? {
        builder = builder
            .with_file(src, options)
            .map_err(|e| app_err!("error adding regular file {}: {}", src, e))?;
    }

    for (src, options) in parse_file_options(&args.exec_file)? {
        builder = builder
            .with_file(src, options.mode(0o100755))
            .map_err(|e| app_err!("error adding executable file {}: {}", src, e))?;
    }

    for (src, options) in parse_file_options(&args.config_file)? {
        builder = builder
            .with_file(src, options.is_config())
            .map_err(|e| app_err!("error adding config file {}: {}", src, e))?;
    }

    for dir in args.dir {
        let parts: Vec<&str> = dir.split(":").collect();
        if parts.len() != 2 {
            return Err(app_err!(
                "invalid file argument:{} it needs to be of the form <source-path>:<dest-path>",
                dir
            ));
        }
        let dir = parts[0];
        let target = PathBuf::from(parts[1]);
        builder = add_dir(dir, &target, builder)
            .map_err(|e| app_err!("error adding dir {}: {}", dir, e))?;
    }

    for (src, options) in parse_file_options(&args.doc_file)? {
        builder = builder
            .with_file(src, options.is_doc())
            .map_err(|e| app_err!("error adding doc file {}: {}", src, e))?;
    }

    if let Some(scriptlet_path) = args.pre_install_script {
        let content = fs::read_to_string(&scriptlet_path).map_err(|e| {
            app_err!(
                "error reading {} {:?}: {}",
                PRE_INSTALL_SCRIPTLET_ARG,
                scriptlet_path,
                e
            )
        })?;
        builder = builder.pre_install_script(content);
    }

    if let Some(scriptlet_path) = args.post_install_script {
        let content = fs::read_to_string(&scriptlet_path).map_err(|e| {
            app_err!(
                "error reading {} {:?}: {}",
                POST_INSTALL_SCRIPTLET_ARG,
                scriptlet_path,
                e
            )
        })?;
        builder = builder.post_install_script(content);
    }

    if let Some(scriptlet_path) = args.pre_uninstall_script {
        let content = fs::read_to_string(&scriptlet_path).map_err(|e| {
            app_err!(
                "error reading {} {:?}: {}",
                PRE_UNINSTALL_SCRIPTLET_ARG,
                scriptlet_path,
                e
            )
        })?;
        builder = builder.pre_uninstall_script(content);
    }

    if let Some(scriptlet_path) = args.post_uninstall_script {
        let content = fs::read_to_string(&scriptlet_path).map_err(|e| {
            app_err!(
                "error reading {} {:?}: {}",
                POST_UNINSTALL_SCRIPTLET_ARG,
                scriptlet_path,
                e
            )
        })?;
        builder = builder.post_uninstall_script(content);
    }

    for raw_entry in args.changelog {
        let parts: Vec<&str> = raw_entry.split(":").collect();
        if parts.len() != 3 {
            return Err(app_err!(
                "invalid file argument:{} it needs to be of the form <author>:<content>:<yyyy-mm-dd>",
                &raw_entry
            ));
        }
        let name = parts[0];
        let content = parts[1];
        let raw_time = parts[2];
        let parse_result = chrono::NaiveDate::parse_from_str(raw_time, "%Y-%m-%d");
        if parse_result.is_err() {
            return Err(app_err!(
                "error while parsing date time: {}",
                parse_result.err().unwrap()
            ));
        }
        let seconds = parse_result
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        builder = builder.add_changelog_entry(name, content, rpm::Timestamp::from(seconds as u32));
    }

    for item in args.requires {
        let dependency = parse_dependency(&item)?;
        builder = builder.requires(dependency);
    }

    for item in args.obsoletes {
        let dependency = parse_dependency(&item)?;
        builder = builder.obsoletes(dependency);
    }

    for item in args.conflicts {
        let dependency = parse_dependency(&item)?;
        builder = builder.conflicts(dependency);
    }

    for item in args.provides {
        let dependency = parse_dependency(&item)?;
        builder = builder.provides(dependency);
    }

    let pkg = if let Some(signing_key_path) = args.sign_with_pgp_asc {
        let raw_key = fs::read(&signing_key_path).map_err(|e| {
            app_err!(
                "unable to load private key file from path {:?}: {}",
                signing_key_path,
                e
            )
        })?;

        let signer = rpm::signature::pgp::Signer::load_from_asc_bytes(&raw_key).map_err(|e| {
            app_err!(
                "unable to create signer from private key {:?}: {}",
                signing_key_path,
                e
            )
        })?;
        builder.build_and_sign(signer)?
    } else {
        builder.build()?
    };

    let output_path = args
        .out
        .unwrap_or_else(|| format!("./{}.rpm", pkg.metadata.get_nevra().unwrap().nvra()).into());

    let mut out_file = fs::File::create(&output_path)
        .map_err(|e| app_err!("unable to create output file {:?}: {}", &output_path, e))?;
    pkg.write(&mut out_file)
        .map_err(|e| app_err!("unable to write package to path {:?}: {}", &output_path, e))?;
    Ok(())
}

fn add_dir<P: AsRef<Path>>(
    full_path: P,
    target_path: &PathBuf,
    mut builder: rpm::PackageBuilder,
) -> Result<rpm::PackageBuilder, AppError> {
    for entry in std::fs::read_dir(full_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let mut new_target = target_path.clone();

        let source = if metadata.file_type().is_symlink() {
            std::fs::read_link(entry.path().as_path())?
        } else {
            entry.path()
        };

        let file_name = source
            .file_name()
            .ok_or_else(|| app_err!("path does not have filename"))?;
        new_target.push(file_name);

        builder = if metadata.file_type().is_dir() {
            add_dir(&source, &new_target, builder)?
        } else {
            builder.with_file(&source, rpm::FileOptions::new(new_target.to_string_lossy()))?
        }
    }
    Ok(builder)
}

fn parse_file_options(
    raw_files: &Vec<String>,
) -> Result<Vec<(&str, rpm::FileOptionsBuilder)>, AppError> {
    raw_files
        .iter()
        .map(|input| {
            let parts: Vec<&str> = input.split(":").collect();
            if parts.len() != 2 {
                return Err(app_err!(
                    "invalid file argument:{} it needs to be of the form <source-path>:<dest-path>",
                    input
                ));
            }
            Ok((parts[0], rpm::FileOptions::new(parts[1])))
        })
        .collect()
}

fn parse_dependency(line: &str) -> Result<rpm::Dependency, AppError> {
    let re = Regex::new(r"^([a-zA-Z0-9\-\._]+)(\s*(>=|>|=|<=|<)(.+))?$").unwrap();

    let parts = re
        .captures(line)
        .ok_or(app_err!("invalid pattern in dependency block {}", line))?;
    let parts: Vec<String> = parts
        .iter()
        .filter(|c| c.is_some())
        .map(|c| String::from(c.unwrap().as_str()))
        .collect();

    if parts.len() <= 2 {
        Ok(rpm::Dependency::any(&parts[1]))
    } else {
        let dep = match parts[3].as_str() {
            "=" => rpm::Dependency::eq(&parts[1], &parts[4]),
            "<" => rpm::Dependency::less(&parts[1], &parts[4]),
            "<=" => rpm::Dependency::less_eq(&parts[1], &parts[4]),
            ">=" => rpm::Dependency::greater_eq(&parts[1], &parts[4]),
            ">" => rpm::Dependency::greater(&parts[1], &parts[4]),
            _ => {
                return Err(app_err!(
                    "regex is invalid here, got unknown match {}",
                    &parts[3]
                ));
            }
        };
        Ok(dep)
    }
}
