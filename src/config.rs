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

//1. deserializes `t`
//2. performs string replace for the resultant string
//3. serizalizes the string after the replace and returns it
//4. it is assumed the caller would substitute (i.e. override) the returned value to `t`
fn variable_expansion<T>(t: &T, variables: &HashMap<String, String>) -> Result<T, Box<dyn Error>>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let json_string = serde_json::to_string(t).unwrap();
    let mut expanded_json_string = json_string.clone();
    let regex = Regex::new(r#"\$\{([^}]+)}"#)?;
    let mut processed = HashSet::new();
    for c in regex.captures_iter(&json_string) {
        let placeholder = c.get(0).unwrap().as_str();
        let variable_name = c.get(1).unwrap().as_str();
        if (processed.contains(&variable_name)) {
            continue;
        }
        processed.insert(variable_name);
        if let Some(v) = variables.get(variable_name) {
            expanded_json_string = expanded_json_string.replace(placeholder, v);
        } else {
            return Err(format!("variable `{}` is not defined", variable_name).into());
        }
    }
    serde_json::from_str(&expanded_json_string).map_err(|e| e.into())
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

        let mut ret = serde_json::from_str::<Self>(&json_string)?;

        //performs variable expansion
        ret.default_header = variable_expansion(&ret.default_header, &ret.variables)?;
        for i in 0..ret.requests.len() {
            //merges the global `variables` and local-to-request `variables`
            let mut variables = ret.variables.clone();
            if let Some(m) = &ret.requests[i].variables {
                m.iter().for_each(|(k, v)| {
                    variables.insert(k.to_owned(), v.to_owned());
                });
            }
            ret.requests[i] = variable_expansion(&ret.requests[i], &variables)?;
        }

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
    pub variables: Option<HashMap<String, String>>,
    pub url: String,
    pub method: HTTPMethod,
    pub header: HashMap<String, String>,
    pub params: HashMap<String, Value>,
    pub body: Option<HashMap<String, Value>>,
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
