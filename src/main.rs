mod config;
mod io;

use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueHint};

use self::config::Config;
use self::io::{ValuesFormat, expand_path, parse_values};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Read values to populate `template` from a file insted of STDIN.
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    /// Format of the values read from stdin or from `file`
    #[arg(long, value_enum, default_value_t, value_name = "FORMAT")]
    pub format: ValuesFormat,

    /// Path to the configuration file that should be used (defaults to "$XDG_CONFIG_HOME/targets/config.toml")
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    let config: Config = cli_args.config.clone().try_into()?;
    let values = parse_values(&cli_args)?;

    for (_name, target) in config.targets.into_iter() {
        let template_path = expand_path(&target.template)?;
        let template = mustache::compile_path(&template_path)
            .map_err(|e| anyhow!("{e:?}"))
            .with_context(|| format!("failed to parse template from: {template_path:?}"))?;
        let out_path = expand_path(&target.out)?;
        std::fs::create_dir_all(
            out_path
                .parent()
                .with_context(|| format!("output path has no parent directory: {out_path:?}"))?,
        )
        .with_context(|| {
            format!("failed to create parent directory for output path: {out_path:?}")
        })?;
        let mut file = File::create(&out_path)
            .with_context(|| format!("failed to create file at output path: {out_path:?}"))?;
        template.render_data(&mut file, &values)?;
        file.sync_all()?;
        target.run_hook()?;
    }

    Ok(())
}
