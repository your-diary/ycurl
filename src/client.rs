use std::{collections::HashMap, error::Error};

use reqwest::{
    blocking::Response,
    header::{HeaderMap, HeaderName},
};

use super::config::{Config, HTTPMethod, Request};

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

impl Client {
    pub fn new(config: &Config, request: &Request) -> Result<Self, Box<dyn Error>> {
        let url = if (request.url.starts_with("http")) {
            request.url.clone()
        } else {
            format!("{}/{}", config.base_url, request.url)
        };

        let client = reqwest::blocking::Client::builder()
            .default_headers(create_headermap(&config.default_header))
            .build()?;

        let mut client = match (request.method) {
            HTTPMethod::Get => client.get(url),
            HTTPMethod::Post => client.post(url),
            HTTPMethod::Put => client.put(url),
            HTTPMethod::Delete => client.delete(url),
            HTTPMethod::Patch => client.patch(url),
            HTTPMethod::Head => client.head(url),
        };

        client = client
            .headers(create_headermap(&request.header))
            .query(&request.params)
            .body(serde_json::to_string_pretty(&request.body).unwrap());

        Ok(Self { client })
    }

    pub fn send(self) -> reqwest::Result<Response> {
        self.client.send()
    }
}
