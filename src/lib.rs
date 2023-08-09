use std::error::Error;

use bat::PrettyPrinter;
use reqwest::blocking::Response;
use serde::Serialize;
use serde_json::Value;

pub mod args;
pub mod client;
pub mod config;

pub fn pretty_print(res: Response) -> Result<(), Box<dyn Error>> {
    println!("{}", res.status());

    let mut body = res.text()?;
    if (body.trim().is_empty()) {
        return Ok(());
    }
    println!();

    let mut printer = PrettyPrinter::new();
    if let Ok(v) = serde_json::from_str::<Value>(&body) {
        {
            //serializes it with four-space indent
            //ref: |https://stackoverflow.com/a/49087292/8776746|
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            v.serialize(&mut ser).unwrap();
            body = String::from_utf8(buf).unwrap();
        }
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

    Ok(())
}
