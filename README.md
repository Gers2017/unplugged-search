# Unplugged search 🐧🎙️🔎

Local First search engine for the [linux unplugged show](https://www.jupiterbroadcasting.com/show/linux-unplugged/).

This project is under development ⚠️

### Local First TL;DR

You have control over the app. The server and client work on you local network.

### Features

- [x] Search by tag
- [x] Search by episode id
- [x] Search by title or partial title
- [x] `" "` operator to include the contents in the search
- [x] Episode's tags connect to other episodes (gray tags in search results)
- [ ] `-` Exclude operator

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

### Running the web server

```sh
cargo run --release
```

The web server is listening on

```sh
127.0.0.1:3000
```

## Usage

In the input field, search using keywords / episode id / partial titles separated by whitespace.

Example queries:

```sh
"remote desktop" ubuntu

"docker shocker" fedora nixos ubuntu

"The docker shocker"

"More features?"
```

![showcase engine](./assets/showcase-lu-engine-v1.gif)
![showcase engine](./assets/showcase-lu-engine-v2.gif)
