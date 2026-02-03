use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;
use std::os::unix::io::AsRawFd;

use smoltcp::iface::{Config, Interface, NeighborCache, SocketSet, Routes};
use smoltcp::phy::{wait as phy_wait, Device};
use smoltcp::phy::tun_tap_interface::TapInterface;
use smoltcp::socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use url::Url;

#[derive(Debug)]
pub enum UpstreamError {
    Network(smoltcp::Error),
    InvalidUrl,
    Content(std::str::Utf8Error),
}

// ... (Display and From impls are fine) ...

pub fn get(
    mut tap: TapInterface,
    mac: EthernetAddress,
    addr: IpAddr,
    url: Url,
) -> Result<(), UpstreamError> {
    let domain_name = url.host_str().ok_or(UpstreamError::InvalidUrl)?;
    
    // Setup Interface
    let mut config = Config::new(mac.into());
    let mut iface = Interface::new(config, &mut tap, Instant::now());
    
    // Set Rust's IP to .100 and Gateway (Host) to .1
    iface.update_ip_addrs(|addrs| {
        addrs.push(IpCidr::new(IpAddress::v4(192, 168, 42, 100), 24)).unwrap();
    });
    iface.routes_mut().add_default_ipv4_route(Ipv4Address::new(192, 168, 42, 1)).unwrap();

    let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    let http_header = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url.path(),
        domain_name,
    );

    let mut state = HttpState::Connect;
    let fd = tap.as_raw_fd();

    loop {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut tap, &mut sockets);

        let mut socket = sockets.get_mut::<TcpSocket>(tcp_handle);

        match state {
            HttpState::Connect if !socket.is_active() => {
                socket.connect(iface.context(), (addr, 80), 49152).unwrap();
                state = HttpState::Request;
            }
            HttpState::Request if socket.may_send() => {
                socket.send_slice(http_header.as_ref()).unwrap();
                state = HttpState::Response;
            }
            HttpState::Response if socket.can_recv() => {
                socket.recv(|data| {
                    print!("{}", String::from_utf8_lossy(data));
                    (data.len(), ())
                }).unwrap();
            }
            HttpState::Response if !socket.is_active() && !socket.may_recv() => break,
            _ => {}
        }

        phy_wait(fd, iface.poll_delay(timestamp, &sockets)).ok();
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum HttpState { Connect, Request, Response }
