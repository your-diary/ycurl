use std::error::Error;

use bat::PrettyPrinter;
use reqwest::blocking::Response;
use serde::Serialize;
use serde_json::{Map, Value};

pub mod args;
pub mod client;
pub mod config;
pub mod logger;

//serializes `Value` with four-space indent
//ref: |https://stackoverflow.com/a/49087292/8776746|
pub fn to_string_pretty_four_space_indent(v: Value) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    v.serialize(&mut ser).unwrap();
    String::from_utf8(buf).unwrap()
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
        PrettyPrinter::new()
            .input_from_bytes(s.as_bytes())
            .language("json")
            .tab_width(Some(4))
            .true_color(false)
            .print()?;
        println!();
    }

    logger.log("\n[response]\n")?;
    logger.log(&format!("{}", res.status()))?;
    logger.log(&format!("\n{:?}", res.headers()))?;

    let mut body = res.text()?;
    if (body.trim().is_empty()) {
        return Ok(());
    }
    println!();

    let mut printer = PrettyPrinter::new();
    if let Ok(v) = serde_json::from_str::<Value>(&body) {
        body = to_string_pretty_four_space_indent(v);
        printer.language("json");
    } else if (body.starts_with('<')) {
        printer.language("html");
    }

    printer
        .input_from_bytes(body.as_bytes())
        .tab_width(Some(4))
        .true_color(false)
        .print()
        .unwrap();
    println!();

    logger.log(&format!("\n{}", body))?;

    Ok(())
}
