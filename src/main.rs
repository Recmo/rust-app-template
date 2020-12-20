#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

mod server;

pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use futures::prelude::*;
    pub use tokio::prelude::*;
    pub use tracing::{debug, error, info, trace, warn};
}

use crate::prelude::*;
use structopt::StructOpt;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, PartialEq, StructOpt)]
struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: usize,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, PartialEq, StructOpt)]
enum Command {
    /// Show version information
    Test,
}

pub fn main() -> Result<()> {
    // Parse CLI and handle help and version (which will stop the application).
    #[rustfmt::skip]
    let version = format!("\
        {version} {commit} ({commit_date})\n\
        {target} ({build_date})\n\
        {author}\n\
        {homepage}\n\
        {description}",
        version     = env!("CARGO_PKG_VERSION"),
        commit      = &env!("COMMIT_SHA")[..8],
        commit_date = env!("COMMIT_DATE"),
        author      = env!("CARGO_PKG_AUTHORS"),
        description = env!("CARGO_PKG_DESCRIPTION"),
        homepage    = env!("CARGO_PKG_HOMEPAGE"),
        target      = env!("TARGET"),
        build_date  = env!("BUILD_DATE"),
    );
    let matches = Options::clap().long_version(version.as_str()).get_matches();
    let options = Options::from_clap(&matches);

    // Initialize log output (prepend CLI verbosity to RUST_LOG)
    let log_cli = match options.verbose {
        0 => "info",
        1 => "rust_app_template=debug",
        2 => "rust_app_template=trace",
        3 => "rust_app_template=trace,debug",
        _ => "trace",
    };
    let log_filter = std::env::var("RUST_LOG").map_or_else(
        |_| log_cli.to_string(),
        |log_env| format!("{},{}", log_cli, log_env),
    );
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(log_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default log subscriber")?;
    tracing_log::LogTracer::init().context("adding log compatibility layer")?;

    // Log version
    info!(
        "{name} {version} {commit}",
        name = env!("CARGO_CRATE_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        commit = &env!("COMMIT_SHA")[..8],
    );

    // Launch Tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Error creating Tokio runtime")?
        .block_on(server::async_main())
        .context("Error in main thread")?;

    // Terminate successfully
    info!("program terminating normally");
    Ok(())
}

#[cfg(test)]
pub mod test {
    pub mod prelude {
        pub use float_eq::assert_float_eq;
        pub use pretty_assertions::{assert_eq, assert_ne};
        pub use proptest::prelude::*;
    }

    use super::*;
    use crate::test::prelude::{assert_eq, *};

    #[test]
    fn parse_args() {
        let cmd = "hello -vvv";
        let options = Options::from_iter_safe(cmd.split(' ')).unwrap();
        assert_eq!(options, Options {
            verbose: 3,
            command: None,
        });
    }

    #[test]
    fn add_commutative() {
        proptest!(|(a in 0.0..1.0, b in 0.0..1.0)| {
            let first: f64 = a + b;
            assert_float_eq!(first, b + a, ulps <= 0);
        })
    }
}

#[cfg(feature = "bench")]
pub mod bench {
    pub mod prelude {
        pub use criterion::{black_box, Criterion};
        pub use futures::executor::block_on;
    }

    use super::*;
    use crate::bench::prelude::*;

    #[cfg(feature = "bench")]
    pub fn main(c: &mut criterion::Criterion) {
        server::bench::group(c);
    }
}
