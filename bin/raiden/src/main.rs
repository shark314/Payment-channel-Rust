#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate tokio;
extern crate web3;

use clap::{App, Arg, SubCommand};
use cli::{Config, RaidenApp};
use ethsign::SecretKey;
use web3::types::Address;
use std::{path::PathBuf, process};

mod accounts;
mod cli;
mod event_handler;
mod raiden_service;
// mod sync_service;
mod traits;

#[tokio::main]
async fn main() {
    let cli = App::new("Raiden unofficial rust client")
        .arg(
            Arg::with_name("chain-id")
                .short("c")
                .long("chain-id")
                .possible_values(&["ropsten", "kovan", "goerli", "rinkeby", "mainnet"])
                .default_value("mainnet")
                .required(true)
                .takes_value(true)
                .help("Specify the blockchain to run Raiden with"),
        )
        .arg(
            Arg::with_name("eth-rpc-endpoint")
                .long("eth-rpc-endpoint")
                .required(true)
                .takes_value(true)
                .help("Specify the RPC endpoint to interact with"),
        )
        .arg(
            Arg::with_name("eth-rpc-socket-endpoint")
                .long("eth-rpc-socket-endpoint")
                .required(true)
                .takes_value(true)
                .help("Specify the RPC endpoint to interact with"),
        )
        .arg(
            Arg::with_name("keystore-path")
                .short("k")
                .long("keystore-path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(SubCommand::with_name("run").about("Run the raiden client"));

    let matches = cli.get_matches();
	let configs = match Config::new(matches.clone()) {
		Ok(configs) => configs,
		Err(e) => {
			eprintln!("Error: {}", e);
			process::exit(1);
		},
	};

	let (node_address, secret_key) = match prompt_key(configs.clone().keystore_path) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {}", e);
			process::exit(1);
		}
	};

	let raiden_app = RaidenApp::new(configs, node_address, secret_key);
    if let Some(_) = matches.subcommand_matches("run") {
		raiden_app.run().await;
        //let server = http::server(log.clone());
        // let _ = eloop.run(server);
    }
}

fn prompt_key(keystore_path: PathBuf) -> (Address, SecretKey) {
	let keys = accounts::list_keys(keystore_path.as_path()).unwrap();
	let selected_key_filename = crate::cli::prompt_key(&keys);
	let our_address = keys[&selected_key_filename].clone();
	let secret_key = crate::cli::prompt_password(selected_key_filename);

	(our_address, secret_key)
}
