// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use crate::mock::{
	evm_test_context, precompile_address, ExtBuilder, events,
	Precompiles, TestAccount::*
};
use crate::{convert_encoded_multilocation_into_multilocation, convert_encoded_junction_to_junction};
use crate::{Action, PrecompileOutput};
use num_enum::TryFromPrimitive;
use pallet_evm::{ExitSucceed, PrecompileSet};
use precompile_utils::{error, Address, EvmDataWriter, Bytes};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};
use xcm::v0::{Junction, MultiLocation, NetworkId};
use sp_std::convert::TryInto;
#[test]
fn test_selector_enum() {
	let mut buffer = [0u8; 4];
	buffer.copy_from_slice(&Keccak256::digest(b"transfer(address, u256, bytes[], u64)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::Transfer,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"transfer_multiasset(bytes[], u256, bytes[], u64)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::TransferMultiAsset,
	);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(error("tried to parse selector out of bounds")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}


#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are such a selector does not exist
		let expected_result = Some(Err(error("unknown selector")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn multilocation_decoder_works() {
	ExtBuilder::default().build().execute_with(|| {

		let mut x1_multi_location = Vec::<Bytes>::new();
		x1_multi_location.push(vec![0u8].into());

		assert_eq!(
			convert_encoded_multilocation_into_multilocation(
				x1_multi_location.clone()
			),
			Ok(MultiLocation::X1(Junction::Parent))
		);

		x1_multi_location.push(vec![0u8].into());

		assert_eq!(
			convert_encoded_multilocation_into_multilocation(
				x1_multi_location.clone()
			),
			Ok(MultiLocation::X2(Junction::Parent, Junction::Parent))
		);

		x1_multi_location.push(vec![0u8].into());

		assert_eq!(
			convert_encoded_multilocation_into_multilocation(
				x1_multi_location.clone()
			),
			Ok(MultiLocation::X3(Junction::Parent, Junction::Parent, Junction::Parent)),
		);
	});
}

#[test]
fn junction_decoder_works() {
	ExtBuilder::default().build().execute_with(|| {

		let parent_junction = vec![0u8];
		assert_eq!(
			(
				convert_encoded_junction_to_junction(parent_junction)
			),
			Ok(Junction::Parent)
		);

		let parachain_junction = vec![1u8, 0u8, 0u8, 0u8, 0u8];
		assert_eq!(
			(
				convert_encoded_junction_to_junction(parachain_junction)
			),
			Ok(Junction::Parachain(0))
		);

		let mut account_id_32 = vec![2u8];
		account_id_32.extend_from_slice(&[1u8;32]);
		account_id_32.extend_from_slice(&[0u8]);

		assert_eq!(
			(
				convert_encoded_junction_to_junction(account_id_32)
			),
			Ok(Junction::AccountId32 {
				network: NetworkId::Any,
				id: [1u8;32],
			 })
		);

		let mut account_index_64 = vec![3u8];
		account_index_64.extend_from_slice(&[1u8;8]);
		account_index_64.extend_from_slice(&[0u8]);

		assert_eq!(
			(
				convert_encoded_junction_to_junction(account_index_64)
			),
			Ok(Junction::AccountIndex64 {
				network: NetworkId::Any,
				index: u64::from_be_bytes([1u8;8]),
			 })
		);

		let mut account_key_20 = vec![4u8];
		account_key_20.extend_from_slice(H160::from(Alice).as_bytes());
		account_key_20.extend_from_slice(&[0u8]);

		assert_eq!(
			(
				convert_encoded_junction_to_junction(account_key_20)
			),
			Ok(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: H160::from(Alice).as_bytes().try_into().unwrap(),
			 })
		);
	});
}

#[test]
fn transfer_self_reserve_works() {
	ExtBuilder::default().with_balances(vec![(Alice, 1000)])
	.build().execute_with(|| {

		let parent_junction: Bytes = vec![0u8].into();
		let mut account_id_32 = vec![2u8];
		account_id_32.extend_from_slice(&[1u8;32]);
		account_id_32.extend_from_slice(&[0u8]);

		assert_eq!(Precompiles::execute(
			Precompile.into(),
			&EvmDataWriter::new()
				.write_selector(Action::Transfer)
				.write(Address(SelfReserve.into()))
				.write(U256::from(500))
				.write(vec![parent_junction, account_id_32.into()])
				.write(U256::from(4000000))
				.build(),
			None,
			&evm::Context {
				address: Precompile.into(),
				caller: Alice.into(),
				apparent_value: From::from(0),
			},
			), Some(Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, cost: 3000, output: vec![], logs: vec![] }))
		);
	});
}


#[test]
fn transfer_to_reserve_works() {
	ExtBuilder::default().with_balances(vec![(Alice, 1000)])
	.build().execute_with(|| {

		let parent_junction: Bytes = vec![0u8].into();
		let mut account_id_32 = vec![2u8];
		account_id_32.extend_from_slice(&[1u8;32]);
		account_id_32.extend_from_slice(&[0u8]);

		// We are transferring asset 0, which we have instructed to be the relay asset
		assert_eq!(Precompiles::execute(
			Precompile.into(),
			&EvmDataWriter::new()
				.write_selector(Action::Transfer)
				.write(Address(AssetId(0u128).into()))
				.write(U256::from(500))
				.write(vec![parent_junction, account_id_32.into()])
				.write(U256::from(4000000))
				.build(),
			None,
			&evm::Context {
				address: Precompile.into(),
				caller: Alice.into(),
				apparent_value: From::from(0),
			},
			), Some(Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, cost: 3000, output: vec![], logs: vec![] }))
		);
	});
}

#[test]
fn transfer_non_reserve_to_non_reserve_works() {
	ExtBuilder::default().with_balances(vec![(Alice, 1000)])
	.build().execute_with(|| {

		let parent_junction: Bytes = vec![0u8].into();
		let mut account_id_32 = vec![2u8];
		account_id_32.extend_from_slice(&[1u8;32]);
		account_id_32.extend_from_slice(&[0u8]);

		// We are transferring asset 1, which corresponds to another parachain Id asset
		assert_eq!(Precompiles::execute(
			Precompile.into(),
			&EvmDataWriter::new()
				.write_selector(Action::Transfer)
				.write(Address(AssetId(1u128).into()))
				.write(U256::from(500))
				.write(vec![parent_junction, account_id_32.into()])
				.write(U256::from(4000000))
				.build(),
			None,
			&evm::Context {
				address: Precompile.into(),
				caller: Alice.into(),
				apparent_value: From::from(0),
			},
			), Some(Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, cost: 3000, output: vec![], logs: vec![] }))
		);
	});
}


#[test]
fn transfer_multi_asset_to_reserve_works() {
	ExtBuilder::default().with_balances(vec![(Alice, 1000)])
	.build().execute_with(|| {

		let parent_junction: Bytes = vec![0u8].into();
		let mut account_id_32 = vec![2u8];
		account_id_32.extend_from_slice(&[1u8;32]);
		account_id_32.extend_from_slice(&[0u8]);

		assert_eq!(Precompiles::execute(
			Precompile.into(),
			&EvmDataWriter::new()
				.write_selector(Action::TransferMultiAsset)
				.write(vec![parent_junction.clone()])
				.write(U256::from(500))
				.write(vec![parent_junction, account_id_32.into()])
				.write(U256::from(4000000))
				.build(),
			None,
			&evm::Context {
				address: Precompile.into(),
				caller: Alice.into(),
				apparent_value: From::from(0),
			},
			), Some(Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, cost: 3000, output: vec![], logs: vec![] }))
		);
	});
}