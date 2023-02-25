use raiden_blockchain::keys::PrivateKey;
use raiden_primitives::{
	traits::ToBytes,
	types::{
		Address,
		ChainID,
		TokenNetworkAddress,
		U256,
		U64,
	},
};
use raiden_state_machine::types::{
	SendWithdrawConfirmation,
	SendWithdrawExpired,
	SendWithdrawRequest,
};
use serde::{
	Deserialize,
	Serialize,
};
use web3::signing::SigningError;

use super::{
	CmdId,
	MessageTypeId,
	SignedMessage,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawRequest {
	pub message_identifier: u32,
	pub chain_id: ChainID,
	pub token_network_address: TokenNetworkAddress,
	pub channel_identifier: U256,
	pub participant: Address,
	pub total_withdraw: U256,
	pub expiration: U64,
	pub nonce: U256,
	pub signature: Vec<u8>,
	pub coop_settle: bool,
}

impl From<SendWithdrawRequest> for WithdrawRequest {
	fn from(event: SendWithdrawRequest) -> Self {
		Self {
			message_identifier: event.message_identifier,
			chain_id: event.canonical_identifier.chain_identifier.clone(),
			token_network_address: event.canonical_identifier.token_network_address,
			channel_identifier: event.canonical_identifier.channel_identifier,
			participant: event.participant,
			total_withdraw: event.total_withdraw,
			expiration: event.expiration,
			nonce: event.nonce,
			signature: vec![],
			coop_settle: event.coop_settle,
		}
	}
}

impl SignedMessage for WithdrawRequest {
	fn bytes(&self) -> Vec<u8> {
		let chain_id: Vec<u8> = self.chain_id.into();
		let cmd_id: [u8; 1] = CmdId::WithdrawRequest.into();
		let message_type_id: [u8; 1] = MessageTypeId::Withdraw.into();

		let mut nonce = [0u8; 32];
		self.nonce.to_big_endian(&mut nonce);

		let mut channel_identifier = [0u8; 32];
		self.channel_identifier.to_big_endian(&mut channel_identifier);

		let mut total_withdraw = [0u8; 32];
		self.total_withdraw.to_big_endian(&mut total_withdraw);

		let mut expiration = [0u8; 32];
		self.expiration.to_big_endian(&mut expiration);

		let mut bytes = vec![];
		bytes.extend_from_slice(&cmd_id);
		bytes.extend_from_slice(&nonce);
		bytes.extend_from_slice(&self.message_identifier.to_be_bytes());
		bytes.extend_from_slice(self.token_network_address.as_bytes());
		bytes.extend_from_slice(&chain_id);
		bytes.extend_from_slice(&message_type_id);
		bytes.extend_from_slice(&channel_identifier);
		bytes.extend_from_slice(self.participant.as_bytes());
		bytes.extend_from_slice(&total_withdraw);
		bytes.extend_from_slice(&expiration);
		bytes
	}

	fn sign(&mut self, key: PrivateKey) -> Result<(), SigningError> {
		self.signature = self.sign_message(key)?.as_vec();
		Ok(())
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawConfirmation {
	pub message_identifier: u32,
	pub chain_id: ChainID,
	pub token_network_address: TokenNetworkAddress,
	pub channel_identifier: U256,
	pub participant: Address,
	pub total_withdraw: U256,
	pub expiration: U64,
	pub nonce: U256,
	pub signature: Vec<u8>,
}

impl From<SendWithdrawConfirmation> for WithdrawConfirmation {
	fn from(event: SendWithdrawConfirmation) -> Self {
		Self {
			message_identifier: event.message_identifier,
			chain_id: event.canonical_identifier.chain_identifier.clone(),
			token_network_address: event.canonical_identifier.token_network_address,
			channel_identifier: event.canonical_identifier.channel_identifier,
			participant: event.participant,
			total_withdraw: event.total_withdraw,
			expiration: event.expiration,
			nonce: event.nonce,
			signature: vec![],
		}
	}
}

impl SignedMessage for WithdrawConfirmation {
	fn bytes(&self) -> Vec<u8> {
		let chain_id: Vec<u8> = self.chain_id.into();
		let cmd_id: [u8; 1] = CmdId::WithdrawConfirmation.into();
		let message_type_id: [u8; 1] = MessageTypeId::Withdraw.into();

		let mut nonce = [0u8; 32];
		self.nonce.to_big_endian(&mut nonce);

		let mut channel_identifier = [0u8; 32];
		self.channel_identifier.to_big_endian(&mut channel_identifier);

		let mut total_withdraw = [0u8; 32];
		self.total_withdraw.to_big_endian(&mut total_withdraw);

		let mut expiration = [0u8; 32];
		self.expiration.to_big_endian(&mut expiration);

		let mut bytes = vec![];
		bytes.extend_from_slice(&cmd_id);
		bytes.extend_from_slice(&nonce);
		bytes.extend_from_slice(&self.message_identifier.to_be_bytes());
		bytes.extend_from_slice(self.token_network_address.as_bytes());
		bytes.extend_from_slice(&chain_id);
		bytes.extend_from_slice(&message_type_id);
		bytes.extend_from_slice(&channel_identifier);
		bytes.extend_from_slice(self.participant.as_bytes());
		bytes.extend_from_slice(&total_withdraw);
		bytes.extend_from_slice(&expiration);
		bytes
	}

	fn sign(&mut self, key: PrivateKey) -> Result<(), SigningError> {
		self.signature = self.sign_message(key)?.as_vec();
		Ok(())
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawExpired {
	pub message_identifier: u32,
	pub chain_id: ChainID,
	pub token_network_address: TokenNetworkAddress,
	pub channel_identifier: U256,
	pub participant: Address,
	pub total_withdraw: U256,
	pub expiration: U64,
	pub nonce: U256,
	pub signature: Vec<u8>,
}

impl From<SendWithdrawExpired> for WithdrawExpired {
	fn from(event: SendWithdrawExpired) -> Self {
		Self {
			message_identifier: event.message_identifier,
			chain_id: event.canonical_identifier.chain_identifier.clone(),
			token_network_address: event.canonical_identifier.token_network_address,
			channel_identifier: event.canonical_identifier.channel_identifier,
			participant: event.participant,
			total_withdraw: event.total_withdraw,
			expiration: event.expiration,
			nonce: event.nonce,
			signature: vec![],
		}
	}
}

impl SignedMessage for WithdrawExpired {
	fn bytes(&self) -> Vec<u8> {
		let chain_id: Vec<u8> = self.chain_id.into();
		let cmd_id: [u8; 1] = CmdId::WithdrawExpired.into();
		let message_type_id: [u8; 1] = MessageTypeId::Withdraw.into();

		let mut nonce = [0u8; 32];
		self.nonce.to_big_endian(&mut nonce);

		let mut channel_identifier = [0u8; 32];
		self.channel_identifier.to_big_endian(&mut channel_identifier);

		let mut total_withdraw = [0u8; 32];
		self.total_withdraw.to_big_endian(&mut total_withdraw);

		let mut expiration = [0u8; 32];
		self.expiration.to_big_endian(&mut expiration);

		let mut bytes = vec![];
		bytes.extend_from_slice(&cmd_id);
		bytes.extend_from_slice(&nonce);
		bytes.extend_from_slice(&self.message_identifier.to_be_bytes());
		bytes.extend_from_slice(self.token_network_address.as_bytes());
		bytes.extend_from_slice(&chain_id);
		bytes.extend_from_slice(&message_type_id);
		bytes.extend_from_slice(&channel_identifier);
		bytes.extend_from_slice(self.participant.as_bytes());
		bytes.extend_from_slice(&total_withdraw);
		bytes.extend_from_slice(&expiration);
		bytes
	}

	fn sign(&mut self, key: PrivateKey) -> Result<(), SigningError> {
		self.signature = self.sign_message(key)?.as_vec();
		Ok(())
	}
}
