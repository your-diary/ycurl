# `ycurl`

## 1. About

A JSON-based cross-platform CUI client for HTTP testing with a variable expansion functionality.

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
      --show-header  Show response header
  -v, --verbose      Verbose mode
  -h, --help         Print help
  -V, --version      Print version
```

## 3. Logging

`ycurl` writes logs to `~/logs/ycurl.txt`. The directory `~/logs` is created if not exists.

## 4. Configurations

By default, requests are defined in `./ycurl.json`. This can be overridden via `-f <file>` option.

### 4.1 Syntax

```json
{
    "base_url": "http://localhost:3000",
    "variables": {
        "name": "Mike"
    },
    "default_header": {
        "Content-Type": "application/json"
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
            "disabled": true,
            "name": "create_user",
            "description": "creates a user",
            "variables": {
                "password": "abc",
                "age": 18
            },
            "url": "/user",
            "method": "POST",
            "header": {},
            "params": {},
            "body": {
                "name": "${name}",
                "password": "${password}",
                "age": "number:${age}"
            }
        }
    ]
}
```

### 4.2 Comments

Lines start with `#`, optionally preceded by spaces, are treated as comments.

```json
{
    "base_url": "http://localhost:3000",
    #This is a comment.
    "default_header": {
        "Content-Type": "application/json"
    },
    ...
}
```

### 4.3 Variable Expansion

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

Every expression of the form `${<variable name>}` inside a string is replaced by the value of the variable `<variable name>`.

```json
{
    ...,
    "requests": [
        {
            "name": "a",
            ...
            "body": {
                #This is interpreted as `"name": "Mike"`.
                "name": "${name}",
                "age": 18
            }
        },
        ...
    ]
}
```

It is possible a variable definition itself includes variables to be expanded. For example, this is a valid definition:
```json
{
    ...
    "variables": {
        "id": "123",
        "name": "Mike",
        "email": "${name}_${id}@example.com"
    },
    ...
}
```

### 4.4 Type Cast

Though a variable expansion occurs only in a string, there should be cases where you want to perform a variable expansion for another datatypes.

```json
"body": {
    #This is invalid as `${id}` is not quoted. 
    "id": ${id}
}
```

For that purpose, type cast is performed when a string starts with `number:` or `bool:`.
```json
"variables": {
    "id": "123",
    "flag": "true"
}
"body": {
    #same as `"id": 123123`
    "id": "number:${id}${id}",
    #same as `"flag": true`
    "flag": "bool:${flag}"
}
```

<!-- vim: set spell: -->
