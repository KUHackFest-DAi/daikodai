use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(name = "no_cap", version = "0.1.0", about = "What, you talkin' to me?")]
pub struct Args {
    #[arg(short, long, default_value = "Config.toml")]
    pub config: String,

    #[arg(short, long, default_value = ".env")]
    pub dotenv: String,

    #[arg(short, long, default_value = "log_config.yml")]
    pub log_config: String,
}
