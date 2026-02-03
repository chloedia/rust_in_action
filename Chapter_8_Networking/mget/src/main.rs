use clap::Parser;
use url::Url;
use std::net::Ipv4Addr;
use smoltcp::phy::tun_tap_interface::TapInterface;
mod dns;
mod ethernet;
mod http;

#[derive(Parser, Debug)]
#[command(name = "mget", about = "GET a webpage manually")]
struct Cli {
    url: Url,
    #[arg(long)]
    tap_device: String,
    #[arg(long, default_value = "1.1.1.1")]
    dns_server: Ipv4Addr,
}

fn main() {
    let args = Cli::parse();

    if args.url.scheme() != "http" {
        eprintln!("error: only HTTP protocol supported");
        std::process::exit(1);
    }

    let tap = TapInterface::new(&args.tap_device)
        .expect("error: unable to use network interface");

    let domain_name = args.url.host_str().expect("domain name required");

    let addr = dns::resolve(args.dns_server, domain_name)
        .expect("DNS resolution failed")
        .expect("No address found");

    let mac = ethernet::MacAddress::new().into();

    // Pass addr (the resolved IP) and args.url to the http module
    http::get(tap, mac, addr, args.url).unwrap();
}
