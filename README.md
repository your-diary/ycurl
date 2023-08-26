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
  -f, --file <FILE>       Config file [default: ./ycurl.json]
      --show-header       Show response header
      --disable-redirect  Disable following redirects
      --include-disabled  Allow `disabled` request to be sent
      --complete          Output shell completion code
  -v, --verbose           Verbose mode
  -h, --help              Print help
  -V, --version           Print version
```

## 3. Logging

`ycurl` writes logs to `~/logs/ycurl.txt`. The directory `~/logs` is created if not exists.

## 4. Configurations

By default, requests are defined in `./ycurl.json`. This can be overridden via `-f <file>` option.

### 4.1 Examples

```json
{
    "cli_options": {
        "show_header": true,
        "verbose": false
    },
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

### 4.2 Fields

#### 4.2.1 Top-Level Fields

| Name | Type | Required | Description |
| :- | :- | :- | :- |
| `description` | `String` | | Any string used for comment. |
| `cli_options` | `CLIOptions` | | Default values for the command-line options. |
| `base_url` | `String` | ✓ | Base URL like `http://localhost:3000`. |
| `variables` | `Map<String, String>` | | Global [variables](#44-variable-expansion). |
| `default_header` | `Map<String, String>` | | Default HTTP request headers. |
| `requests` | `Vec<Request>` | ✓ | Requests sent. |


### 4.2.2 `CliOptions`

| Name | Type | Required | Description |
| :- | :- | :- | :- |
| `show_header` | `bool` | | Default value for `--show-header` option. |
| `verbose` | `bool` | | Default value for `--verbose` option. |

### 4.2.3 `Request`

| Name | Type | Required | Description |
| :- | :- | :- | :- |
| `disabled` | `bool` | | Disables this request. |
| `name` | `String` | ✓ | Arbitrary human-readable name. |
| `description` | `String` | | Any string used for comment. |
| `variables` | `Map<String, String>` | | Local [variables](#44-variable-expansion), which merges into and overrides the global variables. |
| `url` | `String` | ✓ | Path part of URL (e.g. `/user/create`) appended to `baser_url`. |
| `method` | `String` | ✓ | HTTP method. The value shall be an uppercase HTTP method like `GET` or `POST`. |
| `header` | `Map<String, String>` | | HTTP request headers which merges into and overrides `default_header`. |
| `params` | `Map<String, Any>` | | Query parameters. Specifying query parameters as the part of `url` (e.g. `/user/list?page=3&count=10`) is also supported. |
| `body` | `Map<String, Any>` | | Request body. Currently, this is sent as a JSON string though `Content-Type: application/json` is not implied. |

### 4.3 Comments

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

### 4.4 Variable Expansion

Global variables are defined as `Map<String, String>` in the top-level `variables` field, and local-to-request variables are defined in `variables` field inside the request definition.

```json
{
    "base_url": "http://localhost:3000",
    #global variables
    "variables": {
        "name": "Mike"
    },
    ...
    "requests": [
        {
            "name": "create_user",
            "description": "creates a user",
            #local variables
            "variables": {
                "password": "abc"
            },
            ...
        }
    ]
}
```

Every expression of the form `${<variable name>}` ***inside a string*** is replaced by the value of the variable `<variable name>`.

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

### 4.5 Type Cast

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
