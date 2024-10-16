# ris-live-rs

[![Rust](https://github.com/bgpkit/ris-live-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/bgpkit/ris-live-rs/actions/workflows/rust.yml)

The main parsing function, `parse_ris_live_message` converts a JSON-formatted message string into a
vector of `BgpElem`s. The function is now part of the `bgpkit-parser` library.

## `ris-live-reader`

[![asciicast](https://asciinema.org/a/zAxCUmUko9H7T8KM9qFY77uPo.svg)](https://asciinema.org/a/zAxCUmUko9H7T8KM9qFY77uPo)

Full command-line options are:

```
ris-live-reader 0.3.0
ris-live-reader is a simple cli tool that can stream BGP data from RIS-Live project with websocket. Check out
https://ris-live.ripe.net/ for more data source information

USAGE:
    ris-live-reader [FLAGS] [OPTIONS]

FLAGS:
    -h, --help             Prints help information
        --json             Output as JSON objects
        --less-specific    Match prefixes that are less specific (contain) `prefix`
        --more-specific    Match prefixes that are more specific (part of) `prefix`
        --pretty           Pretty-print JSON output
        --raw              Print out raw message without parsing
    -V, --version          Prints version information

OPTIONS:
        --client <client>              client name to identify the stream [default: ris-live-rs]
        --host <host>                  Filter by RRC host: e.g. rrc01. Use "all" for the firehose [default: rrc21]
        --msg-type <msg-type>          Only include messages of a given BGP or RIS type: UPDATE, OPEN, NOTIFICATION,
                                       KEEPALIVE, or RIS_PEER_STATE
        --path <path>                  ASN or pattern to match against the AS PATH attribute
        --peer <peer>                  Only include messages sent by the given BGP peer
        --prefix <prefix>              Filter UPDATE messages by prefixes in announcements or withdrawals
        --require <require>            Only include messages containing a given key
        --update-type <update-type>    Only a given BGP update type: announcement (a) or withdrawal (w)
```

### Installation

Install via cargo by:

```bash
cargo install ris-live-rs
```

Or checkout the repo and run:

```bash
cargo install --path .
```

The program `ris-live-reader` will be installed to your `$CARGO_HOME/bin` (e.g. `~/.cargo/bin`).

### Run with Docker

```bash
docker run --rm -it bgpkit/ris-live-reader --help
```

## Built with ❤️ by BGPKIT Team

<a href="https://bgpkit.com"><img src="https://bgpkit.com/Original%20Logo%20Cropped.png" alt="https://bgpkit.com/favicon.ico" width="200"/></a>
