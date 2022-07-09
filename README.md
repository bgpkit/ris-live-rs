# ris-live-rs

[![Rust](https://github.com/bgpkit/ris-live-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/bgpkit/ris-live-rs/actions/workflows/rust.yml)

Provides parsing functions for [RIS-Live](https://ris-live.ripe.net/manual/) real-time
BGP message stream JSON data.

The main parsing function, `parse_ris_live_message` converts a JSON-formatted message string into a
vector of `BgpElem`s.

## Example

```rust
use serde_json::json;
use tungstenite::{connect, Message};
use url::Url;
use ris_live_rs::error::ParserRisliveError;
use ris_live_rs::parse_ris_live_message;

const RIS_LIVE_URL: &str = "ws://ris-live.ripe.net/v1/ws/?client=rust-bgpkit-parser";

/// This is an example of subscribing to RIS-Live's streaming data.
///
/// For more RIS-Live details, check out their documentation at https://ris-live.ripe.net/manual/
fn main() {
    // connect to RIPE RIS Live websocket server
    let (mut socket, _response) =
        connect(Url::parse(RIS_LIVE_URL).unwrap())
            .expect("Can't connect to RIS Live websocket server");

    // subscribe to messages from one collector
    let msg = json!({"type": "ris_subscribe", "data": null}).to_string();
    socket.write_message(Message::Text(msg)).unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message").to_string();
        if msg.is_empty() {
            continue
        }
        match parse_ris_live_message(msg.as_str()) {
            Ok(elems) => {
                for e in elems {
                    println!("{}", e);
                }
            }
            Err(error) => {
                if let ParserRisliveError::ElemEndOfRibPrefix = error {
                    println!("{:?}", &error);
                    println!("{}", msg);
                    continue
                }
                break;
            }
        }
    }
}
```

## Filtering

`ris-live-rs` support filtering message by composing customized 
ris-live subscription message. Use the `compose_subscription_message`
function to create a filtering message.

```rust
pub fn compose_subscription_message(
    host: &String,
    msg_type: &Option<String>,
    require: &Option<String>,
    peer: &Option<String>,
    prefix: &Option<String>,
    path: &Option<String>,
    more_specific: &bool,
    less_specific: &bool,
) -> String {
    ...
}

// subscribe to messages from one collector
let msg = compose_subscription_message(
    &opts.host,
    &opts.msg_type,
    &opts.require,
    &opts.peer,
    &opts.prefix,
    &opts.path,
    &opts.more_specific,
    &opts.less_specific
);
println!("{}", &msg);
socket.write_message(Message::Text(msg)).unwrap();
```

## `ris-live-reader`

`ris-live-rs` library also comes with a simple command-line program 
that supports filtering and different output formats: `ris-live-reader`.

[![asciicast](https://asciinema.org/a/zAxCUmUko9H7T8KM9qFY77uPo.svg)](https://asciinema.org/a/zAxCUmUko9H7T8KM9qFY77uPo)

Full command-line options are:
```
ris-live-reader 0.2.0
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

## Minimum Supported Rust Version (MSRV)

`1.46.0`

## Built with ❤️ by BGPKIT Team

BGPKIT is a small-team focuses on building the best open-source tooling for BGP data in Rust. We have more than 10 years of
experience working with BGP data and believe that our work can enable more companies to start keeping tracks of BGP data
on their own turf. Learn more about what services we provide at https://bgpkit.com.

<a href="https://bgpkit.com"><img src="https://bgpkit.com/Original%20Logo%20Cropped.png" alt="https://bgpkit.com/favicon.ico" width="200"/></a>
