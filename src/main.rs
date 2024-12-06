extern crate syslog;

use std::process;
use std::str::FromStr;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::Parser;
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(styles = styles())]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Syslogger: CLI tool to send syslog messages", long_about = None)]
struct Cli {
    #[clap(short, long, default_value = "127.0.0.1")]
    ip: String,

    #[clap(short, long, default_value = "514")]
    port: u16,

    #[clap(short, long, help = "Use UDP instead of TCP")]
    udp: bool,

    #[clap(long, default_value = "false", help = "Send pid")]
    pid: bool,

    #[clap(long)]
    facility: Option<String>,

    #[clap(long)]
    tag: Option<String>,

    #[clap(long, help = "Add hostname")]
    hostname: Option<String>,

    #[clap(index = 1)]
    message: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let facility = Facility::from_str(cli.facility.unwrap_or("user".to_string()).as_str());
    if facility.is_err() {
        println!("Invalid facility");
        std::process::exit(1);
    }
    let formatter = Formatter3164 {
        facility: facility.unwrap(),
        process: cli.tag.unwrap_or(env!("CARGO_PKG_NAME").to_string()),
        hostname: cli.hostname,
        pid: if cli.pid { process::id() } else { 0 },
    };
    let server = format!("{}:{}", cli.ip, cli.port);

    let logger: Result<Logger<LoggerBackend, Formatter3164>, syslog::Error> = if !cli.udp {
        syslog::tcp(formatter, server)
    } else {
        syslog::udp(formatter, "0.0.0.0:0", &server)
    };

    if let Err(e) = logger {
        println!("Failed to connect to syslog server: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = logger.unwrap().info(&cli.message.unwrap_or("".to_string())) {
        println!("Failed to send syslog message: {}", e);
        std::process::exit(1);
    }
}
