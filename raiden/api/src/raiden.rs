use std::{
	path::PathBuf,
	sync::Arc,
};

use parking_lot::RwLock;
use raiden_blockchain::{
	contracts::ContractsManager,
	proxies::{
		Account,
		ProxyManager,
	},
};
use raiden_network_messages::messages::TransportServiceMessage;
use raiden_network_transport::config::TransportConfig;
use raiden_pathfinding::config::PFSConfig;
use raiden_primitives::types::{
	Address,
	ChainID,
};
use raiden_state_machine::types::{
	AddressMetadata,
	MediationFeeConfig,
};
use raiden_storage::state_manager::StateManager;
use tokio::sync::mpsc::UnboundedSender;
use web3::{
	transports::Http,
	Web3,
};

#[derive(Clone)]
pub struct DefaultAddresses {
	pub token_network_registry: Address,
	pub one_to_n: Address,
}

#[derive(Clone)]
pub struct RaidenConfig {
	pub chain_id: ChainID,
	pub account: Account<Http>,
	pub mediation_config: MediationFeeConfig,
	pub pfs_config: PFSConfig,
	pub metadata: AddressMetadata,
	/// Default addresses
	pub addresses: DefaultAddresses,
}

pub struct Raiden {
	pub web3: Web3<Http>,
	/// Raiden Configurations
	pub config: RaidenConfig,
	/// Manager for contracts and deployments
	pub contracts_manager: Arc<ContractsManager>,
	/// Contract proxies manager
	pub proxy_manager: Arc<ProxyManager>,
	/// Manager of the current chain state
	pub state_manager: Arc<RwLock<StateManager>>,
	/// Transport layer
	pub transport: UnboundedSender<TransportServiceMessage>,
}
