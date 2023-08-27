use std::{collections::HashMap, error::Error};

use itertools::Itertools;
use reqwest::{
    blocking::Response,
    header::{HeaderMap, HeaderName},
    redirect::Policy,
};
use serde_json::Value;

use super::config::{Config, HTTPMethod, Request};
use super::logger::Logger;

pub struct Client {
    client: reqwest::blocking::RequestBuilder,
}

fn create_headermap(m: &HashMap<String, String>) -> HeaderMap {
    let mut header = HeaderMap::new();
    m.iter().for_each(|(k, v)| {
        header.insert(k.parse::<HeaderName>().unwrap(), v.parse().unwrap());
    });
    header
}

fn merge_headermap(m1: &mut HeaderMap, m2: &HeaderMap) {
    m2.iter().for_each(|(k, v)| {
        m1.insert(k, v.clone());
    });
}

impl Client {
    pub fn new(
        config: &Config,
        request: &Request,
        logger: &mut Logger,
    ) -> Result<Self, Box<dyn Error>> {
        let url = if (request.url.starts_with("http")) {
            request.url.clone()
        } else {
            format!("{}{}", config.base_url, request.url)
        };

        let headers = {
            let mut m1 = create_headermap(&config.default_headers);
            let m2 = create_headermap(&request.headers);
            merge_headermap(&mut m1, &m2);
            m1
        };

        let client = reqwest::blocking::Client::builder()
            .redirect(if (config.cli_options.disable_redirect) {
                Policy::none()
            } else {
                Policy::limited(10)
            })
            .build()?;

        let mut client = match (request.method) {
            HTTPMethod::Get => client.get(url),
            HTTPMethod::Post => client.post(url),
            HTTPMethod::Put => client.put(url),
            HTTPMethod::Delete => client.delete(url),
            HTTPMethod::Patch => client.patch(url),
            HTTPMethod::Head => client.head(url),
        };

        client = client.headers(headers.clone()).query(&request.params);
        if let Some(b) = &request.body {
            match b {
                Value::String(s) => client = client.body(s.to_owned()),
                Value::Object(o) => {
                    let mut is_set = false;
                    if let Some(v) = headers.get("content-type") {
                        if (v.to_str()?.contains("application/x-www-form-urlencoded")) {
                            client = client.form(o);
                            is_set = true;
                        }
                    }
                    if (!is_set) {
                        client = client.body(serde_json::to_string_pretty(o)?);
                    }
                }
                _ => unreachable!(),
            }
        }

        if (config.cli_options.verbose) {
            if (request.params.is_empty()) {
                println!("{}\n", request.url);
            } else {
                let query_parameters = request
                    .params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_string().trim_matches('"')))
                    .join("&");
                println!("{}?{}\n", request.url, query_parameters);
            }
        }

        logger.log("[request]\n")?;
        if let Some(rb) = client.try_clone() {
            if let Ok(req) = rb.build() {
                logger.log(&format!("method: {}\n", req.method()))?;
                logger.log(&format!("url: {}\n", req.url().as_str()))?;
                logger.log(&format!("headers: {:?}\n", headers))?;
            } else {
                logger.log(&format!("request: {:?}\n", client))?;
            }
        } else {
            logger.log(&format!("request: {:?}\n", client))?;
        }
        if let Some(body) = &request.body {
            logger.log(&format!("body: {:?}", body))?;
        } else {
            logger.log("body: None")?;
        }

        Ok(Self { client })
    }

    pub fn send(self) -> reqwest::Result<Response> {
        self.client.send()
    }
}
