/*!
Provides parsing functions for [RIS-Live](https://ris-live.ripe.net/manual/) real-time
BGP message stream JSON data.

The main parsing function, [parse_ris_live_message] converts a JSON-formatted message string into a
vector of [BgpElem]s.

Here is an example parsing stream data from one collector:
```no_run
use serde_json::json;
use tungstenite::{connect, Message};
use ris_live_rs::error::ParserRisliveError;
use ris_live_rs::parse_ris_live_message;

const RIS_LIVE_URL: &str = "ws://ris-live.ripe.net/v1/ws/?client=rust-bgpkit-parser";

/// This is an example of subscribing to RIS-Live's streaming data.
///
/// For more RIS-Live details, check out their documentation at https://ris-live.ripe.net/manual/
fn main() {
    // connect to RIPE RIS Live websocket server
    let (mut socket, _response) =
        connect(RIS_LIVE_URL)
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
 */

use std::net::IpAddr;
use bgp_models::prelude::*;
use crate::error::ParserRisliveError;
use crate::messages::{RisLiveMessage, RisMessageEnum};
use crate::messages::ris_message::path_to_as_path;

pub mod error;
pub mod messages;

// simple macro to make the code look a bit nicer
macro_rules! unwrap_or_return {
    ( $e:expr, $msg_string:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return Err(ParserRisliveError::IncorrectJson($msg_string)),
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    let mut options: Vec<String> = vec![];

    if host.to_lowercase().as_str() != "all" {
        options.push(format!("\"host\": \"{}\"", host))
    }

    if let Some(msg_type) = msg_type {
        options.push(format!("\"type\": \"{}\"", msg_type))
    }

    if let Some(require) = require {
        options.push(format!("\"require\": \"{}\"", require))
    }

    if let Some(peer) = peer {
        options.push(format!("\"peer\": \"{}\"", peer))
    }

    if let Some(prefix) = prefix {
        options.push(format!("\"prefix\": \"{}\"", prefix))
    }

    if let Some(path) = path {
        options.push(format!("\"path\": \"{}\"", path))
    }

    match more_specific {
        true => {
            options.push("\"moreSpecific\": true".to_string())
        }
        false => {
            options.push("\"moreSpecific\": false".to_string())
        }
    }

    match less_specific {
        true => {
            options.push("\"lessSpecific\": true".to_string())
        }
        false => {
            options.push("\"lessSpecific\": false".to_string())
        }
    }

    format!("{{\"type\": \"ris_subscribe\", \"data\":{{ {} }} }}", options.join(","))
}

/// This function parses one message and returns a result of a vector of [BgpElem]s or an error
pub fn parse_ris_live_message(msg_str: &str) -> Result<Vec<BgpElem>, ParserRisliveError> {

    let msg_string = msg_str.to_string();

    // parse RIS Live message to internal struct using serde.
    let msg: RisLiveMessage = match serde_json::from_str(msg_str) {
        Ok(m) => m,
        Err(_e) => return Err(ParserRisliveError::IncorrectJson(msg_string)),
    };

    match msg {
        RisLiveMessage::RisMessage(ris_msg) => {
            // we currently only handles the `ris_message` data type. other
            // types provides meta information, but reveals no BGP elements, and
            // thus for now will be ignored.

            if ris_msg.msg.is_none() {
                return Ok(vec![])
            }

            match ris_msg.msg.unwrap() {
                RisMessageEnum::UPDATE {
                    path,
                    community,
                    origin,
                    med,
                    aggregator,
                    announcements,
                } => {
                    let mut elems: Vec<BgpElem> = vec![];

                    let peer_ip = unwrap_or_return!(ris_msg.peer.parse::<IpAddr>(), msg_string);
                    let peer_asn = Asn::from(unwrap_or_return!(ris_msg.peer_asn.parse::<u32>(), msg_string));

                    // parse path
                    let as_path = path.map(path_to_as_path);

                    // parse community
                    let communities: Option<Vec<MetaCommunity>> = match community {
                        None => {None}
                        Some(cs) => {
                            let mut comms: Vec<MetaCommunity> = vec![];
                            for c in cs {
                                comms.push(MetaCommunity::Community(Community::Custom(Asn::from(c.0),c.1)));
                            }
                            Some(comms)
                        }
                    };

                    // parse origin
                    let bgp_origin = match origin {
                        None => {None}
                        Some(o) => {
                            Some(match o.as_str(){
                                "igp" | "IGP" => Origin::IGP,
                                "egp" | "EGP" => Origin::EGP,
                                "incomplete" | "INCOMPLETE" => Origin::INCOMPLETE,
                                other => {
                                    return Err(ParserRisliveError::ElemUnknownOriginType(other.to_string()))
                                }
                            })
                        }
                    };

                    // parse aggregator
                    let bgp_aggregator = match aggregator{
                        None => {(None, None)}
                        Some(aggr_str) => {
                            let parts = aggr_str.split(':').collect::<Vec<&str>>();
                            if parts.len()!=2 {
                                return Err(ParserRisliveError::ElemIncorrectAggregator(aggr_str))
                            }
                            let asn = Asn::from(unwrap_or_return!(parts[0].to_owned().parse::<u32>(), msg_string));
                            let ip = unwrap_or_return!(parts[1].to_owned().parse::<IpAddr>(), msg_string);
                            (Some(asn), Some(ip))
                        }
                    };

                    // parser announcements
                    if let Some(announcements) = announcements {
                        for announcement in announcements {
                            let nexthop = match announcement.next_hop.parse::<IpAddr>(){
                                Ok(a) => {a}
                                Err(_) => {
                                    return Err(ParserRisliveError::IncorrectJson(msg_string))
                                }
                            };
                            for prefix in &announcement.prefixes {
                                let p = match prefix.parse::<NetworkPrefix>(){
                                    Ok(net) => { net }
                                    Err(_) => {
                                        if prefix == "eor" {
                                            return Err(ParserRisliveError::ElemEndOfRibPrefix)
                                        }
                                        return Err(ParserRisliveError::ElemIncorrectPrefix(prefix.to_string()))
                                    }
                                };

                                elems.push(
                                    BgpElem{
                                        timestamp: ris_msg.timestamp,
                                        elem_type: ElemType::ANNOUNCE,
                                        peer_ip,
                                        peer_asn,
                                        prefix: p,
                                        next_hop: Some(nexthop),
                                        as_path: as_path.clone(),
                                        origin_asns: None,
                                        origin: bgp_origin,
                                        local_pref: None,
                                        med,
                                        communities: communities.clone(),
                                        atomic: None,
                                        aggr_asn: bgp_aggregator.0,
                                        aggr_ip: bgp_aggregator.1,
                                    }
                                );
                            }

                            if let Some(prefixes) = &announcement.withdrawals {
                                for prefix in prefixes {
                                    let p = match prefix.parse::<NetworkPrefix>(){
                                        Ok(net) => { net }
                                        Err(_) => {
                                            if prefix == "eor" {
                                                return Err(ParserRisliveError::ElemEndOfRibPrefix)
                                            }
                                            return Err(ParserRisliveError::ElemIncorrectPrefix(prefix.to_string()))
                                        }
                                    };
                                    elems.push(
                                        BgpElem{
                                            timestamp: ris_msg.timestamp,
                                            elem_type: ElemType::WITHDRAW,
                                            peer_ip,
                                            peer_asn,
                                            prefix: p,
                                            next_hop: None,
                                            as_path: None,
                                            origin_asns: None,
                                            origin: None,
                                            local_pref: None,
                                            med: None,
                                            communities: None,
                                            atomic: None,
                                            aggr_asn: None,
                                            aggr_ip: None,
                                        }
                                    );

                                }
                            }
                        }
                    }

                    Ok(elems)
                }
                _ => Ok(vec![]),
            }
        },
        _ => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ris_live_msg() {
        let msg_str = r#"
        {"type": "ris_message","data":{"timestamp":1636247118.76,"peer":"2001:7f8:24::82","peer_asn":"58299","id":"20-5761-238131559","host":"rrc20","type":"UPDATE","path":[58299,49981,397666],"origin":"igp","announcements":[{"next_hop":"2001:7f8:24::82","prefixes":["2602:fd9e:f00::/40"]},{"next_hop":"fe80::768e:f8ff:fea6:b2c4","prefixes":["2602:fd9e:f00::/40"], "withdrawals": ["1.1.1.0/24", "8.8.8.0/24"]}],"raw":"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF005A02000000434001010040020E02030000E3BB0000C33D00061162800E2B00020120200107F8002400000000000000000082FE80000000000000768EF8FFFEA6B2C400282602FD9E0F"}}
        "#;
        let msg = parse_ris_live_message(&msg_str).unwrap();
        for elem in msg {
            println!("{}", elem);
        }
    }

    #[test]
    fn test_error_message() {
        let msg_str = r#"
        {"type": "ris_message","data":{"timestamp":1636342486.17,"peer":"37.49.237.175","peer_asn":"199524","id":"21-587-22045871","host":"rrc21","type":"UPDATE","path":[199524,1299,3356,13904,13904,13904,13904,13904,13904],"origin":"igp","aggregator":"65000:8.42.232.1","announcements":[{"next_hop":"37.49.237.175","prefixes":["64.68.236.0/22"]}]}}
        "#;
        let msg = parse_ris_live_message(&msg_str).unwrap();
        for elem in msg {
            println!("{}", elem);
        }
    }

    #[test]
    fn test_error_message_2() {
        let msg_str = r#"
        {"type": "ris_message","data":{"timestamp":1636339375.83,"peer":"37.49.236.1","peer_asn":"8218","id":"21-594-37970252","host":"rrc21"}}
        "#;
        let msg = parse_ris_live_message(&msg_str).unwrap();
        for elem in msg {
            println!("{}", elem);
        }
    }
}
