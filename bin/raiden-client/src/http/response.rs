use raiden_state_machine::{
	types::{
		ChannelState,
		ChannelStatus,
		RevealTimeout,
		SettleTimeout,
		TokenAddress,
		TokenAmount,
		TokenNetworkAddress,
	},
	views,
};
use serde::Serialize;
use web3::types::{
	Address,
	U256,
};

#[derive(Serialize)]
pub struct AddressResponse {
	pub our_address: Address,
}

#[derive(Serialize)]
pub struct ChannelResponse {
	channel_identifier: U256,
	token_network_address: TokenNetworkAddress,
	token_address: TokenAddress,
	partner_address: Address,
	settle_timeout: SettleTimeout,
	reveal_timeout: RevealTimeout,
	balance: TokenAmount,
	state: ChannelStatus,
	total_deposit: TokenAmount,
	total_withdraw: TokenAmount,
}

#[derive(Serialize)]
pub struct CreateChannelResponse {
	token_address: TokenAddress,
	partner_address: Address,
	reveal_timeout: RevealTimeout,
	settle_timeout: SettleTimeout,
	total_deposit: TokenAmount,
}

impl From<ChannelState> for ChannelResponse {
	fn from(channel: ChannelState) -> Self {
		ChannelResponse {
			channel_identifier: channel.canonical_identifier.channel_identifier,
			token_network_address: channel.canonical_identifier.token_network_address,
			token_address: channel.token_address,
			partner_address: channel.partner_state.address,
			settle_timeout: channel.settle_timeout,
			reveal_timeout: channel.reveal_timeout,
			total_deposit: channel.our_state.contract_balance,
			total_withdraw: channel.our_state.total_withdraw(),
			state: channel.status(),
			balance: views::channel_balance(&channel.our_state, &channel.partner_state),
		}
	}
}
