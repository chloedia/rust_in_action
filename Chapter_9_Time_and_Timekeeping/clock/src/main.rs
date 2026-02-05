use chrono::{Local,TimeZone, DateTime, Utc, Timelike, Duration as ChronoDuration};
use byteorder::{BigEndian, ReadBytesExt};
use clap::{Parser, Subcommand};
#[cfg(not(windows))]
use libc;

use std::mem::zeroed;
use std::time::Duration;
use std::net::UdpSocket;

const LOCAL_ADDR: &'static str = "0.0.0.0:12300";
const NTP_MESSAGE_LENGTH: usize = 48;
const NTP_TO_UNIX_SECONDS: i64 = 2_208_988_800;
#[derive(Debug)]
struct NTPResult {
    t1 : DateTime<Utc>,
    t2 : DateTime<Utc>,
    t3 : DateTime<Utc>,
    t4 : DateTime<Utc>,
}

struct NTPMessage{
    data : [u8; NTP_MESSAGE_LENGTH]
}

impl NTPResult {
    fn offset(&self) -> i64 {
        let duration = (self.t2 - self.t1) + (self.t4 - self.t3);
        duration.num_milliseconds()/2
    }

    fn delay(&self) -> i64 {
        let delay = (self.t4 - self.t1) - (self.t3 - self.t2);
        delay.num_milliseconds()
    }

}

struct NTPTimestamp {
    seconds : u32,
    fraction : u32,
}

impl From<NTPTimestamp> for DateTime<Utc> {
    fn from(ntp: NTPTimestamp) -> DateTime<Utc> {
        let secs = ntp.seconds as i64 - NTP_TO_UNIX_SECONDS;
        let mut nanos = ntp.fraction as f64;
        nanos *= 1e9;
        nanos /= 2_f64.powi(32);
        Utc.timestamp(secs, nanos as u32)
    }

}

impl From<DateTime<Utc>> for NTPTimestamp {
    fn from(utc : DateTime<Utc>) -> Self {
        let secs = utc.timestamp() + NTP_TO_UNIX_SECONDS;
        let mut fraction = utc.nanosecond() as f64;
        fraction *= 2_f64.powi(32);
        fraction /= 1e9;

        NTPTimestamp {
            seconds: secs as u32,
            fraction: fraction as u32
        }

    }

}

impl NTPMessage {
    fn new() -> Self {
        NTPMessage{
            data: [0; NTP_MESSAGE_LENGTH]
        }
    }
    fn client() -> Self {
        const VERSION: u8 = 0b00_011_000;
        const MODE: u8 = 0b00_000_011;
        let mut msg = NTPMessage::new();
        msg.data[0] |= VERSION;
        msg.data[0] |= MODE;
        msg
    }

    fn parse_timestamp(&self, i: usize) -> Result<NTPTimestamp, std::io::Error> {
        let mut reader = &self.data[i..i + 8];
        let seconds = reader.read_u32::<BigEndian>()?;
        let fraction = reader.read_u32::<BigEndian>()?;

        Ok(NTPTimestamp {
            seconds: seconds,
            fraction: fraction,
            })
    }
    
    fn rx_time(&self) -> Result<NTPTimestamp, std::io::Error> {
        self.parse_timestamp(32)
    }
    fn tx_time(&self) -> Result<NTPTimestamp, std::io::Error> {
        self.parse_timestamp(40)
    }
}

fn weighted_mean(values: &[f64], weights: &[f64]) -> f64 {
    let mut result = 0.0;
    let mut sum_of_weights = 0.0;

    for(v,w) in values.iter().zip(weights) {
        result += v*w;
        sum_of_weights += w;
    }
    result / sum_of_weights
}

fn ntp_roundtrip(host: &str, port: u16) -> Result<NTPResult, std::io::Error> {
    let destination = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(1);

    let request = NTPMessage::client();
    let mut response = NTPMessage::new();

    let message = request.data;

    let udp = UdpSocket::bind(LOCAL_ADDR)?;
    udp.connect(&destination).expect("unable to connect");

    let t1 = Utc::now();
    udp.send(&message)?;
    udp.set_read_timeout(Some(timeout))?;
    udp.recv_from(&mut response.data)?;
    let t4 = Utc::now();
    let t2: DateTime<Utc> = response.rx_time().unwrap().into();
    let t3: DateTime<Utc> = response.tx_time().unwrap().into();

    Ok(NTPResult {
            t1: t1,
            t2: t2,
            t3: t3,
            t4: t4
        })
    }



fn check_time() -> Result<f64, std::io::Error> {
    const NTP_PORT: u16 = 123;
    let servers = [
        "time.nist.gov",
        "time.apple.com",
        "time.euro.apple.com",
        "time.google.com",
        "time2.google.com",
    ];
    let mut times = Vec::with_capacity(servers.len());
    
    for &server in servers.iter() {
        print!("{} =>", server);
        let calc = ntp_roundtrip(&server, NTP_PORT);
        match calc {
            Ok(time) => {
                println!(" {}ms away from local system time", time.offset());
                times.push(time);
            }
            Err(_) => {
                println!(" ? [response took too long]")
            }


        }
    }
    let mut offsets = Vec::with_capacity(servers.len());
    let mut offset_weights = Vec::with_capacity(servers.len());
    
    for time in &times {
        let offset = time.offset() as f64;
        let delay = time.delay() as f64;
        let weight : f64 = 1_000_000.0 / (delay * delay);
        if weight.is_finite() {
            offsets.push(offset);
            offset_weights.push(weight);
        }

    }

    let avg_offset = weighted_mean(&offsets, &offset_weights);
    Ok(avg_offset)
    

}

#[derive(Parser, Debug)]
#[command(name = "clock", about = "get or set the time")]
struct Cli {
    #[command(subcommand)]
    command: Commands
}
#[derive(Subcommand, Debug)]
enum Commands {
    Get {
        #[arg(long, default_value = "rfc2822")]
        use_standard: String,
    },
    Set {
        #[arg(long, default_value = "rfc2822")]
        use_standard: String,
        
        datetime : String
    },
    Check_NTP {
    },
}

struct Clock;
impl Clock {
    fn get() -> DateTime<Local> {
    Local::now()
    }
        
    #[cfg(not(windows))]
    fn set<Tz: TimeZone>(t: DateTime<Tz>) -> () {
        use libc::{timeval, time_t, suseconds_t};
        use libc::{settimeofday, timezone};

        let t = t.with_timezone(&Local);
        let mut u: timeval = unsafe{zeroed()};

        u.tv_sec = t.timestamp() as time_t;
        u.tv_usec = t.timestamp_subsec_micros() as suseconds_t;

        unsafe{
            let mock_tz: *const timezone = std::ptr::null();
            settimeofday(&u as *const timeval, mock_tz);
        }

    }
}
fn main() {
    let args = Cli::parse();
    match args.command {
    Commands::Get {use_standard} => {
        let now = Clock::get();
        match use_standard.as_str() {
            "timestamp" => println!("{}", now.timestamp()),
            "rfc2822" => println!("{}", now.to_rfc2822()),
            "rfc3339" => println!("{}", now.to_rfc3339()),
            _ => eprintln!("Unknown format")
        }
    },
    Commands::Check_NTP {} => {
        let offset = check_time().unwrap() as isize;
        let adjust_ms_ = offset.signum() * offset.abs().min(200) / 5;
        let adjust_ms = ChronoDuration::milliseconds(adjust_ms_ as i64);
        let now: DateTime<Utc> = Utc::now() + adjust_ms;
        Clock::set(now);

    }
    Commands::Set {use_standard, datetime}=>{
        let dt = match use_standard.as_str() {
        "rfc3339" => DateTime::parse_from_rfc3339(&datetime)
            .expect("Failed to parse RFC3339")
            .with_timezone(&Local),
            
        "rfc2822" => DateTime::parse_from_rfc2822(&datetime)
            .expect("Failed to parse RFC2822")
            .with_timezone(&Local),
            
        _ => panic!("Unsupported format for setting time"),
        };
        println!("Parsed Local DateTime : {}", dt);
        Clock::set(dt);
        let maybe_error = std::io::Error::last_os_error();
        let os_error_code = &maybe_error.raw_os_error();

        match os_error_code {
            Some(0) => (),
            Some(_) => eprintln!("Unable to set the time: {:?}", maybe_error),
            None => (),
        }
        let now = Clock::get();
        match use_standard.as_str() {
            "timestamp" => println!("{}", now.timestamp()),
            "rfc2822" => println!("{}", now.to_rfc2822()),
            "rfc3339" => println!("{}", now.to_rfc3339()),
            _ => eprintln!("Unknown format")
        }
    }

    }
}  
