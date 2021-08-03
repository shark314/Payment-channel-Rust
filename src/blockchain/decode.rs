use std::sync::Arc;

use ethabi::Token;
use web3::types::{
    Address,
    Bytes,
    H256,
    U256,
};

use crate::{
    constants,
    primitives::{
        CanonicalIdentifier,
        RaidenConfig,
        TransactionExecutionStatus,
        TransactionResult,
        U64,
    },
    state_machine::{
        types::{
            ChainState,
            ChannelState,
            ContractReceiveChannelClosed,
            ContractReceiveChannelDeposit,
            ContractReceiveChannelOpened,
            ContractReceiveChannelSettled,
            ContractReceiveChannelWithdraw,
            ContractReceiveTokenNetworkCreated,
            ContractReceiveUpdateTransfer,
            StateChange,
            TokenNetworkState,
            TransactionChannelDeposit,
        },
        views,
    },
};

use super::{
    events::Event,
    proxies::ProxyManager,
};

pub struct EventDecoder {
    proxy_manager: Arc<ProxyManager>,
    config: RaidenConfig,
}

impl EventDecoder {
    pub fn new(config: RaidenConfig, proxy_manager: Arc<ProxyManager>) -> Self {
        Self { proxy_manager, config }
    }

    pub async fn as_state_change(&self, event: Event, chain_state: &ChainState) -> Option<StateChange> {
        match event.name.as_ref() {
            "TokenNetworkCreated" => self.token_network_created(event),
            "ChannelOpened" => self.channel_opened(chain_state, event),
            "ChannelNewDeposit" => self.channel_deposit(chain_state, event),
            "ChannelWithdraw" => self.channel_withdraw(chain_state, event),
            "ChannelClosed" => self.channel_closed(chain_state, event),
            "ChannelSettled" => self.channel_settled(chain_state, event).await,
            "NonClosingBalanceProofUpdated" => self.channel_non_closing_balance_proof_updated(chain_state, event),
            _ => None,
        }
    }

    fn token_network_created(&self, event: Event) -> Option<StateChange> {
        let token_address = match event.data.get("token_address")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let token_network_address = match event.data.get("token_network_address")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let token_network = TokenNetworkState::new(token_network_address, token_address);
        let token_network_registry_address = event.address;
        Some(StateChange::ContractReceiveTokenNetworkCreated(
            ContractReceiveTokenNetworkCreated {
                transaction_hash: Some(event.transaction_hash),
                block_number: event.block_number,
                block_hash: event.block_hash,
                token_network_registry_address,
                token_network,
            },
        ))
    }

    fn channel_opened(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        let participant1 = match event.data.get("participant1")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let participant2 = match event.data.get("participant2")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let settle_timeout = match event.data.get("settle_timeout")? {
            Token::Uint(timeout) => U256::from(timeout.clone().low_u64()),
            _ => U256::zero(),
        };

        let partner_address: Address;
        let our_address = chain_state.our_address;
        if our_address == participant1 {
            partner_address = participant2;
        } else if our_address == participant2 {
            partner_address = participant1;
        } else {
            return None;
        }

        let token_network_address = event.address;
        let _token_network_registry =
            views::get_token_network_registry_by_token_network_address(&chain_state, token_network_address)?;
        let token_network = views::get_token_network_by_address(&chain_state, token_network_address)?;
        let token_address = token_network.token_address;
        let token_network_registry_address = Address::zero();
        let reveal_timeout = U64::from(constants::DEFAULT_REVEAL_TIMEOUT);
        let open_transaction = TransactionExecutionStatus {
            started_block_number: Some(U64::from(0)),
            finished_block_number: Some(event.block_number.clone()),
            result: Some(TransactionResult::Success),
        };
        let channel_state = ChannelState::new(
            CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
            token_address,
            token_network_registry_address,
            our_address,
            partner_address,
            reveal_timeout,
            settle_timeout,
            open_transaction,
            self.config.mediation_config.clone(),
        )
        .ok()?;

        Some(StateChange::ContractReceiveChannelOpened(
            ContractReceiveChannelOpened {
                transaction_hash: Some(event.transaction_hash),
                block_number: event.block_number,
                block_hash: event.block_hash,
                channel_state,
            },
        ))
    }

    fn channel_deposit(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let token_network_address = event.address;
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        let participant = match event.data.get("participant")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let total_deposit = match event.data.get("total_deposit")? {
            Token::Int(total_deposit) => total_deposit.clone(),
            _ => U256::zero(),
        };
        let channel_deposit = ContractReceiveChannelDeposit {
            canonical_identifier: CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
            deposit_transaction: TransactionChannelDeposit {
                participant_address: participant,
                contract_balance: total_deposit,
                deposit_block_number: event.block_number,
            },
            fee_config: self.config.mediation_config.clone(),
        };
        Some(StateChange::ContractReceiveChannelDeposit(channel_deposit))
    }

    fn channel_withdraw(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let token_network_address = event.address;
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        let participant = match event.data.get("participant")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let total_withdraw = match event.data.get("total_withdraw")? {
            Token::Int(total_withdraw) => total_withdraw.clone(),
            _ => U256::zero(),
        };
        let channel_withdraw = ContractReceiveChannelWithdraw {
            canonical_identifier: CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
            participant,
            total_withdraw,
            fee_config: self.config.mediation_config.clone(),
        };
        Some(StateChange::ContractReceiveChannelWithdraw(channel_withdraw))
    }

    fn channel_closed(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        let transaction_from = match event.data.get("closing_participant")? {
            Token::Address(address) => address.clone(),
            _ => Address::zero(),
        };
        let token_network_address = event.address;
        let channel_closed = ContractReceiveChannelClosed {
            transaction_hash: Some(event.transaction_hash),
            block_number: event.block_number,
            block_hash: event.block_hash,
            transaction_from,
            canonical_identifier: CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
        };
        Some(StateChange::ContractReceiveChannelClosed(channel_closed))
    }

    fn channel_non_closing_balance_proof_updated(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let token_network_address = event.address;
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        let nonce = match event.data.get("nonce")? {
            Token::Uint(nonce) => nonce.clone(),
            _ => U256::zero(),
        };
        let update_transfer = ContractReceiveUpdateTransfer {
            canonical_identifier: CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
            nonce,
        };
        Some(StateChange::ContractReceiveUpdateTransfer(update_transfer))
    }

    async fn channel_settled(&self, chain_state: &ChainState, event: Event) -> Option<StateChange> {
        let token_network_address = event.address;
        let channel_identifier = match event.data.get("channel_identifier")? {
            Token::Uint(identifier) => identifier.clone(),
            _ => U256::zero(),
        };
        println!("Channel search");
        let channel_state = views::get_channel_by_canonical_identifier(
            chain_state,
            CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
        )?;
        println!("Channel search finished");

        let (our_onchain_locksroot, partner_onchain_locksroot) = self
            .get_onchain_locksroot(channel_state, chain_state.block_hash)
            .await?;

        let channel_settled = ContractReceiveChannelSettled {
            transaction_hash: Some(event.transaction_hash),
            block_number: event.block_number,
            block_hash: event.block_hash,
            canonical_identifier: CanonicalIdentifier {
                chain_identifier: chain_state.chain_id.clone(),
                token_network_address,
                channel_identifier,
            },
            our_onchain_locksroot,
            partner_onchain_locksroot,
        };
        Some(StateChange::ContractReceiveChannelSettled(channel_settled))
    }

    async fn get_onchain_locksroot(&self, channel_state: &ChannelState, block: H256) -> Option<(Bytes, Bytes)> {
        let payment_channel = self.proxy_manager.payment_channel(&channel_state).await.ok()?;
        let (our_data, partner_data) = payment_channel
            .token_network
            .participants_details(
                channel_state.canonical_identifier.channel_identifier,
                channel_state.our_state.address,
                channel_state.partner_state.address,
                block,
            )
            .await
            .map_err(|e| {
                println!("ERROR: {:?}", e);
                e
            })
            .ok()?;
        Some((our_data.locksroot, partner_data.locksroot))
    }
}
