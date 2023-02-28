use raiden_primitives::types::{
	BlockId,
	H256,
	U256,
	U64,
};
use web3::{
	contract::{
		Contract,
		Options,
	},
	Transport,
};

use super::ProxyError;

type Result<T> = std::result::Result<T, ProxyError>;

#[derive(Clone)]
pub struct SecretRegistryProxy<T: Transport> {
	contract: Contract<T>,
}

impl<T: Transport> SecretRegistryProxy<T> {
	pub fn new(contract: Contract<T>) -> Self {
		Self { contract }
	}

	pub async fn get_secret_registration_block_by_secrethash(
		&self,
		secrethash: H256,
		block: Option<H256>,
	) -> Result<Option<U64>> {
		let block = block.map(|b| BlockId::Hash(b));
		self.contract
			.query("getSecretRevealBlockHeight", (secrethash,), None, Options::default(), block)
			.await
			.map(|b: U256| {
				let b = b.as_u64();
				Some(b.into())
			})
			.map_err(Into::into)
	}

	pub async fn is_secret_registered(
		&self,
		secrethash: H256,
		block: Option<H256>,
	) -> Result<bool> {
		let block = self.get_secret_registration_block_by_secrethash(secrethash, block).await?;
		Ok(block.is_some())
	}
}