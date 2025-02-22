// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Gateway chain specification for CLI.

use crate::cli::{
	bridge,
	encode_call::{self, Call, CliEncodeCall},
	encode_message, send_message, CliChain,
};
use codec::Decode;
use frame_support::weights::{GetDispatchInfo, Weight};
use pallet_bridge_dispatch::{CallOrigin, MessagePayload};
use relay_gateway_client::Gateway;
use sp_version::RuntimeVersion;

impl CliEncodeCall for Gateway {
	fn max_extrinsic_size() -> u32 {
		bp_gateway::max_extrinsic_size()
	}

	fn encode_call(call: &Call) -> anyhow::Result<Self::Call> {
		println!("gateway: encode call {:?}", call);
		Ok(match call {
			Call::Raw { data } => Decode::decode(&mut &*data.0)?,
			Call::Remark { remark_payload, .. } => gateway_runtime::Call::System(gateway_runtime::SystemCall::remark(
				remark_payload.as_ref().map(|x| x.0.clone()).unwrap_or_default(),
			)),
			Call::Transfer { recipient, amount } => {
				gateway_runtime::Call::Balances(gateway_runtime::BalancesCall::transfer(recipient.raw_id(), amount.0))
			}
			Call::BridgeSendMessage {
				lane,
				payload,
				fee,
				bridge_instance_index,
			} => match *bridge_instance_index {
				bridge::GATEWAY_TO_CIRCUIT_INDEX => {
					let payload = Decode::decode(&mut &*payload.0)?;
					gateway_runtime::Call::BridgeCircuitMessages(gateway_runtime::MessagesCall::send_message(
						lane.0, payload, fee.0,
					))
				}
				_ => anyhow::bail!(
					"Unsupported target bridge pallet with instance index: {}",
					bridge_instance_index
				),
			},
		})
	}
}

impl CliChain for Gateway {
	const RUNTIME_VERSION: RuntimeVersion = gateway_runtime::VERSION;

	type KeyPair = sp_core::sr25519::Pair;
	type MessagePayload =
		MessagePayload<bp_gateway::AccountId, bp_circuit::AccountSigner, bp_circuit::Signature, Vec<u8>>;

	fn ss58_format() -> u16 {
		gateway_runtime::SS58Prefix::get() as u16
	}

	fn max_extrinsic_weight() -> Weight {
		bp_gateway::max_extrinsic_weight()
	}

	fn encode_message(message: encode_message::MessagePayload) -> Result<Self::MessagePayload, String> {
		match message {
			encode_message::MessagePayload::Raw { data } => MessagePayload::decode(&mut &*data.0)
				.map_err(|e| format!("Failed to decode Gateway's MessagePayload: {:?}", e)),
			encode_message::MessagePayload::Call { mut call, mut sender } => {
				type Source = Gateway;
				type Target = relay_circuit_client::Circuit;

				sender.enforce_chain::<Source>();
				let spec_version = Target::RUNTIME_VERSION.spec_version;
				let origin = CallOrigin::SourceAccount(sender.raw_id());
				encode_call::preprocess_call::<Source, Target>(&mut call, bridge::GATEWAY_TO_CIRCUIT_INDEX);
				let call = Target::encode_call(&call).map_err(|e| e.to_string())?;
				let weight = call.get_dispatch_info().weight;

				Ok(send_message::message_payload(spec_version, weight, origin, &call))
			}
		}
	}
}
