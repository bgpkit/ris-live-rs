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

## Built with ❤️ by BGPKIT Team

BGPKIT is a small-team start-up that focus on building the best tooling for BGP data in Rust. We have 10 years of
experience working with BGP data and believe that our work can enable more companies to start keeping tracks of BGP data
on their own turf. Learn more about what services we provide at https://bgpkit.com.

<a href="https://bgpkit.com"><img src="https://bgpkit.com/Original%20Logo%20Cropped.png" alt="https://bgpkit.com/favicon.ico" width="200"/></a>
