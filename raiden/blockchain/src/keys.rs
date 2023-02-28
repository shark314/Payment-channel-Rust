use std::fs::File;

pub use ecies::SecpError;
use ethsign::{
	KeyFile,
	Protected,
	SecretKey,
};
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
	RecoveryError,
	Signature,
	SigningError,
};

pub fn hash_data(data: &[u8]) -> [u8; 32] {
	let prefix_msg = "\x19Ethereum Signed Message:\n";
	let len_str = data.len().to_string();
	let mut res: Vec<u8> = Vec::new();
	res.append(&mut prefix_msg.as_bytes().to_vec());
	res.append(&mut len_str.as_bytes().to_vec());
	res.append(&mut data.to_vec());

	let mut keccak = Keccak::v256();
	let mut result = [0u8; 32];
	keccak.update(&res);
	keccak.finalize(&mut result);

	result
}

pub fn recover(data: &[u8], signature: &[u8]) -> Result<Address, RecoveryError> {
	let hashed_data = hash_data(data);
	let recovery_id = signature[64] as i32 - 27;
	let a = signing::recover(data, &signature[..64], recovery_id)?;
	Ok(a)
}

pub fn encrypt(receiver_pub: &[u8], data: &[u8]) -> Result<Vec<u8>, SecpError> {
	ecies::encrypt(receiver_pub, data)
}

pub fn decrypt(private_key: &PrivateKey, data: &[u8]) -> Result<Vec<u8>, SecpError> {
	ecies::decrypt(private_key.plain.as_ref(), data)
}

#[derive(Clone)]
pub struct PrivateKey {
	plain: Protected,
	inner: SecretKey,
}

impl PrivateKey {
	pub fn new(filename: String, password: String) -> Result<Self, String> {
		let file =
			File::open(&filename).map_err(|e| format!("Could not open file: {}", filename))?;

		let key: KeyFile = serde_json::from_reader(file)
			.map_err(|e| format!("Could not read file: {}", filename))?;

		let plain = key
			.crypto
			.decrypt(&password.into())
			.map_err(|e| format!("Could not decrypt private key file: {}", filename))?;

		let inner = SecretKey::from_raw(&plain)
			.map_err(|e| format!("Could not generate secret key from file: {}", filename))?;

		Ok(Self { plain: plain.into(), inner })
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
		let data_hash = hash_data(message);
		let signature = self.inner.sign(&data_hash).map_err(|_| SigningError::InvalidMessage)?;

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