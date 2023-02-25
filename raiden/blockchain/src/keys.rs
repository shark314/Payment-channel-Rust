use ethsign::SecretKey;
use raiden_primitives::types::{
	Address,
	H256,
};
use tiny_keccak::{
	Hasher,
	Keccak,
};
use web3::signing::{
	self,
	Key,
	Signature,
	SigningError,
};

#[derive(Clone)]
pub struct PrivateKey {
	inner: SecretKey,
}

impl PrivateKey {
	pub fn new(inner: SecretKey) -> Self {
		Self { inner }
	}
}

impl Key for PrivateKey {
	fn sign(
		&self,
		message: &[u8],
		chain_id: Option<u64>,
	) -> Result<signing::Signature, SigningError> {
		let signature = self.inner.sign(message).map_err(|_| SigningError::InvalidMessage)?;

		let standard_v = signature.v as u64;
		let v = if let Some(chain_id) = chain_id {
			standard_v + 35 + chain_id * 2
		} else {
			standard_v + 27
		};
		Ok(Signature { r: H256::from(signature.r), s: H256::from(signature.s), v })
	}

	fn sign_message(&self, message: &[u8]) -> Result<Signature, SigningError> {
		let prefix_msg = "\x19Ethereum Signed Message:\n";
		let len_str = message.len().to_string();
		let mut res: Vec<u8> = Vec::new();
		res.append(&mut prefix_msg.as_bytes().to_vec());
		res.append(&mut len_str.as_bytes().to_vec());
		res.append(&mut message.to_vec());

		let mut keccak = Keccak::v256();
		let mut result = [0u8; 32];
		keccak.update(&res);
		keccak.finalize(&mut result);

		let signature = self.inner.sign(&result).map_err(|_| SigningError::InvalidMessage)?;

		Ok(Signature {
			r: H256::from(signature.r),
			s: H256::from(signature.s),
			v: signature.v as u64 + 27,
		})
	}

	fn address(&self) -> Address {
		Address::from(self.inner.public().address())
	}
}
