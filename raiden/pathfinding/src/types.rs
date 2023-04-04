use raiden_blockchain::keys::PrivateKey;
use raiden_primitives::{
	packing::pack_one_to_n_iou,
	serializers::{
		to_checksummed_str,
		u256_to_str,
	},
	traits::ToBytes,
	types::{
		Address,
		BlockExpiration,
		Bytes,
		ChainID,
		OneToNAddress,
		TokenAmount,
	},
};
use serde::{
	Deserialize,
	Serialize,
};
use web3::signing::{
	self,
	Key,
};

#[derive(Copy, Clone, PartialEq)]
pub enum RoutingMode {
	PFS,
	Private,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IOU {
	#[serde(serialize_with = "to_checksummed_str")]
	pub sender: Address,
	#[serde(serialize_with = "to_checksummed_str")]
	pub receiver: Address,
	#[serde(serialize_with = "to_checksummed_str")]
	pub one_to_n_address: OneToNAddress,
	#[serde(serialize_with = "u256_to_str")]
	pub amount: TokenAmount,
	pub expiration_block: BlockExpiration,
	pub chain_id: ChainID,
	pub signature: Option<Bytes>,
}

impl IOU {
	pub fn sign(&mut self, private_key: PrivateKey) -> Result<(), signing::SigningError> {
		let data = pack_one_to_n_iou(
			self.one_to_n_address,
			self.sender,
			self.receiver,
			self.amount,
			self.expiration_block,
			self.chain_id,
		);
		let signature = private_key.sign_message(&data.0)?;
		self.signature = Some(Bytes(signature.to_bytes()));
		Ok(())
	}
}
