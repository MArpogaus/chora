//! Command line client for the chora IPC socket.

use std::{io::Read, process::ExitCode};

use chora_scene::{
    Command, Event,
    ipc::{Apply, Client, ClientError, Hello, Query, Subscribe},
};
use clap::{Parser, Subcommand};
use serde_json::Value;

/// Talk to the chora compositor in `CHORA_SOCKET`.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The client name, the owner of the nodes that this call creates.
    #[arg(long, default_value = "choractl")]
    name: String,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the nodes that match a selector, for example `kind:surface`.
    Query {
        /// The selector; every node if missing.
        #[arg(default_value = "")]
        selector: String,
    },
    /// Apply a transaction: a JSON list of commands from a file or stdin.
    Apply {
        /// The file with the commands; stdin if missing or `-`.
        file: Option<String>,
    },
    /// Print events as JSON lines until the core closes the socket.
    Events {
        /// Event names, for example `surface.created`; every event if none.
        names: Vec<String>,
    },
    /// Print the outputs.
    Outputs,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("choractl: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), ClientError> {
    let hello = Hello {
        name: args.name,
        keep_nodes: true,
    };
    let mut client = Client::from_env(&hello)?;
    match args.command {
        Cmd::Query { selector } => {
            print(&client.call::<_, Value>("scene.query", Query { selector })?);
        }
        Cmd::Apply { file } => {
            let commands = read_commands(file.as_deref())?;
            client.call::<_, Value>("scene.apply", Apply { commands })?;
        }
        Cmd::Events { names } => {
            client.call::<_, Value>("events.subscribe", Subscribe { events: names })?;
            loop {
                let event: Event = client.next_event()?;
                println!("{}", serde_json::to_string(&event)?);
            }
        }
        Cmd::Outputs => print(&client.call::<_, Value>("output.list", Value::Null)?),
    }
    Ok(())
}

fn read_commands(file: Option<&str>) -> Result<Vec<Command>, ClientError> {
    let text = match file {
        None | Some("-") => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        }
        Some(path) => std::fs::read_to_string(path)?,
    };
    Ok(serde_json::from_str(&text)?)
}

fn print(value: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("values always serialize")
    );
}
