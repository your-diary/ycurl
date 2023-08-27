use std::error::Error;

use bat::PrettyPrinter;
use itertools::Itertools;
use reqwest::blocking::Response;
use serde::Serialize;
use serde_json::{Map, Value};

pub mod args;
pub mod client;
pub mod config;
pub mod logger;

//serializes `Value` with four-space indent
//ref: |https://stackoverflow.com/a/49087292/8776746|
fn to_string_pretty_four_space_indent(v: Value) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    v.serialize(&mut ser).unwrap();
    String::from_utf8(buf).unwrap()
}

fn bat(s: &str, language: Option<&str>) -> Result<(), Box<dyn Error>> {
    let mut printer = PrettyPrinter::new();
    printer
        .input_from_bytes(s.trim().as_bytes())
        .tab_width(Some(4))
        .true_color(false);
    if let Some(lang) = language {
        printer.language(lang);
    }
    printer.print()?;
    println!();
    Ok(())
}

pub fn show_config(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let config = serde_json::to_value(config)?;
    let s = to_string_pretty_four_space_indent(config);
    bat(&s, Some("json"))
}

pub fn show_complete(config: &config::Config) {
    let request_names = config.requests.iter().map(|e| &e.name).join(" ");
    let cli_options = "-f --file --show-headers --disable-redirect --complete -v --verbose";
    let words = format!("{} {}", request_names, cli_options);

    let command = format!(
        "complete -f -W '{}' -X '!@({}|*.json)' ycurl",
        words,
        words.replace(' ', "|")
    );
    println!("{}", command);
}

pub fn show_requests(config: &config::Config) -> Result<(), Box<dyn Error>> {
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
    bat(&s, Some("json"))?;
    Ok(())
}

pub fn pretty_print(
    res: Response,
    logger: &mut logger::Logger,
    config: &config::Config,
) -> Result<(), Box<dyn Error>> {
    if (res.status().is_success()) {
        println!("\u{001B}[032m{}\u{001B}[0m", res.status());
    } else {
        println!("\u{001B}[031m{}\u{001B}[0m", res.status());
    }
    if (config.cli_options.show_headers) {
        let mut m = Map::new();
        for (k, v) in res.headers() {
            m.insert(k.to_string(), Value::String(v.to_str()?.to_owned()));
        }
        let s = serde_json::to_string(&Value::from(m))?;
        println!();
        bat(&s, Some("json"))?;
    }

    logger.log("\n[response]\n")?;
    logger.log(&format!("{}", res.status()))?;
    logger.log(&format!("\n{:?}", res.headers()))?;

    let mut body = res.text()?;
    if (body.trim().is_empty()) {
        return Ok(());
    }

    let mut language = None;
    if let Ok(v) = serde_json::from_str::<Value>(&body) {
        body = to_string_pretty_four_space_indent(v);
        language = Some("json");
    } else if (body.starts_with('<')) {
        language = Some("html");
    }

    println!();
    bat(&body, language)?;

    logger.log(&format!("\n{}", body))?;

    Ok(())
}
