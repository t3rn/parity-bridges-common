// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! Substrate-to-substrate relay entrypoint.

#![warn(missing_docs)]

use relay_utils::initialize::initialize_relay;

mod cli;
mod finality_pipeline;
mod finality_target;
mod headers_initialize;
mod messages_lane;
mod messages_source;
mod messages_target;

mod gateway_circuit;
mod rialto_millau;

fn main() {
	initialize_relay();
	let command = cli::parse_args();
	let run = command.run();
	let result = async_std::task::block_on(run);
	if let Err(error) = result {
		log::error!(target: "bridge", "Failed to start relay: {}", error);
	}
}
