// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A set of constant values used in substrate runtime.

/// Money matters.
pub mod currency {

	use crate::Balance;

	/// Constant values used within the runtime.
	pub const MICROPAN: Balance = 1_000_000;
	pub const MILLIPAN: Balance = 1_000 * MICROPAN;
	pub const CENTS: Balance = 10 * MILLIPAN; // assume this is worth about a cent.
	pub const PAN: Balance = 100 * CENTS;

	pub const INIT_SUPPLY_FACTOR: Balance = 100;
	pub const STORAGE_BYTE_FEE: Balance = 4 * MICROPAN * INIT_SUPPLY_FACTOR;

	/// Charge fee for stored bytes and items.
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}
