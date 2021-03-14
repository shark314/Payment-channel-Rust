#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate tokio;
extern crate web3;

use clap::{
    App,
    Arg,
};
use cli::{
    Config,
    RaidenApp,
};
use raiden::blockchain::key::PrivateKey;
use std::{
    fs,
    path::PathBuf,
    process,
};
use web3::types::Address;

mod accounts;
mod cli;
mod event_handler;
mod http;
mod services;
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
            Arg::with_name("datadir")
                .long("datadir")
                .default_value("~/.raiden")
                .takes_value(true)
                .help("Directory for storing raiden data."),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        );

    let matches = cli.get_matches();
    let configs = match Config::new(matches.clone()) {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    match setup_data_directory(configs.clone().datadir) {
        Err(e) => {
            eprintln!("Error initializing data directory: {}", e);
            process::exit(1);
        }
        _ => {}
    };

    let (node_address, secret_key) = prompt_key(configs.clone().keystore_path);

    let raiden_app = match RaidenApp::new(configs, node_address, secret_key) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Error initializing app: {}", e);
            process::exit(1);
        }
    };

    raiden_app.run().await;
    //let server = http::server(log.clone());
    // let _ = eloop.run(server);
}

fn setup_data_directory(path: PathBuf) -> Result<PathBuf, String> {
    if !path.is_dir() {
        return Err("Datadir has to be a directory".to_owned());
    }

    if !path.exists() {
        match fs::create_dir(path.clone()) {
            Err(e) => return Err(format!("Could not create directory: {:?} because {}", path.clone(), e)),
            _ => {}
        }
    }
    Ok(path.to_path_buf())
}

fn prompt_key(keystore_path: PathBuf) -> (Address, PrivateKey) {
    let keys = accounts::list_keys(keystore_path.as_path()).unwrap();
    let selected_key_filename = crate::cli::prompt_key(&keys);
    let our_address = keys[&selected_key_filename].clone();
    let secret_key = crate::cli::prompt_password(selected_key_filename);

    (our_address, PrivateKey::new(secret_key))
}
