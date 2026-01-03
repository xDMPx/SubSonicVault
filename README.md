# SubSonicVault

A small, self-hosted cross-platform server for streaming your music collection, written in Rust.
With plans to support the [Open Subsonic API](https://opensubsonic.netlify.app/) in the future.

## Features

- Ability to handle large music collections
- Hash-based music file ID system that prevents serving duplicate files
- Streams audio format such as: m4b, m4a, mp3, flac, wav, opus
- Multi-platform, runs on Linux and Windows
- Visit home endpoint to get served a random music file

## Clients

Currently only one client is supported:
- [SonicTunes](https://github.com/xDMPx/SonicTunes) - TUI based, works within Termux

## Building

To build, clone this repository and run:
```sh
cargo build --release
```

## Installation

### Linux

Locally:
```sh
CARGO_INSTALL_ROOT=~/.local cargo install --path=.
```

### Windows

Build the binary as described in [Building](#Building) section and use the generated executable in `target/release`.

## How to use

Serve your music collection with server binary:
```sh
subsonic_vault /path/to/music_collection
```

Or via Docker Compose:
```sh
docker-compose up -d
```
by default it serves `~/Music`. To change this adjust volume in `docker-compose.yml`.

## Usage

```
Usage: subsonic_vault [OPTIONS] DIRECTORY
       subsonic_vault --help
Options:
	 --help
	 --port=<u16> # default: 65421
```

## Endpoints
| Endpoint     | Method | Description                                                                          |
| ------------ | ------ | ------------------------------------------------------------------------------------ |
| `/`          | GET    | Serves a random audio file from the collection                                       |
| `/scan`      | GET    | Rescans the base directory                                                           |
| `/files`     | GET    | Returns a JSON array of all indexed audio files with their IDs, paths and MIME types |
| `/file/{id}` | GET    | Streams the audio file by the provided ID/hash                                       |
| `/ping`      | GET    | Health-check; returns JSON `{"status":"ok","version":"<ver>"}`                       |

## Preview

<img src="assets/preview.gif"></img>

## License

This project is licensed under [MIT](LICENSE) License.
