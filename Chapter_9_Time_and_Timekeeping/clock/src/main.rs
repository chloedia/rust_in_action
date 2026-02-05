use chrono::{Local,TimeZone, DateTime};
use clap::{Parser, Subcommand};
#[cfg(not(windows))]
use libc;

use std::mem::zeroed;
    

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
    }
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
