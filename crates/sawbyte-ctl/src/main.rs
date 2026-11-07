//! `sawbyte-ctl` v0.5.1 — Unified 65-Command Agent Controller & MCP Server for Sawbyte
//! (Refactored from `horizon-ctl` into native `sawbyte-ctl` naming).
//!
//! Supports both `sawbyte-ctl <command>` (e.g. `state`, `inventory`, `goto`, `break`, `place`,
//! `interact-entity`, `ui`, `click`, `click-slot`, `chat`, `command`, `screenshot`, `launch`)
//! AND `sawbyte-ctl mcp` (stdio JSON-RPC 2.0 MCP server).
#![allow(clippy::all, warnings, unused)]

use sawbyte_core::mcp::{LiveSessionTransport, SawbyteControlBridge, TcpLiveSessionTransport};
use std::io::{self, BufRead, Write};

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
        println!("sawbyte-ctl 0.5.1 — Unified Agent Control CLI & MCP Server for Sawbyte 26.3");
        println!("Usage:");
        println!("  sawbyte-ctl [--socket <addr_or_path>] <command> [args...]");
        println!("  sawbyte-ctl [--socket <addr_or_path>] mcp");
        println!("  sawbyte-ctl launch [--world <NAME>] [--seed <N>] [--creative] [--offscreen] [--wait]");
        println!("  sawbyte-ctl state | looking-at | entities | inventory | screen | ui | messages | events");
        println!("  sawbyte-ctl inspect.sensory | inspect.viewport | inspect.svpe | inspect.xray | inspect.long_range | inspect.small_map");
        println!("  sawbyte-ctl move <dir> <secs> | goto <X> <Y> <Z> | jump | look <yaw> <pitch> | look-at <X> <Y> <Z>");
        println!("  sawbyte-ctl break <X> <Y> <Z> | place <X> <Y> <Z> | interact <X> <Y> <Z> | use-item | attack [ID]");
        println!("  sawbyte-ctl select <SLOT|ITEM> | interact-entity <ID> | click-slot <SLOT> | click <ID> | close");
        println!("  sawbyte-ctl chat <TEXT> | command <TEXT> | wait <TICKS> | screenshot [PATH] | stop | quit");
        return std::process::ExitCode::SUCCESS;
    }

    let mut endpoint = std::env::var("SAWBYTE_CONTROL").unwrap_or_else(|_| "127.0.0.1:25580".to_string());
    let mut idx = 0;
    if args.get(idx).map(String::as_str) == Some("--socket") && idx + 1 < args.len() {
        endpoint = args[idx + 1].clone();
        idx += 2;
    }

    let rem = &args[idx..];
    if rem.first().map(String::as_str) == Some("mcp") {
        let mut bridge = SawbyteControlBridge {
            live_endpoint: Some(endpoint),
            ..Default::default()
        };
        let stdin = io::stdin();
        let mut stdout = io::stdout().lock();
        for line in stdin.lock().lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            let resp = bridge.handle_mcp_json_rpc(&line);
            if !resp.is_empty() {
                let _ = writeln!(stdout, "{resp}");
                let _ = stdout.flush();
            }
        }
        return std::process::ExitCode::SUCCESS;
    }

    let cmd_line = rem.join(" ");
    let mut transport = TcpLiveSessionTransport::new(endpoint);
    match transport.send_live_command(&cmd_line) {
        Ok(reply) => {
            println!("{reply}");
            std::process::ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{{"error": {:?}, "kind": "connection"}}", err);
            std::process::ExitCode::from(3)
        }
    }
}
