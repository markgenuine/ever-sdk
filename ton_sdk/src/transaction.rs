/*
* Copyright 2018-2021 TON Labs LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::SdkError;
use crate::json_helper;
use crate::types::grams_to_u64;
use crate::types::StringId;
use crate::{Message, MessageId};

use ton_block::{
    AccStatusChange, AddSub, ComputeSkipReason, GetRepresentationHash, TrComputePhase,
    TransactionDescr, TransactionProcessingStatus,
};
use ton_types::Result;

use std::convert::TryFrom;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct ComputePhase {
    #[serde(deserialize_with = "json_helper::deserialize_skipped_reason")]
    pub skipped_reason: Option<ComputeSkipReason>,
    pub exit_code: Option<i32>,
    pub exit_arg: Option<i32>,
    pub success: Option<bool>,
    #[serde(with = "json_helper::uint")]
    pub gas_fees: u64,
    #[serde(with = "json_helper::uint")]
    pub gas_used: u64,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct StoragePhase {
    #[serde(deserialize_with = "json_helper::deserialize_acc_state_change")]
    pub status_change: AccStatusChange,
    #[serde(with = "json_helper::uint")]
    pub storage_fees_collected: u64,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct ActionPhase {
    pub success: bool,
    pub valid: bool,
    pub no_funds: bool,
    pub result_code: i32,
    #[serde(with = "json_helper::uint")]
    pub total_fwd_fees: u64,
    #[serde(with = "json_helper::uint")]
    pub total_action_fees: u64,
}

pub type TransactionId = StringId;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Transaction {
    pub id: TransactionId,
    #[serde(deserialize_with = "json_helper::deserialize_tr_state")]
    pub status: TransactionProcessingStatus,
    pub now: u32,
    pub in_msg: Option<MessageId>,
    pub out_msgs: Vec<MessageId>,
    pub out_messages: Vec<Message>,
    pub aborted: bool,
    pub compute: ComputePhase,
    pub storage: Option<StoragePhase>,
    pub action: Option<ActionPhase>,
    #[serde(with = "json_helper::uint")]
    pub total_fees: u64,

    #[serde(with = "json_helper::uint")]
    ext_in_msg_fee: u64,
    #[serde(with = "json_helper::uint")]
    account_fees: u64,
}

impl TryFrom<&ton_block::Transaction> for Transaction {
    type Error = failure::Error;
    fn try_from(transaction: &ton_block::Transaction) -> Result<Self> {
        let descr = if let TransactionDescr::Ordinary(descr) = transaction.read_description()? {
            descr
        } else {
            return Err(SdkError::InvalidData {
                msg: "Invalid transaction type".to_owned(),
            }
            .into());
        };

        let account_fees = transaction.total_fees().grams.clone();
        let mut in_msg_fee = account_fees.clone();

        let storage_phase = if let Some(phase) = descr.storage_ph {
            in_msg_fee.sub(&phase.storage_fees_collected)?;
            Some(StoragePhase {
                status_change: phase.status_change,
                storage_fees_collected: grams_to_u64(&phase.storage_fees_collected)?,
            })
        } else {
            None
        };

        let compute_phase = match descr.compute_ph {
            TrComputePhase::Skipped(ph) => ComputePhase {
                skipped_reason: Some(ph.reason),
                exit_code: None,
                exit_arg: None,
                success: None,
                gas_fees: 0,
                gas_used: 0,
            },
            TrComputePhase::Vm(ph) => {
                in_msg_fee.sub(&ph.gas_fees)?;
                ComputePhase {
                    skipped_reason: None,
                    exit_code: Some(ph.exit_code),
                    exit_arg: ph.exit_arg,
                    success: Some(ph.success),
                    gas_fees: grams_to_u64(&ph.gas_fees)?,
                    gas_used: ph.gas_used.as_u64(),
                }
            }
        };

        let action_phase = if let Some(phase) = descr.action {
            if let Some(fees) = phase.total_action_fees.as_ref() {
                in_msg_fee.sub(fees)?;
            }

            Some(ActionPhase {
                success: phase.success,
                valid: phase.valid,
                no_funds: phase.no_funds,
                result_code: phase.result_code,
                total_fwd_fees: grams_to_u64(&phase.total_fwd_fees.unwrap_or_default())?,
                total_action_fees: grams_to_u64(&phase.total_action_fees.unwrap_or_default())?,
            })
        } else {
            None
        };

        let in_msg = transaction.in_msg.as_ref().map(|msg| msg.hash().into());
        let mut out_msgs = vec![];
        transaction.out_msgs.iterate_slices(|slice| {
            if let Ok(cell) = slice.reference(0) {
                out_msgs.push(cell.repr_hash().into());
            }
            Ok(true)
        })?;
        let mut out_messages = vec![];
        transaction.out_msgs.iterate(|msg| {
            out_messages.push(Message::with_msg(&msg.0)?);
            Ok(true)
        })?;

        let is_ext_in = if let Some(msg) = transaction.in_msg.as_ref() {
            msg.read_struct()?.is_inbound_external()
        } else {
            false
        };
        Ok(Transaction {
            id: transaction.hash()?.into(),
            status: TransactionProcessingStatus::Finalized,
            now: transaction.now(),
            in_msg,
            out_msgs,
            out_messages: out_messages,
            aborted: descr.aborted,
            total_fees: grams_to_u64(&transaction.total_fees().grams)?,
            storage: storage_phase,
            compute: compute_phase,
            action: action_phase,
            ext_in_msg_fee: if is_ext_in {
                grams_to_u64(&in_msg_fee)?
            } else {
                0
            },
            account_fees: grams_to_u64(&account_fees)?,
        })
    }
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone, Default)]
pub struct TransactionFees {
    /// Deprecated. Left for backward compatibility. 
    /// Does not participate in account transaction fees calculation.
    pub in_msg_fwd_fee: u64,
    /// Fee for account storage
    pub storage_fee: u64,
    /// Fee for processing 
    pub gas_fee: u64,
    /// Deprecated. Contains the same data as total_fwd_fees field. Deprecated because of
    /// its confusing name, that is not the same with GraphQL API Transaction type's field.
    pub out_msgs_fwd_fee: u64,
    /// Deprecated. This is the field that is named as `total_fees` in GraphQL API Transaction type.
    /// `total_account_fees` name is misleading, because it does not mean account fees, instead it means 
    /// validators total fees received for the transaction execution. It does not include some forward fees that account
    /// actually pays now, but validators will receive later during value delivery to another account (not even in the receiving
    /// transaction). 
    /// Because of all of this, this field is not interesting for those who wants to understand
    /// the real account fees, this is why it is deprecated and left for backward compatibility.
    pub total_account_fees: u64,
    /// Deprecated because it means total value sent in the transaction, which does not relate to any fees.
    pub total_output: u64,
    /// Fee for inbound external message import.
    pub ext_in_msg_fee: u64,
    /// Total fees the account pays for message forwarding
    pub total_fwd_fees: u64,
    /// Total account fees for the transaction execution.
    /// Compounds of storage_fee + gas_fee + ext_in_msg_fee + total_fwd_fees
    pub account_fees: u64,
}

// The struct represents performed transaction and allows to access their properties.
impl Transaction {
    // Returns transaction's processing status
    pub fn status(&self) -> TransactionProcessingStatus {
        self.status
    }

    // Returns id of transaction's input message if it exists
    pub fn in_message_id(&self) -> Option<MessageId> {
        self.in_msg.clone()
    }

    // Returns id of transaction's out messages if it exists
    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.out_msgs
    }

    // Returns message's identifier
    pub fn id(&self) -> TransactionId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.id.clone()
    }

    // Returns `aborted` flag
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    pub fn calc_fees(&self) -> TransactionFees {
        let mut fees = TransactionFees::default();

        fees.gas_fee = self.compute.gas_fees;

        if let Some(storage) = &self.storage {
            fees.storage_fee = storage.storage_fees_collected;
        }
        let mut total_action_fees = 0;
        if let Some(action) = &self.action {
            fees.out_msgs_fwd_fee = action.total_fwd_fees;
            fees.total_fwd_fees = action.total_fwd_fees;
            total_action_fees = action.total_action_fees;
        }
        // `transaction.total_fees` is calculated as
        // `transaction.total_fees = inbound_fwd_fees + storage_fees + gas_fees + total_action_fees`
        // but this total_fees is fees collected the validators, not the all fees taken from account
        // because total_action_fees contains only part of all forward fees
        // to get all fees paid by account we need exchange `total_action_fees part` to `out_msgs_fwd_fee`
        let total_account_fees =
            self.total_fees as i128 - total_action_fees as i128 + fees.out_msgs_fwd_fee as i128;
        fees.total_account_fees = if total_account_fees > 0 {
            total_account_fees as u64
        } else {
            0
        };
        // inbound_fwd_fees is not represented in transaction fields so need to calculate it
        let in_msg_fwd_fee = fees.total_account_fees as i128
            - fees.storage_fee as i128
            - fees.gas_fee as i128
            - fees.out_msgs_fwd_fee as i128;
        fees.in_msg_fwd_fee = if in_msg_fwd_fee > 0 {
            in_msg_fwd_fee as u64
        } else {
            0
        };

        let total_output = self
            .out_messages
            .iter()
            .fold(0u128, |acc, msg| acc + msg.value as u128);
        fees.total_output = if total_output <= std::u64::MAX as u128 {
            total_output as u64
        } else {
            0
        };

        fees.ext_in_msg_fee = self.ext_in_msg_fee;
        fees.account_fees = self.account_fees;
        fees
    }
}
