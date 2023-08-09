use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub base_url: String,
    pub default_header: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub requests: Vec<Request>,
}

impl Config {
    pub fn new(config_file: &str) -> Result<Self, Box<dyn Error>> {
        let json_string: String = {
            let file = File::open(config_file)?;
            let comment_regex = Regex::new(r#"^\s*#.*"#)?;
            BufReader::new(file)
                .lines()
                .filter(|l| !comment_regex.is_match(l.as_ref().unwrap()))
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n")
        };

        let ret = serde_json::from_str::<Self>(&json_string)?;
        ret.validate()?;
        Ok(ret)
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        let mut s = HashSet::new();
        for i in 0..self.requests.len() {
            if (s.contains(&self.requests[i].name)) {
                return Err(format!(
                    "two or more entries have the same name: {}",
                    self.requests[i].name
                )
                .into());
            }
            s.insert(&self.requests[i].name);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub name: String,
    pub description: String,
    pub url: String,
    pub method: HTTPMethod,
    pub header: HashMap<String, String>,
    pub params: HashMap<String, Value>,
    pub body: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HTTPMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}
