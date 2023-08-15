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

    /// Verbose mode
    #[arg(short, long)]
    pub verbose: bool,
}
