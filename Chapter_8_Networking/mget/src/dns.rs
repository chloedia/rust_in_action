use std::error::Error;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr, IpAddr};
use std::time::Duration;

use trust_dns_client::op::{Message, MessageType, OpCode, Query};
use trust_dns_client::proto::error::ProtoError;
use trust_dns_client::rr::domain::Name;
use trust_dns_client::rr::record_type::RecordType;
use trust_dns_client::serialize::binary::*;

#[derive(Debug)]
pub enum DnsError {
    ParseDomainName(ProtoError),
    ParseDnsServerAddress(std::net::AddrParseError),
    Encoding(ProtoError),
    Decoding(ProtoError),
    Network(std::io::Error),
    Sending(std::io::Error),
    Receiving(std::io::Error),
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DnsError {}

fn message_id() -> u16 {
    let mut id = rand::random::<u16>();
    if id == 0 { id = 1; }
    id
}

pub fn resolve(
    dns_server_address: Ipv4Addr,
    domain_name: &str,
) -> Result<Option<IpAddr>, Box<dyn Error>> {
    let domain_name = Name::from_ascii(domain_name).map_err(DnsError::ParseDomainName)?;
    let dns_server = SocketAddr::new(IpAddr::V4(dns_server_address), 53);

    let mut request_buffer = Vec::with_capacity(64);
    let mut response_buffer = vec![0; 512];

    let mut request = Message::new();
    request.add_query(Query::query(domain_name, RecordType::A));
    request.set_id(message_id())
           .set_message_type(MessageType::Query)
           .set_op_code(OpCode::Query)
           .set_recursion_desired(true);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(DnsError::Network)?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).map_err(DnsError::Network)?;

    let mut encoder = BinEncoder::new(&mut request_buffer);
    request.emit(&mut encoder).map_err(DnsError::Encoding)?;

    socket.send_to(&request_buffer, dns_server).map_err(DnsError::Sending)?;

    let (_amt, remote) = socket.recv_from(&mut response_buffer).map_err(DnsError::Receiving)?;
    
    let response = Message::from_vec(&response_buffer).map_err(DnsError::Decoding)?;

    for answer in response.answers() {
        if answer.record_type() == RecordType::A {
            if let Some(data) = answer.data() {
                if let Some(ip) = data.as_a() {
                    return Ok(Some(IpAddr::V4(ip.0)));
                }
            }
        }
    }
    Ok(None)
}
