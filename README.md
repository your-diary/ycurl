# `ycurl`

## 1. About

A JSON-based cross-platform CUI client for HTTP testing.

## 2. Usage

### 2.1 Installation

#### From GitHub

```bash
$ cargo install --git 'https://github.com/your-diary/ycurl'
```

#### From Source

```bash
$ git clone 'https://github.com/your-diary/ycurl'
$ cd ycurl/
$ cargo install --path .
```

### 2.2 Run

```bash
$ ycurl [-f <file>] <index> #sends a request of the given index/name
$ ycurl [-f <file>]         #lists all of the requests defined in the config file
```

### 2.3 Show Help

```bash
$ ycurl --help

Usage: ycurl [OPTIONS] [INDEX]

Arguments:
  [INDEX]  Index or name of the request sent

Options:
  -f, --file <FILE>  Config file [default: ./ycurl.json]
  -h, --help         Print help
  -V, --version      Print version
```

## 3. Configurations

By default, requests are defined in `./ycurl.json`. This can be overridden via `-f <file>` option.

### 3.1 Syntax

```json
{
    "base_url": "http://localhost:3000",
    "default_header": {
        "Content-Type": "application/json"
    },
    "variables": {
        "name": "Mike"
    },
    "requests": [
        {
            "name": "search_user_by_name",
            "description": "lists users with the given name",
            "url": "/users/name/${name}",
            "method": "GET",
            "header": {
                "Accept": "application/json"
            },
            "params": {
                "page": 1,
                "per_page": 2
            },
            "body": null
        },
        {
            "name": "create_user",
            "description": "creates a user",
            "variables": {
                "password": "abc"
            },
            "url": "/user",
            "method": "POST",
            "header": {},
            "params": {},
            "body": {
                "name": "${name}",
                "password": "${password}",
                "age": 18
            }
        }
    ]
}
```

### 3.2 Comments

Lines start with `#`, optionally preceded by spaces, are treated as comments.

### 3.3 Variable Expansion

Global variables are defined as `Map<String, String>` in the top-level `variables` field, and local-to-request variables are defined in `variables` field inside the request definition.

```json
{
    "base_url": "http://localhost:3000",
    "variables": {
        "name": "Mike"
    },
    ...
    "requests": [
        {
            "name": "create_user",
            "description": "creates a user",
            "variables": {
                "password": "abc"
            },
            ...
        }
    ]
}
```

Every expression of the form `${<variable name>}` is replaced by the value of the variable `<variable name>`.

```json
{
    ...,
    "requests": [
        {
            "name": "a",
            ...
            "body": {
                "name": "${name}", #This is interpreted as `"name": "Mike"`.
                "age": 18
            }
        },
        ...
    ]
}
```

<!-- vim: set spell: -->

