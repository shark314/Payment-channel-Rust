use clap::ArgMatches;
use ethsign::SecretKey;
use futures::{StreamExt, executor, future, stream::FuturesUnordered};
use parking_lot::RwLock;
use raiden::{blockchain::contracts::{self, ContractRegistry}, state_machine::types::ChainID, state_manager::StateManager, storage::Storage};
use rusqlite::Connection;
use slog::{Drain, Logger};
use std::{path::{Path, PathBuf}, sync::Arc};
use web3::types::Address;
use crate::{event_handler::EventHandler, services::{SyncService, TransitionService}, traits::{ToHTTPEndpoint, ToSocketEndpoint}};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone)]
pub struct Config {
	pub chain_id: ChainID,
	pub datadir: PathBuf,
    pub keystore_path: PathBuf,
    pub eth_http_rpc_endpoint: String,
    pub eth_socket_rpc_endpoint: String,
}

impl Config {
	pub fn new(args: ArgMatches) -> Result<Self> {
		// TODO: No unwrap
		let chain_name = args.value_of("chain-id").unwrap();
		let chain_id = chain_name.parse().unwrap();

		let eth_rpc_http_endpoint = args.value_of("eth-rpc-endpoint").unwrap();
		let eth_rpc_socket_endpoint = args.value_of("eth-rpc-socket-endpoint").unwrap();
		let http_endpoint = eth_rpc_http_endpoint.to_http();
		if let Err(e) = http_endpoint {
			return Err(format!("Invalid RPC endpoint: {}", e));
		}

		let socket_endpoint = eth_rpc_socket_endpoint.to_socket();
		if let Err(e) = socket_endpoint {
			return Err(format!("Invalid RPC endpoint: {}", e));
		}

		let keystore_path = Path::new(args.value_of("keystore-path").unwrap());
		let datadir = expanduser::expanduser(args.value_of("datadir").unwrap()).unwrap();

		Ok(Self {
			chain_id,
			datadir,
			keystore_path: keystore_path.to_path_buf(),
			eth_http_rpc_endpoint: http_endpoint.unwrap(),
			eth_socket_rpc_endpoint: socket_endpoint.unwrap(),
		})
	}
}

pub struct RaidenApp {
	config: Config,
	node_address: Address,
	private_key: SecretKey,
	contracts_registry: Arc<RwLock<ContractRegistry>>,
	storage: Arc<Storage>,
	state_manager: Arc<RwLock<StateManager>>,
	logger: Logger,
}

impl RaidenApp {
	pub fn new(config: Config, node_address: Address, private_key: SecretKey) -> Result<Self> {
		let decorator = slog_term::TermDecorator::new().build();
		let drain = slog_term::FullFormat::new(decorator).build().fuse();
		let drain = slog_async::Async::new(drain).build().fuse();

		let logger = slog::Logger::root(drain, o!());

		let contracts_registry = match contracts::ContractRegistry::new(config.chain_id.clone()) {
			Ok(contracts_registry) => contracts_registry,
			Err(e) => {
				return Err(format!("Error creating contracts registry: {}", e));
			},
		};
        let conn = match Connection::open(config.datadir.join("raiden.db")) {
            Ok(conn) => conn,
            Err(e) => {
                return Err(format!("Could not connect to database: {}", e));
            },
        };
		let storage = Arc::new(Storage::new(conn));

        let mut state_manager = StateManager::new(storage.clone());
        if let Err(e) = state_manager.setup() {
            return Err(format!("Could not setup database: {}", e));
        }

		let token_network_registry = contracts_registry.token_network_registry();

		match state_manager.restore_or_init_state(
            config.chain_id.clone(),
            node_address.clone(),
            token_network_registry.address,
            token_network_registry.deploy_block_number,
        ) {
            Ok(_) => {
                debug!(logger, "Restored state");
            }
            Err(_) => {
                debug!(logger, "Initialized node",);
            }
        };

		Ok(Self {
			config,
			node_address,
			private_key,
			contracts_registry: Arc::new(RwLock::new(contracts_registry)),
			storage,
			state_manager: Arc::new(RwLock::new(state_manager)),
			logger,
		})
	}

	pub async fn run(&self) {
		let http = web3::transports::Http::new(&self.config.eth_http_rpc_endpoint).unwrap();
		let web3 = web3::Web3::new(http);
		let latest_block_number = web3.eth().block_number().await.unwrap();

		let event_handler = EventHandler::new(self.state_manager.clone(), self.contracts_registry.clone());
		let transition_service = Arc::new(TransitionService::new(self.state_manager.clone(), event_handler));

		// let mut services = FuturesUnordered::new();
		// services.push(transition_service.start());

		let token_network_registry = self.contracts_registry.read().token_network_registry();
		let sync_start_block_number = match &self.state_manager.read().current_state {
			Some(chain) => chain.block_number,
			None => token_network_registry.deploy_block_number,
		};
		let mut sync_service = SyncService::new(
			web3.clone(),
			self.state_manager.clone(),
			self.contracts_registry.clone(),
			transition_service.clone()
		);
		sync_service.sync(sync_start_block_number, latest_block_number).await;

		// TODO: Now start the block monitor and HTTP service
	}
}
