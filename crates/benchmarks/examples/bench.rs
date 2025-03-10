fn main() -> Result<(), lexopt::Error> {
    let args = Args::parse()?;

    match args.parser {
        Parser::Document => {
            let _doc = CARGO_MANIFEST.parse::<toml_edit::DocumentMut>().unwrap();
            #[cfg(debug_assertions)] // Don't interefere with profiling
            drop(_doc);
        }
        Parser::De => {
            let _doc = toml::from_str::<manifest::Manifest>(CARGO_MANIFEST).unwrap();
            #[cfg(debug_assertions)] // Don't interefere with profiling
            drop(_doc);
        }
        Parser::Table => {
            let _doc = CARGO_MANIFEST.parse::<toml::Table>().unwrap();
            #[cfg(debug_assertions)] // Don't interefere with profiling
            drop(_doc);
        }
    }
    Ok(())
}

struct Args {
    parser: Parser,
}

impl Args {
    fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut parser = Parser::Document;

        let mut args = lexopt::Parser::from_env();
        while let Some(arg) = args.next()? {
            match arg {
                Long("parser") => {
                    let value = args.value()?;
                    parser = match &value.to_str() {
                        Some("document") => Parser::Document,
                        Some("de") => Parser::De,
                        Some("table") => Parser::Table,
                        _ => {
                            return Err(lexopt::Error::UnexpectedValue {
                                option: "parser".to_owned(),
                                value: value.clone(),
                            });
                        }
                    };
                }
                _ => return Err(arg.unexpected()),
            }
        }

        Ok(Self { parser })
    }
}

enum Parser {
    Document,
    De,
    Table,
}

mod manifest {
    use std::collections::HashMap;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct Manifest {
        package: Package,
        #[serde(default)]
        lib: Option<Lib>,
        #[serde(default)]
        bin: Vec<Bin>,
        #[serde(default)]
        features: HashMap<String, Vec<String>>,
        #[serde(default)]
        dependencies: HashMap<String, Dependency>,
        #[serde(default)]
        build_dependencies: HashMap<String, Dependency>,
        #[serde(default)]
        dev_dependencies: HashMap<String, Dependency>,
        #[serde(default)]
        target: HashMap<String, Target>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct Package {
        name: String,
        version: String,
        #[serde(default)]
        edition: Option<String>,
        #[serde(default)]
        authors: Vec<String>,
        #[serde(default)]
        license: Option<String>,
        #[serde(default)]
        homepage: Option<String>,
        #[serde(default)]
        repository: Option<String>,
        #[serde(default)]
        documentation: Option<String>,
        #[serde(default)]
        readme: Option<String>,
        #[serde(default)]
        description: Option<String>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct Lib {
        name: String,
        #[serde(default)]
        path: Option<String>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct Bin {
        name: String,
        #[serde(default)]
        test: bool,
        #[serde(default)]
        doc: bool,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    #[serde(untagged)]
    pub(crate) enum Dependency {
        Version(String),
        Full(DependencyFull),
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct DependencyFull {
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        default_features: bool,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        features: Vec<String>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) struct Target {
        #[serde(default)]
        dependencies: HashMap<String, Dependency>,
        #[serde(default)]
        build_dependencies: HashMap<String, Dependency>,
        #[serde(default)]
        dev_dependencies: HashMap<String, Dependency>,
    }
}

const CARGO_MANIFEST: &str = r#"
[package]
name = "cargo"
version = "0.57.0"
edition = "2018"
authors = ["Yehuda Katz <wycats@gmail.com>",
           "Carl Lerche <me@carllerche.com>",
           "Alex Crichton <alex@alexcrichton.com>"]
license = "MIT OR Apache-2.0"
homepage = "https://crates.io"
repository = "https://github.com/rust-lang/cargo"
documentation = "https://docs.rs/cargo"
readme = "README.md"
description = """
Cargo, a package manager for Rust.
"""

[lib]
name = "cargo"
path = "src/cargo/lib.rs"

[dependencies]
atty = "0.2"
bytesize = "1.0"
cargo-platform = { path = "crates/cargo-platform", version = "0.1.2" }
cargo-util = { path = "crates/cargo-util", version = "0.1.1" }
crates-io = { path = "crates/crates-io", version = "0.33.0" }
crossbeam-utils = "0.8"
curl = { version = "0.4.38", features = ["http2"] }
curl-sys = "0.4.45"
env_logger = "0.9.0"
pretty_env_logger = { version = "0.4", optional = true }
anyhow = "1.0"
filetime = "0.2.9"
flate2 = { version = "1.0.3", default-features = false, features = ["zlib"] }
git2 = "0.13.16"
git2-curl = "0.14.1"
glob = "0.3.0"
hex = "0.4"
home = "0.5"
humantime = "2.0.0"
ignore = "0.4.7"
lazy_static = "1.2.0"
jobserver = "0.1.24"
lazycell = "1.2.0"
libc = "0.2"
log = "0.4.6"
libgit2-sys = "0.12.18"
memchr = "2.1.3"
num_cpus = "1.0"
opener = "0.5"
percent-encoding = "2.0"
rustfix = "0.6.0"
semver = { version = "1.0.3", features = ["serde"] }
serde = { version = "1.0.123", features = ["derive"] }
serde_ignored = "0.1.0"
serde_json = { version = "1.0.30", features = ["raw_value"] }
shell-escape = "0.1.4"
strip-ansi-escapes = "0.1.0"
tar = { version = "0.4.35", default-features = false }
tempfile = "3.0"
termcolor = "1.1"
toml = "0.5.7"
unicode-xid = "0.2.0"
url = "2.2.2"
walkdir = "2.2"
clap = "2.31.2"
unicode-width = "0.1.5"
openssl = { version = '0.10.11', optional = true }
im-rc = "15.0.0"
itertools = "0.10.0"

# A noop dependency that changes in the Rust repository, it's a bit of a hack.
# See the `src/tools/rustc-workspace-hack/README.md` file in `rust-lang/rust`
# for more information.
rustc-workspace-hack = "1.0.0"

[target.'cfg(windows)'.dependencies]
fwdansi = "1.1.0"

[target.'cfg(windows)'.dependencies.winapi]
version = "0.3"
features = [
  "basetsd",
  "handleapi",
  "jobapi",
  "jobapi2",
  "memoryapi",
  "minwindef",
  "ntdef",
  "ntstatus",
  "processenv",
  "processthreadsapi",
  "psapi",
  "synchapi",
  "winerror",
  "winbase",
  "wincon",
  "winnt",
]

[dev-dependencies]
cargo-test-macro = { path = "crates/cargo-test-macro" }
cargo-test-support = { path = "crates/cargo-test-support" }

[build-dependencies]
flate2 = { version = "1.0.3", default-features = false, features = ["zlib"] }
tar = { version = "0.4.26", default-features = false }

[[bin]]
name = "cargo"
test = false
doc = false

[features]
deny-warnings = []
vendored-openssl = ["openssl/vendored"]
pretty-env-logger = ["pretty_env_logger"]
"#;
