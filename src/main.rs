use std::error::Error;

use bat::PrettyPrinter;
use chrono::Local;
use clap::Parser;
use itertools::Itertools;

use ycurl::args;
use ycurl::client::Client;
use ycurl::config;
use ycurl::logger::Logger;

#[allow(unused_macros)]
macro_rules! o {
    (---)             => { println!("\u{001B}[090m{}\u{001B}[0m", "―".repeat(10)) };
    ($expr:literal)   => { println!("{}", $expr) };
    ($expr:expr)      => { println!("{} = {:#?}", stringify!($expr), $expr) };
    ($($expr:expr),*) => { println!($($expr),*) };
}

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
        let config = serde_json::to_value(config)?;
        let s = ycurl::to_string_pretty_four_space_indent(config);
        PrettyPrinter::new()
            .input_from_bytes(s.as_bytes())
            .language("json")
            .tab_width(Some(4))
            .true_color(false)
            .print()?;
        return Ok(());
    }

    if (args.complete) {
        let request_names = config.requests.into_iter().map(|e| e.name).join(" ");
        let cli_options = "-f --file --show-headers --disable-redirect --complete -v --verbose";
        let words = format!("{} {}", request_names, cli_options);

        let command = format!(
            "complete -f -W '{}' -X '!@({}|*.json)' ycurl",
            words,
            words.replace(' ', "|")
        );
        println!("{}", command);

        return Ok(());
    }

    if (args.index.is_none()) {
        let mut l = vec![];
        for i in 0..config.requests.len() {
            if (config.requests[i].disabled) {
                continue;
            }
            l.push(format!(
                r#"{{"index": {}, "name": "{}", "url": "{}"}}"#,
                i, config.requests[i].name, config.requests[i].url
            ));
        }
        let s = l.iter().join("\n");
        PrettyPrinter::new()
            .input_from_bytes(s.as_bytes())
            .language("json")
            .tab_width(Some(4))
            .true_color(false)
            .print()?;
        println!();
        return Ok(());
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
