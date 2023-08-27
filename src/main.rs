use std::error::Error;

use chrono::Local;
use clap::Parser;

use ycurl::args;
use ycurl::client::Client;
use ycurl::config;
use ycurl::logger::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::Args::parse();

    let mut config = config::Config::new(&args.file)?;
    if (args.show_headers) {
        config.cli_options.show_headers = true;
    }
    if (args.disable_redirect) {
        config.cli_options.disable_redirect = true;
    }
    if (args.verbose) {
        config.cli_options.verbose = true;
    }

    if (args.show_config) {
        return ycurl::show_config(&config);
    }

    if (args.complete) {
        ycurl::show_complete(&config);
        return Ok(());
    }

    if (args.index.is_none()) {
        return ycurl::show_requests(&config);
    }

    let request = if let Ok(i) = args.index.as_ref().unwrap().parse::<usize>() {
        if (i >= config.requests.len()) {
            return Err("index out of bounds".into());
        }
        &config.requests[i]
    } else {
        #[allow(clippy::collapsible_else_if)]
        if let Some(r) = config
            .requests
            .iter()
            .find(|r| &r.name == args.index.as_ref().unwrap())
        {
            r
        } else {
            return Err("no entry found for the name".into());
        }
    };

    if (request.disabled && !args.include_disabled) {
        return Err("disabled request".into());
    }

    let mut logger = Logger::new()?;
    logger.log(&format!(
        "\n-------------------- {} --------------------",
        Local::now().format("%Y/%m/%d(%a)%H:%M:%S")
    ))?;

    let client = Client::new(&config, request, &mut logger)?;
    let res = client.send()?;

    ycurl::pretty_print(res, &mut logger, &config)?;

    Ok(())
}
