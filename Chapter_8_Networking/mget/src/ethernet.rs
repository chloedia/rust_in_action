use rand::RngCore;
use smoltcp::wire::EthernetAddress;
use std::fmt;

#[derive(Debug)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub fn new() -> Self {
        let mut octets = [0u8; 6];
        rand::thread_rng().fill_bytes(&mut octets);
        octets[0] |= 0b0000_0010; // Locally administered
        octets[0] &= 0b1111_1110; // Unicast
        MacAddress(octets)
    }
}

impl From<MacAddress> for EthernetAddress {
    fn from(mac: MacAddress) -> Self {
        EthernetAddress(mac.0)
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let o = self.0;
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", o[0], o[1], o[2], o[3], o[4], o[5])
    }
}
