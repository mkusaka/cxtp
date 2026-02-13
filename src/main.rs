use clap::Parser;
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cxtp",
    version,
    about = "Register a directory as trusted/untrusted in Codex project settings"
)]
struct Cli {
    #[arg(value_name = "DIRECTORY", default_value = ".")]
    directory: PathBuf,

    #[arg(long, value_enum, default_value_t = TrustLevelArg::Trusted)]
    trust_level: TrustLevelArg,

    #[arg(long, value_name = "PATH")]
    codex_home: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TrustLevelArg {
    Trusted,
    Untrusted,
}

impl From<TrustLevelArg> for cxtp::TrustLevel {
    fn from(value: TrustLevelArg) -> Self {
        match value {
            TrustLevelArg::Trusted => cxtp::TrustLevel::Trusted,
            TrustLevelArg::Untrusted => cxtp::TrustLevel::Untrusted,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let result = cxtp::set_project_trust(
        &cli.directory,
        cli.codex_home.as_deref(),
        cli.trust_level.into(),
    )?;

    if result.changed {
        println!(
            "updated: {} -> {} in {}",
            result.project_path.display(),
            result.trust_level,
            result.config_path.display()
        );
    } else {
        println!(
            "no changes: {} is already {} in {}",
            result.project_path.display(),
            result.trust_level,
            result.config_path.display()
        );
    }

    Ok(())
}
