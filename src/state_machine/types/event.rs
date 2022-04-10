use derive_more::Deref;
use serde::{
    Deserialize,
    Serialize,
};
use web3::types::Address;

use crate::primitives::{
    AddressMetadata,
    BlockExpiration,
    BlockHash,
    CanonicalIdentifier,
    MessageIdentifier,
    Nonce,
    PaymentIdentifier,
    QueueIdentifier,
    RevealTimeout,
    Secret,
    SecretHash,
    Signature,
    TokenAddress,
    TokenAmount,
    TokenNetworkAddress,
    TokenNetworkRegistryAddress,
};

use super::{
    BalanceProofState,
    LockedTransferState,
};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    ContractSendChannelClose(ContractSendChannelClose),
    ContractSendChannelWithdraw(ContractSendChannelWithdraw),
    ContractSendChannelSettle(ContractSendChannelSettle),
    ContractSendChannelUpdateTransfer(ContractSendChannelUpdateTransfer),
    ContractSendChannelBatchUnlock(ContractSendChannelBatchUnlock),
    ContractSendSecretReveal(ContractSendSecretReveal),
    PaymentReceivedSuccess(PaymentReceivedSuccess),
    PaymentSentSuccess(PaymentSentSuccess),
    SendWithdrawExpired(SendWithdrawExpired),
    SendWithdrawRequest(SendWithdrawRequest),
    SendLockedTransfer(SendLockedTransfer),
    SendLockExpired(SendLockExpired),
    SendSecretRequest(SendSecretRequest),
    SendSecretReveal(SendSecretReveal),
    SendUnlock(SendUnlock),
    SendProcessed(SendProcessed),
    UnlockSuccess(UnlockSuccess),
    UnlockClaimSuccess(UnlockClaimSuccess),
    UpdatedServicesAddresses(UpdatedServicesAddresses),
    ErrorUnlockClaimFailed(ErrorUnlockClaimFailed),
    ErrorInvalidActionWithdraw(ErrorInvalidActionWithdraw),
    ErrorInvalidActionSetRevealTimeout(ErrorInvalidActionSetRevealTimeout),
    ErrorPaymentSentFailed(ErrorPaymentSentFailed),
    ErrorRouteFailed(ErrorRouteFailed),
    ErrorUnlockFailed(ErrorUnlockFailed),
    ErrorInvalidSecretRequest(ErrorInvalidSecretRequest),
    ErrorInvalidReceivedLockedTransfer(ErrorInvalidReceivedLockedTransfer),
    ErrorInvalidReceivedLockExpired(ErrorInvalidReceivedLockExpired),
    ErrorInvalidReceivedTransferRefund(ErrorInvalidReceivedTransferRefund),
    ErrorInvalidReceivedUnlock(ErrorInvalidReceivedUnlock),
    ErrorUnexpectedReveal(ErrorUnexpectedReveal),
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum SendMessageEvent {
    SendLockExpired(SendLockExpired),
    SendLockedTransfer(SendLockedTransfer),
    SendSecretReveal(SendSecretReveal),
    SendSecretRequest(SendSecretRequest),
    SendUnlock(SendUnlock),
    SendWithdrawRequest(SendWithdrawRequest),
    SendWithdrawConfirmation(SendWithdrawConfirmation),
    SendWithdrawExpired(SendWithdrawExpired),
    SendProcessed(SendProcessed),
}

impl TryFrom<Event> for SendMessageEvent {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(match event {
            Event::SendWithdrawExpired(inner) => SendMessageEvent::SendWithdrawExpired(inner),
            Event::SendWithdrawRequest(inner) => SendMessageEvent::SendWithdrawRequest(inner),
            Event::SendLockedTransfer(inner) => SendMessageEvent::SendLockedTransfer(inner),
            Event::SendLockExpired(inner) => SendMessageEvent::SendLockExpired(inner),
            Event::SendSecretRequest(inner) => SendMessageEvent::SendSecretRequest(inner),
            Event::SendSecretReveal(inner) => SendMessageEvent::SendSecretReveal(inner),
            Event::SendUnlock(inner) => SendMessageEvent::SendUnlock(inner),
            Event::SendProcessed(inner) => SendMessageEvent::SendProcessed(inner),
            _ => return Err(())
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ContractSendEvent {
    ContractSendChannelClose(ContractSendChannelClose),
    ContractSendChannelWithdraw(ContractSendChannelWithdraw),
    ContractSendChannelSettle(ContractSendChannelSettle),
    ContractSendChannelUpdateTransfer(ContractSendChannelUpdateTransfer),
    ContractSendChannelBatchUnlock(ContractSendChannelBatchUnlock),
    ContractSendSecretReveal(ContractSendSecretReveal),
}

impl TryFrom<Event> for ContractSendEvent {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(match event {
            Event::ContractSendChannelClose(inner) => ContractSendEvent::ContractSendChannelClose(inner),
            Event::ContractSendChannelWithdraw(inner) => ContractSendEvent::ContractSendChannelWithdraw(inner),
            Event::ContractSendChannelSettle(inner) => ContractSendEvent::ContractSendChannelSettle(inner),
            Event::ContractSendChannelUpdateTransfer(inner) => ContractSendEvent::ContractSendChannelUpdateTransfer(inner),
            Event::ContractSendChannelBatchUnlock(inner) => ContractSendEvent::ContractSendChannelBatchUnlock(inner),
            Event::ContractSendSecretReveal(inner) => ContractSendEvent::ContractSendSecretReveal(inner),
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct SendMessageEventInner {
    pub recipient: Address,
    pub recipient_metadata: Option<AddressMetadata>,
    pub canonical_identifier: CanonicalIdentifier,
    pub message_identifier: MessageIdentifier,
}

impl SendMessageEventInner {
    pub fn queue_identifier(&self) -> QueueIdentifier {
        QueueIdentifier {
            recipient: self.recipient.clone(),
            canonical_identifier: self.canonical_identifier.clone(),
        }
    }
}

#[cfg(test)]
impl PartialEq for SendMessageEventInner {
    fn eq(&self, other: &Self) -> bool {
        self.recipient == other.recipient
            && self.recipient_metadata == other.recipient_metadata
            && self.canonical_identifier == other.canonical_identifier
    }
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendWithdrawRequest {
    #[deref]
    pub inner: SendMessageEventInner,
    pub participant: Address,
    pub expiration: BlockExpiration,
    pub nonce: Nonce,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendWithdrawConfirmation {
    #[deref]
    pub inner: SendMessageEventInner,
    pub participant: Address,
    pub total_withdraw: TokenAmount,
    pub nonce: Nonce,
    pub expiration: BlockExpiration,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendWithdrawExpired {
    #[deref]
    pub inner: SendMessageEventInner,
    pub participant: Address,
    pub total_withdraw: TokenAmount,
    pub nonce: Nonce,
    pub expiration: BlockExpiration,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendLockedTransfer {
    #[deref]
    pub inner: SendMessageEventInner,
    pub transfer: LockedTransferState,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendSecretRequest {
    #[deref]
    pub inner: SendMessageEventInner,
    pub payment_identifier: PaymentIdentifier,
    pub amount: TokenAmount,
    pub expiration: BlockExpiration,
    pub secrethash: SecretHash,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendSecretReveal {
    #[deref]
    pub inner: SendMessageEventInner,
    pub secret: Secret,
    pub secrethash: SecretHash,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendLockExpired {
    #[deref]
    pub inner: SendMessageEventInner,
    pub balance_proof: BalanceProofState,
    pub secrethash: SecretHash,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendUnlock {
    #[deref]
    pub inner: SendMessageEventInner,
    pub payment_identifier: PaymentIdentifier,
    pub token_address: TokenAddress,
    pub balance_proof: BalanceProofState,
    pub secret: Secret,
    pub secrethash: SecretHash,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendProcessed {
    #[deref]
    pub inner: SendMessageEventInner,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PaymentReceivedSuccess {
    pub token_network_registry_address: TokenNetworkRegistryAddress,
    pub token_network_address: TokenNetworkAddress,
    pub identifier: PaymentIdentifier,
    pub amount: TokenAmount,
    pub initiator: Address,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PaymentSentSuccess {
    pub token_network_registry_address: TokenNetworkRegistryAddress,
    pub token_network_address: TokenNetworkAddress,
    pub identifier: PaymentIdentifier,
    pub amount: TokenAmount,
    pub target: Address,
    pub secret: Secret,
    pub route: Vec<Address>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UnlockSuccess {
    pub identifier: PaymentIdentifier,
    pub secrethash: SecretHash,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UnlockClaimSuccess {
    pub identifier: PaymentIdentifier,
    pub secrethash: SecretHash,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendEventInner {
    pub triggered_by_blockhash: BlockHash,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendChannelClose {
    #[deref]
    pub inner: ContractSendEventInner,
    pub canonical_identifier: CanonicalIdentifier,
    pub balance_proof: BalanceProofState,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendChannelWithdraw {
    #[deref]
    pub inner: ContractSendEventInner,
    pub canonical_identifier: CanonicalIdentifier,
    pub total_withdraw: TokenAmount,
    pub expiration: BlockExpiration,
    pub partner_signature: Signature,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendChannelSettle {
    #[deref]
    pub inner: ContractSendEventInner,
    pub canonical_identifier: CanonicalIdentifier,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendChannelUpdateTransfer {
    #[deref]
    pub inner: ContractSendEventInner,
    pub expiration: BlockExpiration,
    pub balance_proof: BalanceProofState,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendChannelBatchUnlock {
    #[deref]
    pub inner: ContractSendEventInner,
    pub canonical_identifier: CanonicalIdentifier,
    pub sender: Address,
}

#[derive(Deref, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ContractSendSecretReveal {
    #[deref]
    pub inner: ContractSendEventInner,
    pub expiration: BlockExpiration,
    pub secret: Secret,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidActionWithdraw {
    pub attemped_withdraw: TokenAmount,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidActionSetRevealTimeout {
    pub reveal_timeout: RevealTimeout,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorPaymentSentFailed {
    pub token_network_registry_address: TokenNetworkRegistryAddress,
    pub token_network_address: TokenNetworkAddress,
    pub identifier: PaymentIdentifier,
    pub target: Address,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorUnlockFailed {
    pub identifier: PaymentIdentifier,
    pub secrethash: SecretHash,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorRouteFailed {
    pub secrethash: SecretHash,
    pub route: Vec<Address>,
    pub token_network_address: TokenNetworkAddress,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidSecretRequest {
    pub payment_identifier: PaymentIdentifier,
    pub intended_amount: TokenAmount,
    pub actual_amount: TokenAmount,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidReceivedLockedTransfer {
    pub payment_identifier: PaymentIdentifier,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidReceivedLockExpired {
    pub secrethash: SecretHash,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidReceivedTransferRefund {
    pub payment_identifier: PaymentIdentifier,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorInvalidReceivedUnlock {
    pub secrethash: SecretHash,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorUnlockClaimFailed {
    pub identifier: PaymentIdentifier,
    pub secrethash: SecretHash,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ErrorUnexpectedReveal {
    pub secrethash: SecretHash,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpdatedServicesAddresses {
    pub service_address: Address,
    pub validity: u32,
}
