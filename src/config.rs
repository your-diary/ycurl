use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

//expands variables inside variable definition itself
//This allows variable definition like this:
// "variables": {
//     "id": "123",
//     "name": "Mike",
//     "email": "${name}_${id}@example.com"
// }
//As the definitions are scanned from the first line to the last line, this is invalid:
// "variables": {
//     "id": "123",
//     "email": "${name}_${id}@example.com", //`name` is not defined
//     "name": "Mike"
// }
//The optional parameter `global_variables` is used to merge local variables into the global variables.
fn create_local_variables(
    variables: &IndexMap<String, String>,
    global_variables: Option<&IndexMap<String, String>>,
) -> Result<IndexMap<String, String>, Box<dyn Error>> {
    let mut ret = if let Some(v) = global_variables {
        v.clone()
    } else {
        IndexMap::<String, String>::new()
    };
    let regex = Regex::new(r#"\$\{([^}]+)}"#)?;
    for (k, s) in variables {
        let mut s_expanded = s.clone();
        let mut processed = HashSet::new();
        for c in regex.captures_iter(s) {
            let placeholder = c.get(0).unwrap().as_str();
            let variable_name = c.get(1).unwrap().as_str();
            if (processed.contains(&variable_name)) {
                continue;
            }
            processed.insert(variable_name);
            if let Some(v) = ret.get(variable_name) {
                s_expanded = s_expanded.replace(placeholder, v);
            } else {
                return Err(format!("variable `{}` is not defined", variable_name).into());
            }
        }
        ret.insert(k.clone(), s_expanded);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests_create_local_variables {
    //{{{
    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        let mut variables = IndexMap::new();
        variables.insert("id".to_owned(), "12345".to_owned());
        variables.insert("color".to_owned(), "red".to_owned());
        variables.insert("email".to_owned(), "test_${id}_${id}_${color}".to_owned());
        let v = create_local_variables(&variables, None);
        println!("{:?}", v);
        assert!(v.is_ok());
        variables = v.unwrap();
        assert_eq!(3, variables.len());
        assert_eq!("12345", variables["id"]);
        assert_eq!("red", variables["color"]);
        assert_eq!("test_12345_12345_red", variables["email"]);
    }

    #[test]
    // #[ignore]
    fn test02() {
        let mut variables = IndexMap::new();
        variables.insert("id".to_owned(), "12345".to_owned());
        variables.insert("email".to_owned(), "test_${id}_${id}_${color}".to_owned());
        variables.insert("color".to_owned(), "red".to_owned());
        let v = create_local_variables(&variables, None);
        println!("{:?}", v);
        assert_eq!(
            "variable `color` is not defined",
            v.map_err(|e| e.to_string()).unwrap_err()
        );
    }

    #[test]
    // #[ignore]
    fn test03() {
        let mut global_variables = IndexMap::new();
        global_variables.insert("id".to_owned(), "xyz".to_owned());
        global_variables.insert("name".to_owned(), "Mike".to_owned());
        global_variables.insert("pi".to_owned(), "3.14".to_owned());
        let mut variables = IndexMap::new();
        variables.insert("id".to_owned(), "12345".to_owned());
        variables.insert("color".to_owned(), "red".to_owned());
        variables.insert(
            "email".to_owned(),
            "test_${id}_${id}_${color}_${name}".to_owned(),
        );
        variables.insert("name".to_owned(), "Lisa".to_owned());
        let v = create_local_variables(&variables, Some(&global_variables));
        println!("{:?}", v);
        assert!(v.is_ok());
        variables = v.unwrap();
        assert_eq!(5, variables.len());
        assert_eq!("12345", variables["id"]);
        assert_eq!("red", variables["color"]);
        assert_eq!("test_12345_12345_red_Mike", variables["email"]);
        assert_eq!("Lisa", variables["name"]);
        assert_eq!("3.14", variables["pi"]);
    }
    //}}}
}

//1. deserializes `t`
//2. performs string replace for the resultant string
//3. serizalizes the string after the replace and returns it
//4. it is assumed the caller would substitute (i.e. override) the returned value to `t`
fn variable_expansion<T>(t: &T, variables: &IndexMap<String, String>) -> Result<T, Box<dyn Error>>
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

#[cfg(test)]
mod tests_variable_expansion {
    //{{{
    use super::*;

    use serde_json::json;

    #[test]
    // #[ignore]
    fn test01() {
        let mut variables = IndexMap::new();
        variables.insert("id".to_owned(), "12345".to_owned());
        variables.insert("name".to_owned(), "Mike".to_owned());

        let input = json!({
            "email": "${name}_${id}@example.com",
            "a": "${id}_${name}_${name}_${id}"
        });

        let expected = json!({
            "email": "Mike_12345@example.com",
            "a": "12345_Mike_Mike_12345"
        });

        let actual = variable_expansion(&input, &variables);
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    // #[ignore]
    fn test02() {
        let mut variables = IndexMap::new();
        variables.insert("id".to_owned(), "12345".to_owned());

        let input = json!({
            "email": "${name}_${id}@example.com"
        });

        let actual = variable_expansion(&input, &variables);
        assert!(actual.is_err());
        assert_eq!(
            "variable `name` is not defined",
            actual.map_err(|e| e.to_string()).unwrap_err()
        );
    }
    //}}}
}

//recursively replaces `Value::String(s)` with `Value::Number` or `Value::Bool` if `s` starts with `number:` or `bool:`
//This is useful for example when you want to perform a variable expansion and then cast the result to a number (e.g. `"id": "number:${id}"`).
fn type_cast(v: &mut Value) -> Result<(), Box<dyn Error>> {
    match v {
        Value::String(s) => {
            if (s.starts_with("number:")) {
                *v = Value::Number(s.replace("number:", "").parse()?);
            } else if (s.starts_with("bool:")) {
                *v = Value::Bool(s.replace("bool:", "").parse()?);
            }
        }
        Value::Array(l) => {
            for e in l.iter_mut() {
                type_cast(e)?;
            }
        }
        Value::Object(o) => {
            for (_, v) in o.iter_mut() {
                type_cast(v)?;
            }
        }
        _ => (),
    }
    Ok(())
}

#[cfg(test)]
mod tests_type_cast {
    //{{{
    use super::*;

    use serde_json::json;

    #[test]
    // #[ignore]
    fn test01() {
        let mut input = json!({
            "a": "123",
            "b": "true",
            "c": {
                "d": [
                    {
                        "e": "123",
                        "f": "-123",
                        "g": "3.14",
                        "h": "true",
                        "i": "false",
                        "j": "number:123",
                        "k": "number:-123",
                        "l": "number:3.14",
                        "m": "bool:true",
                        "n": "bool:false"
                    }
                ]
            }
        });

        let expected = json!({
            "a": "123",
            "b": "true",
            "c": {
                "d": [
                    {
                        "e": "123",
                        "f": "-123",
                        "g": "3.14",
                        "h": "true",
                        "i": "false",
                        "j": 123,
                        "k": -123,
                        "l": 3.14,
                        "m": true,
                        "n": false
                    }
                ]
            }
        });

        let ret = type_cast(&mut input);
        println!("{:?}", ret);
        assert!(ret.is_ok());

        assert_eq!(expected, input);
    }

    #[test]
    // #[ignore]
    fn test02() {
        let mut input = json!({
            "a": "bool:abc"
        });
        let ret = type_cast(&mut input);
        println!("{:?}", ret);
        assert!(ret.is_err());
        assert_eq!(
            "provided string was not `true` or `false`",
            ret.map_err(|e| e.to_string()).unwrap_err()
        );
    }

    //}}}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cli_options: CLIOptions,
    pub base_url: String,
    #[serde(default)]
    pub variables: IndexMap<String, String>,
    #[serde(default)]
    pub default_header: HashMap<String, String>,
    pub requests: Vec<Request>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CLIOptions {
    #[serde(default)]
    pub show_header: bool,
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub disabled: bool,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub variables: Option<IndexMap<String, String>>,
    pub url: String,
    pub method: HTTPMethod,
    #[serde(default)]
    pub header: HashMap<String, String>,
    #[serde(default)]
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
        Config::new_with_json_string(&json_string)
    }

    fn new_with_json_string(json_string: &str) -> Result<Self, Box<dyn Error>> {
        let mut ret = serde_json::from_str::<Self>(json_string)?;

        ret.variables = create_local_variables(&ret.variables, None)?;

        //performs variable expansion
        ret.default_header = variable_expansion(&ret.default_header, &ret.variables)?;
        for i in 0..ret.requests.len() {
            //merges the global `variables` and local-to-request `variables`
            if let Some(m) = &ret.requests[i].variables {
                let variables = create_local_variables(m, Some(&ret.variables))?;
                ret.requests[i] = variable_expansion(&ret.requests[i], &variables)?;
            } else {
                ret.requests[i] = variable_expansion(&ret.requests[i], &ret.variables)?;
            }
        }

        for i in 0..ret.requests.len() {
            if let Some(ref mut v) = ret.requests[i].body {
                for (_, v) in v.iter_mut() {
                    type_cast(v)?;
                }
            }
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

#[cfg(test)]
mod tests_config {
    //{{{
    use super::*;

    use serde_json::json;

    #[test]
    // #[ignore]
    fn test01() {
        let input = r#"
            {
                "base_url": "http://example.com",
                "variables": {
                    "id": "123",
                    "name": "Mike",
                    "email": "${name}_${id}@gmail.com",
                    "color": "red"
                },
                "default_header": {
                    "x": "y",
                    "p": "Bearer: ${id}"
                },
                "requests": [
                    {
                        "name": "req1",
                        "description": "desc",
                        "variables": {
                            "user_id": "50",
                            "color": "blue",
                            "flag": "true",
                            "var1": "${flag}",
                            "var2": "${var1}_${id}"
                        },
                        "url": "/v1/user/${user_id}",
                        "method": "GET",
                        "header": {
                            "s": "t",
                            "u": "${name}_${user_id}"
                        },
                        "params": {
                            "a": "b",
                            "c": "${user_id}_${name}"
                        },
                        "body": {
                            "f": "g",
                            "h": "${user_id}_${name}_${user_id}_${color}",
                            "i": "number:${user_id}",
                            "j": "bool:${flag}",
                            "k": "${var1}",
                            "l": "${var2}"
                        }
                    }
                ]
            }
        "#;

        let expected = json!({
            "base_url": "http://example.com",
            "variables": {
                "id": "123",
                "name": "Mike",
                "email": "Mike_123@gmail.com",
                "color": "red"
            },
            "default_header": {
                "x": "y",
                "p": "Bearer: 123"
            },
            "requests": [
                {
                    "name": "req1",
                    "description": "desc",
                    "variables": {
                        "user_id": "50",
                        "color": "blue",
                        "flag": "true",
                        "var1": "true",
                        "var2": "true_123"
                    },
                    "url": "/v1/user/50",
                    "method": "GET",
                    "header": {
                        "s": "t",
                        "u": "Mike_50"
                    },
                    "params": {
                        "a": "b",
                        "c": "50_Mike"
                    },
                    "body": {
                        "f": "g",
                        "h": "50_Mike_50_blue",
                        "i": 50,
                        "j": true,
                        "k": "true",
                        "l": "true_123"
                    }
                }
            ]
        });

        let config = Config::new_with_json_string(input);
        println!("{:?}", config);
        assert!(config.is_ok());
        let config = config.unwrap();
        let actual = serde_json::to_value(&config).unwrap();
        assert_eq!(expected, actual);
    }

    //}}}
}
