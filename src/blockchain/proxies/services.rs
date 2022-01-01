use std::sync::Arc;

use tokio::sync::RwLock;
use web3::{
    contract::{
        Contract,
        Options,
    },
    types::{
        Address,
        BlockId,
        H256,
        U256,
    },
    Transport,
    Web3,
};

use super::ProxyError;

type Result<T> = std::result::Result<T, ProxyError>;

#[derive(Clone)]
pub struct ServiceRegistryProxy<T: Transport> {
    web3: Web3<T>,
    contract: Contract<T>,
    lock: Arc<RwLock<bool>>,
}

impl<T: Transport> ServiceRegistryProxy<T> {
    pub fn new(web3: Web3<T>, contract: Contract<T>) -> Self {
        Self {
            web3,
            contract,
            lock: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn ever_made_deposits(&self, index: u32, block: Option<H256>) -> Result<Address> {
        let block = block.map(|b| BlockId::Hash(b));
        self.contract
            .query(
                "everMadeDeposits",
                (index,),
                None,
                Options::default(),
                block,
            )
            .await
            .map_err(Into::into)
    }

    pub async fn ever_made_deposits_len(&self, block: Option<H256>) -> Result<U256> {
        let block = block.map(|b| BlockId::Hash(b));
        self.contract
            .query("everMadeDepositsLen", (), None, Options::default(), block)
            .await
            .map_err(Into::into)
    }

    pub async fn has_valid_registration(&self, address: Address, block: Option<H256>) -> Result<bool> {
        let block = block.map(|b| BlockId::Hash(b));
        self.contract
            .query("hasValidRegistration", (address,), None, Options::default(), block)
            .await
            .map_err(Into::into)
    }

    pub async fn get_service_url(&self, address: Address, block: Option<H256>) -> Result<String> {
        let block = block.map(|b| BlockId::Hash(b));
        self.contract
            .query("urls", (address,), None, Options::default(), block)
            .await
            .map_err(Into::into)
    }
}