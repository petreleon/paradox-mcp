mod args;
mod handlers;
mod mcp;
mod pxlib;

use args::Args;
use clap::Parser;
use mcp::{RpcRequest, RpcResponse};
use std::io::{self, BufRead, Write};

fn main() {
    let args = Args::parse();

    // Minimal initialization of pxlib
    unsafe {
        pxlib::PX_boot();
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Ok(req) = serde_json::from_str::<RpcRequest>(&line) {
                if let Some(id) = req.id.clone() {
                    let response = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(handlers::handle_request(&req, &args)),
                        error: None,
                    };
                    if let Ok(json_response) = serde_json::to_string(&response) {
                        writeln!(stdout, "{}", json_response).unwrap();
                        stdout.flush().unwrap();
                    }
                }
            }
        }
    }

    unsafe {
        pxlib::PX_shutdown();
    }
}
