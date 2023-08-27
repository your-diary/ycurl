use clap::Parser;

#[derive(Parser)]
#[command(name = "ycurl")]
#[command(version)]
pub struct Args {
    /// Config file
    #[arg(short, long, default_value = "./ycurl.json")]
    pub file: String,

    /// Index or name of the request sent
    #[arg()]
    pub index: Option<String>,

    /// Show response headers
    #[arg(long)]
    pub show_headers: bool,

    /// Disable following redirects
    #[arg(long)]
    pub disable_redirect: bool,

    /// Allow `disabled` request to be sent
    #[arg(long)]
    pub include_disabled: bool,

    /// Show configurations after variable expansion and exit
    #[arg(long)]
    pub show_config: bool,

    /// Output shell completion code
    #[arg(long)]
    pub complete: bool,

    /// Verbose mode
    #[arg(short, long)]
    pub verbose: bool,
}
