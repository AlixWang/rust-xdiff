use clap::Parser;
use std::io::Write;
use xdiff::{
    cli::{Action, Args, RunArgs},
    DiffConfig, ExtraArgs,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let result = match args.action {
        Action::Run(args) => run(args).await,
    };

    match result {
        Ok(_) => (),
        Err(err) => println!("{:?}", err),
    }

    Ok(())
}

async fn run(args: RunArgs) -> anyhow::Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./xdiff.yaml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    config.validate()?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow::anyhow!(
            "Profile {} is not found in config file {}",
            args.profile,
            config_file
        )
    })?;

    let extra_args: ExtraArgs = args.extra_params.into();

    let output = profile.diff(extra_args).await?;

    let mut stdout = std::io::stdout();

    write!(stdout, "{}", output)?;

    Ok(())
}
