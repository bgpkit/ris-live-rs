extern crate core;

use bgpkit_parser::models::ElemType;
use bgpkit_parser::parse_ris_live_message;
use bgpkit_parser::rislive::error::ParserRisliveError;
use bgpkit_parser::rislive::messages::ris_subscribe::RisSubscribeType;
use bgpkit_parser::rislive::messages::{RisLiveClientMessage, RisSubscribe};
use ipnet::IpNet;
use std::net::IpAddr;
use std::str::FromStr;
use structopt::StructOpt;
use tungstenite::{connect, Message};

const RIS_LIVE_URL_BASE: &str = "ws://ris-live.ripe.net/v1/ws/";

/// ris-live-reader is a simple cli tool that can stream BGP data from RIS-Live project with websocket.
/// Check out https://ris-live.ripe.net/ for more data source information.
#[derive(StructOpt, Debug)]
#[structopt(name = "ris-live-reader")]
struct Opts {
    /// client name to identify the stream
    #[structopt(long, default_value = "ris-live-rs")]
    client: String,

    /// Filter by RRC host: e.g. rrc01. Use "all" for the firehose.
    #[structopt(long, default_value = "rrc21")]
    host: String,

    /// Only include messages of a given BGP or RIS type: UPDATE, OPEN, NOTIFICATION, KEEPALIVE, or RIS_PEER_STATE
    #[structopt(long)]
    msg_type: Option<String>,

    /// Only a given BGP update type: announcement (a) or withdrawal (w)
    #[structopt(long)]
    update_type: Option<String>,

    /// Only include messages containing a given key
    #[structopt(long)]
    require: Option<String>,

    /// Only include messages sent by the given BGP peer
    #[structopt(long)]
    peer: Option<String>,

    /// Filter UPDATE messages by prefixes in announcements or withdrawals
    #[structopt(long)]
    prefix: Option<String>,

    /// Match prefixes that are more specific (part of) `prefix`
    #[structopt(long, parse(from_flag = std::ops::Not::not))]
    more_specific: bool,

    /// Match prefixes that are less specific (contain) `prefix`
    #[structopt(long)]
    less_specific: bool,

    /// ASN or pattern to match against the AS PATH attribute
    #[structopt(long)]
    path: Option<String>,

    /// Output as JSON objects
    #[structopt(long)]
    json: bool,

    /// Pretty-print JSON output
    #[structopt(long)]
    pretty: bool,

    /// Print out raw message without parsing
    #[structopt(long)]
    raw: bool,
}

/// This is an example of subscribing to RIS-Live's streaming data.
///
/// For more RIS-Live details, check out their documentation at https://ris-live.ripe.net/manual/
fn main() {
    let opts: Opts = Opts::from_args();

    let url = format!("{}?client={}", RIS_LIVE_URL_BASE, opts.client);
    // connect to RIPE RIS Live websocket server
    let (mut socket, _response) =
        connect(url.as_str()).expect("Can't connect to RIS Live websocket server");

    let mut subscribe_msg = RisSubscribe::new();
    if opts.host == "all" {
        subscribe_msg.host = None;
    } else {
        subscribe_msg.host = Some(opts.host.clone());
    }
    if let Some(msg_type) = &opts.msg_type {
        subscribe_msg.data_type = match msg_type.as_str() {
            "UPDATE" => Some(RisSubscribeType::UPDATE),
            "OPEN" => Some(RisSubscribeType::OPEN),
            "NOTIFICATION" => Some(RisSubscribeType::NOTIFICATION),
            "KEEPALIVE" => Some(RisSubscribeType::KEEPALIVE),
            "RIS_PEER_STATE" => Some(RisSubscribeType::RIS_PEER_STATE),
            _ => None,
        };
    }

    if let Some(require) = &opts.require {
        subscribe_msg.require = Some(require.to_string());
    }
    if let Some(peer) = &opts.peer {
        subscribe_msg.peer = Some(IpAddr::from_str(peer).unwrap());
    }
    if let Some(prefix) = &opts.prefix {
        subscribe_msg.prefix = Some(IpNet::from_str(prefix).unwrap());
    }
    if let Some(path) = &opts.path {
        subscribe_msg.path = Some(path.to_string());
    }
    if opts.more_specific {
        subscribe_msg.more_specific = Some(true);
    }
    if opts.less_specific {
        subscribe_msg.less_specific = Some(true);
    }
    socket
        .send(Message::Text(subscribe_msg.to_json_string()))
        .unwrap();

    loop {
        let msg = socket.read().expect("Error reading message").to_string();
        if msg.is_empty() {
            continue;
        }
        if opts.raw {
            println!("{}", msg.as_str());
            continue;
        }
        match parse_ris_live_message(msg.as_str()) {
            Ok(elems) => {
                for e in elems {
                    if let Some(t) = &opts.update_type {
                        match t.to_lowercase().chars().next().unwrap() {
                            'a' => match e.elem_type {
                                ElemType::ANNOUNCE => {}
                                ElemType::WITHDRAW => continue,
                            },
                            'w' => match e.elem_type {
                                ElemType::ANNOUNCE => continue,
                                ElemType::WITHDRAW => {
                                    dbg!("withdrawal appeared");
                                }
                            },
                            _ => {
                                panic!("the update types can only be announce or withdrawal")
                            }
                        }
                    }

                    if opts.json {
                        if opts.pretty {
                            println!("{}", serde_json::to_string_pretty(&e).unwrap());
                        } else {
                            println!("{}", serde_json::json!(e));
                        }
                    } else {
                        println!("{}", e);
                    }
                }
            }
            Err(error) => {
                if let ParserRisliveError::ElemEndOfRibPrefix = error {
                    println!("{:?}", &error);
                    println!("{}", msg);
                    continue;
                }
                break;
            }
        }
    }
}
