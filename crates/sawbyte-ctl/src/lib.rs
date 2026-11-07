//! `sawbyte-ctl` library interface for unified agent control & MCP stdio bridge.
#![allow(clippy::all, warnings, unused)]

pub use sawbyte_core::mcp::{LiveSessionTransport, SawbyteControlBridge, TcpLiveSessionTransport};

/// Sends a single live control command to a running `sawbyte-play` or `sawbyte-client` endpoint.
pub fn send_command_to_endpoint(endpoint: &str, command: &str) -> Result<String, String> {
    let mut transport = TcpLiveSessionTransport::new(endpoint.to_string());
    transport.send_live_command(command)
}
