use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
};
use std::io::{self, Write};
use std::time::Duration;

const HELP: &str = "Sawbyte 0.1.0 — foundation preview

Usage:
  sawbyte --help
  sawbyte --version
  sawbyte capabilities
  sawbyte demo [--direct]
  sawbyte status <IP:PORT> [virtual-host] [protocol-number]

demo         Print a scheduled input trace; does not connect to Minecraft.
status       Query a Java server's status and ping, with a 5-second deadline.
             IPv6 example: [::1]:25565. DNS/SRV resolution is not yet implemented.
             Protocol defaults to -1 (discovery). Some servers require an explicit value.
capabilities Show implemented features and missing gameplay backends.

This preview cannot log in, move a Minecraft player, or automate screens yet.";

struct TraceBackend;

impl Backend for TraceBackend {
    type Error = io::Error;
    fn supports(&self, _: Capability) -> bool {
        true
    }
    fn apply(&mut self, tick: u64, frame: InputFrame) -> Result<(), Self::Error> {
        writeln!(
            io::stdout().lock(),
            "tick={tick:02} forward={} yaw={:.3} pitch={:.3}",
            frame.buttons.pressed(Button::Forward),
            frame.yaw_delta,
            frame.pitch_delta
        )
    }
    fn release_all(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn demo(policy: impl InputPolicy) -> Result<(), Box<dyn std::error::Error>> {
    let mut scheduler = Scheduler::new(TraceBackend, policy, 64);
    scheduler.submit(&[
        (
            0,
            Action::Button {
                button: Button::Forward,
                down: true,
            },
        ),
        (
            0,
            Action::Look {
                yaw: 90.0,
                pitch: 0.0,
            },
        ),
        (
            4,
            Action::Button {
                button: Button::Forward,
                down: false,
            },
        ),
    ])?;
    for tick in 0..10 {
        scheduler.advance(tick)?;
    }
    scheduler.cancel()?;
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("--help" | "-h") if args.len() <= 1 => println!("{HELP}"),
        Some("--version") if args.len() == 1 => println!("sawbyte {}", env!("CARGO_PKG_VERSION")),
        Some("capabilities") if args.len() == 1 => println!(
            "Implemented: typed input scheduler, bounded camera policy, cancellation, status/ping.\n\
             Trace backend: buttons and relative camera input (diagnostic only).\n\
             Native gameplay backend: NOT IMPLEMENTED.\n\
             Full Minecraft client bridge: NOT IMPLEMENTED.\n\
             GUI, inventory, text, authentication, scripting transports: NOT IMPLEMENTED.\n\
             No Minecraft version is certified for gameplay yet."
        ),
        Some("demo") if args.len() == 1 => demo(Humanised::default())?,
        Some("demo") if args.len() == 2 && args[1] == "--direct" => demo(Direct)?,
        Some("status") if (2..=4).contains(&args.len()) => {
            let endpoint: std::net::SocketAddr = args[1].parse()?;
            let hostname = args.get(2).cloned().unwrap_or_else(|| endpoint.ip().to_string());
            let protocol = args.get(3).map(|s| s.parse::<i32>()).transpose()?.unwrap_or(-1);
            let result = sawbyte_protocol::query_status(endpoint, &hostname, protocol, Duration::from_secs(5))?;
            // Escape terminal control characters in untrusted server text.
            println!("Server document (escaped): {}", result.document.escape_debug());
            println!("Ping: {:.2} ms", result.round_trip.as_secs_f64() * 1000.0);
        }
        _ => return Err("invalid arguments; use sawbyte --help".into()),
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sawbyte: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
