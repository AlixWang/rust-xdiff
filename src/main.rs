use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use std::io::Write;
use xdiff::{
    cli::{Action, Args, RunArgs},
    height_light_text, DiffConfig, DiffProfile, ExtraArgs, RequestProfile, ResponseProfile,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let result = match args.action {
        Action::Run(args) => run(args).await,
        Action::Parse => parse().await,
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

async fn parse() -> anyhow::Result<()> {
    let theme = &ColorfulTheme::default();
    let url1: String = Input::with_theme(theme).with_prompt("url1").interact()?;
    let url2: String = Input::with_theme(theme).with_prompt("url2").interact()?;

    let req1: RequestProfile = url1.parse()?;
    let req2: RequestProfile = url2.parse()?;

    let name = Input::with_theme(theme).with_prompt("Profile").interact()?;

    let res = req1.send(&ExtraArgs::default()).await?;

    let headers = res.get_headers_keys();

    let chosen = MultiSelect::with_theme(theme)
        .with_prompt("Select headers to skip")
        .items(&headers)
        .interact()?;

    let skip_headers = chosen.iter().map(|i| headers[*i].to_string()).collect();
    let res = ResponseProfile::new(skip_headers, vec![]);
    let profile = DiffProfile::new(req1, req2, res);
    let config = DiffConfig::new(vec![(name, profile)].into_iter().collect());
    let result = serde_yaml::to_string(&config)?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    // if atty::is(atty::Stream::Stdout) {
    // write!(stdout, "{}", highlight_text(&result, "yaml", None)?)?;
    // } else {
    write!(stdout, "{}", height_light_text(&result, "yaml")?)?;
    // }

    Ok(())
}
