use tungstenite::{connect, Message};
use url::Url;
use ris_live_rs::error::ParserRisliveError;
use ris_live_rs::{compose_subscription_message, parse_ris_live_message};
use clap::Parser;

const RIS_LIVE_URL_BASE: &str = "ws://ris-live.ripe.net/v1/ws/";

/// ris-live-reader is a simple cli tool that can stream BGP data from RIS-Live project with websocket.
#[derive(Parser)]
#[clap(version = "0.1.0", author = "Mingwei Zhang <mingwei@bgpkit.com>")]
struct Opts {

    /// client name to identify the stream
    #[clap(long, default_value="ris-live-rs")]
    client: String,

    /// Filter by RRC host: e.g. rrc01
    #[clap(long)]
    host: Option<String>,

    /// Only include messages of a given BGP or RIS type: UPDATE, OPEN, NOTIFICATION, KEEPALIVE, or RIS_PEER_STATE
    #[clap(long)]
    msg_type: Option<String>,

    /// Only include messages containing a given key
    #[clap(long)]
    require: Option<String>,

    /// Only include messages sent by the given BGP peer
    #[clap(long)]
    peer: Option<String>,

    /// Filter UPDATE messages by prefixes in announcements or withdrawals
    #[clap(long)]
    prefix: Option<String>,

    /// Match prefixes that are more specific (part of) `prefix`
    #[clap(long, parse(from_flag = std::ops::Not::not))]
    more_specific: bool,

    /// Match prefixes that are less specific (contain) `prefix`
    #[clap(long)]
    less_specific: bool,

    /// ASN or pattern to match against the AS PATH attribute
    #[clap(long)]
    path: Option<String>,

    /// Output as JSON objects
    #[clap(long)]
    json: bool,

    /// Pretty-print JSON output
    #[clap(long)]
    pretty: bool,

    /// Print out raw message without parsing
    #[clap(long)]
    raw: bool,
}

/// This is an example of subscribing to RIS-Live's streaming data.
///
/// For more RIS-Live details, check out their documentation at https://ris-live.ripe.net/manual/
fn main() {
    let opts: Opts = Opts::parse();

    let url = format!("{}?client={}", RIS_LIVE_URL_BASE, opts.client);
    // connect to RIPE RIS Live websocket server
    let (mut socket, _response) =
        connect(Url::parse(url.as_str()).unwrap())
            .expect("Can't connect to RIS Live websocket server");

    // subscribe to messages from one collector
    let msg = compose_subscription_message(
        opts.host,
        opts.msg_type,
        opts.require,
        opts.peer,
        opts.prefix,
        opts.path,
        opts.more_specific,
        opts.less_specific
    );
    println!("{}", &msg);
    socket.write_message(Message::Text(msg)).unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message").to_string();
        if msg.is_empty() {
            continue
        }
        if opts.raw {
            println!("{}", msg.as_str());
            continue
        }
        match parse_ris_live_message(msg.as_str()) {
            Ok(elems) => {
                for e in elems {
                    if opts.json{
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
                    continue
                }
                break;
            }
        }
    }
}
