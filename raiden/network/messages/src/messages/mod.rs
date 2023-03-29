use raiden_blockchain::keys::PrivateKey;
use raiden_primitives::types::{
	Address,
	AddressMetadata,
	MessageIdentifier,
	QueueIdentifier,
	H256,
};
use serde::{
	Deserialize,
	Serialize,
};
use web3::signing::{
	Key,
	Signature,
	SigningError,
};

mod metadata;
mod synchronization;
mod transfer;
mod withdraw;

pub use metadata::*;
pub use synchronization::*;
pub use transfer::*;
pub use withdraw::*;

enum CmdId {
	Processed = 0,
	SecretRequest = 3,
	Unlock = 4,
	LockedTransfer = 7,
	RevealSecret = 11,
	Delivered = 12,
	LockExpired = 13,
	WithdrawExpired = 17,
}

impl Into<[u8; 1]> for CmdId {
	fn into(self) -> [u8; 1] {
		(self as u8).to_be_bytes()
	}
}

#[derive(Debug)]
pub enum TransportServiceMessage {
	Enqueue((QueueIdentifier, OutgoingMessage)),
	Send(OutgoingMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageInner {
	LockedTransfer(LockedTransfer),
	LockExpired(LockExpired),
	SecretRequest(SecretRequest),
	SecretReveal(SecretReveal),
	Unlock(Unlock),
	WithdrawRequest(WithdrawRequest),
	WithdrawConfirmation(WithdrawConfirmation),
	WithdrawExpired(WithdrawExpired),
	Processed(Processed),
	Delivered(Delivered),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutgoingMessage {
	pub message_identifier: MessageIdentifier,
	pub recipient: Address,
	pub recipient_metadata: AddressMetadata,
	#[serde(flatten)]
	pub inner: MessageInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncomingMessage {
	pub message_identifier: MessageIdentifier,
	pub inner: MessageInner,
}

pub trait SignedMessage {
	fn bytes_to_sign(&self) -> Vec<u8>;
	fn sign(&mut self, key: PrivateKey) -> Result<(), SigningError>;
	fn sign_message(&self, key: PrivateKey) -> Result<Signature, SigningError> {
		let bytes = self.bytes_to_sign();
		key.sign_message(&bytes)
	}
}

pub trait SignedEnvelopeMessage: SignedMessage {
	fn message_hash(&self) -> H256;
}

#[macro_export]
macro_rules! to_message {
	( $send_message_event:ident, $private_key:ident, $message_type:tt ) => {{
		let message_identifier = $send_message_event.inner.message_identifier;
		let recipient = $send_message_event.inner.recipient;
		let address_metadata = $send_message_event
			.inner
			.recipient_metadata
			.clone()
			.expect("Address metadata should be set at this point");
		let mut message: $message_type = $send_message_event.into();
		let _ = message.sign($private_key);
		OutgoingMessage {
			message_identifier,
			recipient,
			recipient_metadata: address_metadata,
			inner: MessageInner::$message_type(message),
		}
	}};
}
