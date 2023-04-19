# Unplugged search 🐧🎙️🔎

Local First search engine for the [linux unplugged show](https://www.jupiterbroadcasting.com/show/linux-unplugged/).

This project is under development ⚠️

### Local First TL;DR

There's no central server. The app runs locally on your machine.

You have control over the app. The server and client work on you local network.

## Table of contents

- [Setup](#setup)
- [Running the web server](#running-the-web-server)
- [Usage](#usage)

### Features

- [x] Search by tag
- [x] Search by episode id
- [x] Search by title or partial title
- [x] `" "` operator to include the exact contents in the search
- [x] Episode discovery (through tags in the results page)
- [x] `-` Exclude operator
- [ ] Faster tag search
- [ ] Dockerfile

![showcase engine](./assets/showcase-lu-engine-v1.gif)

## Setup

### Requirements

- Python3 (min ver. 3.10)
  - requests & BeautifulSoup
- Rust (min ver. 1.68)

### Indexing

Run the python script at the root of the project:

```sh
python index/main.py --range 1 16 --download
```

Range flag accepts two parameters, `from` and `to`.

If you already have the pages, run the following to index the pages:

```sh
python index/main.py --range 1 16
```

Getting help

```sh
python index/main.py --help
```

## Running the web server

```sh
cargo run --release
```

Set the `DEBUG` variable to enable logging

```sh
export DEBUG=true; cargo run --release
```

The web server is listening on

```sh
127.0.0.1:3000
```

## Usage

In the input field, search using keywords / episode id / partial titles separated by whitespace.

Example queries:

```sh
# searches episodes with ubuntu and "remote desktop" tags
ubuntu "remote desktop"

# searches "docker shocker" and fedora but excludes episodes with nixos or "windows server" tags
"docker shocker" fedora -nixos -"windows server"

# searches episodes with a similar title as "DNF or Die"
"DNF or Die"

# searches for the episode with id 404
404
```

![showcase engine](./assets/showcase-lu-engine-v2.gif)
