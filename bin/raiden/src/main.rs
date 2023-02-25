use std::{
	fs,
	path::{
		PathBuf,
	},
	process,
	sync::Arc,
};

use raiden_api::{
	api::Api,
	event_handler::EventHandler,
	payments::PaymentsRegistry,
	raiden::{
		Raiden,
		RaidenConfig,
	},
};
use raiden_blockchain::{
	contracts,
	proxies::{
		Account,
		ProxyManager,
	},
};
use raiden_client::{
	cli::get_private_key,
	services::{
		BlockMonitorService,
		SyncService,
	},
};
use raiden_pathfinding::{
	self,
	config::PFSConfig,
};
use raiden_primitives::types::ChainID;
use raiden_state_machine::types::MediationFeeConfig;
use raiden_storage::state_transition::TransitionService;
use structopt::StructOpt;
use tokio::sync::RwLock;
use tracing::info;
use web3::{
	signing::Key,
	transports::WebSocket,
	types::Address,
};

use crate::{
	cli::Opt,
	traits::{
		ToHTTPEndpoint,
		ToSocketEndpoint,
	},
};

mod cli;
mod http;
mod init;
mod traits;

use init::*;

#[tokio::main]
async fn main() {
	let cli = Opt::from_args();

	tracing_subscriber::fmt::init();

	match setup_data_directory(cli.datadir.clone()) {
		Err(e) => {
			eprintln!("Error initializing data directory: {}", e);
			process::exit(1);
		},
		_ => {},
	};

	let private_key = match get_private_key(cli.keystore_path.clone()) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("{}", e);
			process::exit(1);
		},
	};

	info!("Welcome to Raiden");
	info!("Initializing");

	// #
	// # Initialize chain related components
	// #
	let chain_id: ChainID = cli.chain_id.into();
	let eth_rpc_http_endpoint = match cli.eth_rpc_endpoint.to_http() {
		Ok(e) => e,
		Err(e) => {
			eprintln!("Invalid RPC endpoint: {}", e);
			process::exit(1);
		},
	};

	let eth_rpc_socket_endpoint = match cli.eth_rpc_socket_endpoint.to_socket() {
		Ok(e) => e,
		Err(e) => {
			eprintln!("Invalid RPC endpoint: {}", e);
			process::exit(1);
		},
	};

	// #
	// # Initialize web3
	// #
	let http = web3::transports::Http::new(&eth_rpc_http_endpoint).unwrap();
	let web3 = web3::Web3::new(http);
	let nonce = match web3.eth().transaction_count(private_key.address(), None).await {
		Ok(nonce) => nonce,
		Err(e) => {
			eprintln!("Failed to fetch nonce: {}", e);
			process::exit(1);
		},
	};
	let account = Account::new(web3.clone(), private_key, nonce);

	// #
	// # Initialize state manager
	// #
	let datadir = match expanduser::expanduser(cli.datadir.to_string_lossy()) {
		Ok(p) => p,
		Err(e) => {
			eprintln!("Error expanding data directory: {}", e);
			process::exit(1);
		},
	};

	let storage = match init_storage(datadir) {
		Ok(storage) => storage,
		Err(e) => {
			eprintln!("Error creating contracts manager: {}", e);
			process::exit(1);
		},
	};

	let contracts_manager = match contracts::ContractsManager::new(chain_id.clone()) {
		Ok(contracts_manager) => Arc::new(contracts_manager),
		Err(e) => {
			eprintln!("Error creating contracts manager: {}", e);
			process::exit(1);
		},
	};
	let (state_manager, sync_start_block_number, default_addresses) =
		match init_state_manager(contracts_manager.clone(), storage, chain_id, account.clone()) {
			Ok(result) => result,
			Err(e) => {
				eprintln!("{}", e);
				process::exit(1);
			},
		};

	// #
	// # Initialize PFS
	// #
	let mediation_config = MediationFeeConfig {
		token_to_flat_fee: cli
			.mediation_fees
			.flat_fee
			.into_iter()
			.map(|(a, v)| (Address::from_slice(a.as_bytes()), v.into()))
			.collect(),
		token_to_proportional_fee: cli
			.mediation_fees
			.proportional_fee
			.into_iter()
			.map(|(a, v)| (Address::from_slice(a.as_bytes()), v.into()))
			.collect(),
		token_to_proportional_imbalance_fee: cli
			.mediation_fees
			.proportional_imbalance_fee
			.into_iter()
			.map(|(a, v)| (Address::from_slice(a.as_bytes()), v.into()))
			.collect(),
		cap_meditation_fees: cli.mediation_fees.cap_mediation_fees,
	};

	// #
	// # Initialize transport
	// #
	let (transport_service, transport_sender, our_metadata) = match init_transport(
		cli.environment_type.into(),
		cli.matrix_transport_config.matrix_server,
		cli.matrix_transport_config.retry_timeout,
		cli.matrix_transport_config.retry_count,
		cli.matrix_transport_config.retry_timeout_max,
		account.clone(),
	)
	.await
	{
		Ok(result) => result,
		Err(e) => {
			eprintln!("{}", e);
			process::exit(1);
		},
	};

	let proxy_manager = match ProxyManager::new(web3.clone(), contracts_manager.clone()) {
		Ok(pm) => Arc::new(pm),
		Err(e) => {
			eprintln!("Failed to initialize proxy manager: {}", e);
			process::exit(1);
		},
	};

	let pfs_info = match init_pfs_info(
		contracts_manager.clone(),
		proxy_manager.clone(),
		cli.services_config.clone().into(),
	)
	.await
	{
		Ok(info) => info,
		Err(e) => {
			eprintln!("{}", e);
			process::exit(1);
		},
	};

	// #
	// # Initialize Raiden
	// #
	//
	let config = RaidenConfig {
		chain_id,
		mediation_config,
		account: account.clone(),
		metadata: our_metadata,
		pfs_config: PFSConfig {
			url: cli.services_config.pathfinding_service_specific_address,
			info: pfs_info,
			maximum_fee: cli.services_config.pathfinding_max_fee,
			iou_timeout: cli.services_config.pathfinding_iou_timeout.into(),
			max_paths: cli.services_config.pathfinding_max_paths,
		},
		addresses: default_addresses,
	};
	let raiden = Arc::new(Raiden {
		web3,
		config,
		contracts_manager,
		proxy_manager,
		state_manager,
		transport: transport_sender.clone(),
	});

	let transport_sender_inner = transport_sender.clone();
	let transition_service_account = account.clone();
	let transition_service_raiden = raiden.clone();
	let transition_service =
		Arc::new(TransitionService::new(raiden.state_manager.clone(), move |event| {
			let event_handler = EventHandler::new(
				transition_service_account.clone(),
				transition_service_raiden.state_manager.clone(),
				transport_sender_inner.clone(),
			);
			async move { event_handler.handle_event(event).await }
		}));

	let ws = match WebSocket::new(&eth_rpc_socket_endpoint).await {
		Ok(ws) => ws,
		Err(e) => {
			eprintln!("Error connecting to websocket: {:?}", e);
			process::exit(1);
		},
	};

	let mut sync_service = SyncService::new(raiden.clone(), transition_service.clone());
	let latest_block_number = raiden.web3.eth().block_number().await.unwrap();
	sync_service.sync(sync_start_block_number, latest_block_number.into()).await;

	let block_monitor_service = match BlockMonitorService::new(
		raiden.clone(),
		ws,
		transition_service.clone(),
		sync_service,
	) {
		Ok(service) => service,
		Err(_) => {
			eprintln!("Could not initialize block monitor service");
			process::exit(1);
		},
	};
	let payments_registry = Arc::new(RwLock::new(PaymentsRegistry::new()));
	let api = Api::new(raiden.clone(), transition_service, payments_registry);
	let http_service = crate::http::HttpServer::new(raiden.clone(), Arc::new(api));

	info!("Raiden is starting");

	futures::join!(block_monitor_service.start(), transport_service.run(), http_service.start());
}

fn setup_data_directory(path: PathBuf) -> Result<PathBuf, String> {
	let path = expanduser::expanduser(path.to_string_lossy())
		.map_err(|_| "Failed to expand data directory".to_owned())?;

	if !path.is_dir() {
		return Err("Datadir has to be a directory".to_owned())
	}

	if !path.exists() {
		match fs::create_dir(path.clone()) {
			Err(e) =>
				return Err(format!("Could not create directory: {:?} because {}", path.clone(), e)),
			_ => {},
		}
	}
	Ok(path.to_path_buf())
}
