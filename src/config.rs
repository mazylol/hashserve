use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Configuration {
    /// Set the servers port
    #[arg(long)]
    #[clap(default_value = "3000")]
    pub port: u16,

    /// Enable persistance
    #[arg(long)]
    #[clap(default_value = "false")]
    pub persist: bool,

    /// Set the servers password
    #[arg(long, short)]
    pub password: String,
}
