#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Randomness},
	transactional,
};
use frame_system::{
	offchain::{SendTransactionTypes, SubmitTransaction},
	pallet_prelude::*,
};
pub use pallet::*;
use rand_chacha::{
	rand_core::{RngCore, SeedableRng},
	ChaChaRng,
};
use sp_io::hashing::blake2_128;
use sp_runtime::offchain::storage_lock::{BlockAndTime, StorageLock};
use sp_std::{convert::TryInto, prelude::*};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// mod weights;

// pub use weights::WeightInfo;
#[cfg(test)]
mod tests;
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum KittyGender {
	Male,
	Female,
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]);
impl Kitty {
	pub fn gender(&self) -> KittyGender {
		if self.0[0] % 2 == 0 {
			KittyGender::Male
		} else {
			KittyGender::Female
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<TokenData = Kitty, ClassData = ()>
		+ SendTransactionTypes<Call<Self>>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		// type WeightInfo: WeightInfo;
		#[pallet::constant]
		type DefaultDifficulty: Get<u32>;
	}
	pub type KittyIndexOf<T> = <T as orml_nft::Config>::TokenId;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Nonce for auto breed to prevent replay attack
	#[pallet::storage]
	#[pallet::getter(fn auto_breed_nonce)]
	pub type AutoBreedNonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn offchain_worker(_now: T::BlockNumber) {
			let _ = Self::run_offchain_worker();
		}
	}

	// Stores all the kitties prices. Key is (kitty_id, price).
	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub type Prices<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyIndexOf<T>, BalanceOf<T>, OptionQuery>;

	// Class Id orml_nft
	#[pallet::storage]
	#[pallet::getter(fn class_id)]
	pub type ClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig;
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// create a NFT class
			let class_id = orml_nft::Pallet::<T>::create_class(&Default::default(), Vec::new(), ())
				.expect("Cannot fail or invalid chain spec");
			ClassId::<T>::put(class_id);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", KittyIndexOf<T> = "KittyIndex", Option<BalanceOf<T>> = "Option<Balance>", BalanceOf<T> = "Balance")]
	pub enum Event<T: Config> {
		// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(T::AccountId, KittyIndexOf<T>, Kitty),
		// A kitty is bred. \[owner, kitty_id, kitty, kitty_parent_1_, kitty_parent_2\]
		KittyBred(
			T::AccountId,
			KittyIndexOf<T>,
			Kitty,
			KittyIndexOf<T>,
			KittyIndexOf<T>,
		),
		// A kitty is transferred. \[from, to, kitty_id, kitty\]
		KittyTransferred(T::AccountId, T::AccountId, KittyIndexOf<T>, Kitty),
		// A kitty has price updated. \[from, kitty_id, kitty, max_price\]
		KittyPriceUpdated(T::AccountId, KittyIndexOf<T>, Kitty, Option<BalanceOf<T>>),
		// A kitty is sold. \[from, to, kitty_id, kitty, max_price\]
		KittySold(
			T::AccountId,
			T::AccountId,
			KittyIndexOf<T>,
			Kitty,
			BalanceOf<T>,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameGender,
		NotOwner,
		NotForSale,
		PriceTooLow,
		BuyFromSelf,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create a new kitty
		#[pallet::weight(1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Generate a random 128bit value
			let dna: [u8; 16] = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			let kitty_id =
				orml_nft::Pallet::<T>::mint(&sender, Self::class_id(), Vec::new(), kitty.clone())?;
			// Emit event
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));

			Ok(())
		}
		// Create a new kitty
		#[pallet::weight(1000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyIndexOf<T>,
			kitty_id_2: KittyIndexOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty1 = Self::kitties(&sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Self::kitties(&sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
			Self::do_breed(sender, kitty1, kitty2, kitty_id_1, kitty_id_2)
		}
		// Transfer an owned kitty
		#[pallet::weight(1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: KittyIndexOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			orml_nft::Pallet::<T>::transfer(&sender, &to, (Self::class_id(), kitty_id))?;
			if sender != to {
				Prices::<T>::remove(kitty_id);
				let transferred_kitty =
					Self::kitties(&to, kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
				// Emit event
				Self::deposit_event(Event::KittyTransferred(
					sender,
					to,
					kitty_id,
					transferred_kitty,
				));
			}
			Ok(())
		}
		// set kitty's price- None will unlist the kitty
		#[pallet::weight(1000)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: KittyIndexOf<T>,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(
				orml_nft::TokensByOwner::<T>::contains_key(&sender, (Self::class_id(), kitty_id)),
				Error::<T>::NotOwner
			);
			Prices::<T>::mutate_exists(kitty_id, |price| *price = new_price);
			// Emit event
			let updated_kitty =
				Self::kitties(&sender, kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			Self::deposit_event(Event::KittyPriceUpdated(
				sender,
				kitty_id,
				updated_kitty,
				new_price,
			));
			Ok(())
		}
		// buy a kitty
		#[pallet::weight(1000)]
		#[transactional]
		pub fn buy(
			origin: OriginFor<T>,
			owner: T::AccountId,
			kitty_id: KittyIndexOf<T>,
			max_price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender != owner, Error::<T>::BuyFromSelf);
			Prices::<T>::try_mutate_exists(kitty_id, |price| -> DispatchResult {
				// remove old owner kitty price - if absent not for sale
				let price = price.take().ok_or(Error::<T>::NotForSale)?;
				ensure!(max_price >= price, Error::<T>::PriceTooLow);
				orml_nft::Pallet::<T>::transfer(&owner, &sender, (Self::class_id(), kitty_id))?;
				T::Currency::transfer(&sender, &owner, price, ExistenceRequirement::KeepAlive)?;
				let bought_kitty =
					Self::kitties(&sender, kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
				Self::deposit_event(Event::KittySold(
					owner,
					sender,
					kitty_id,
					bought_kitty,
					price,
				));
				Ok(())
			})
		}
		// auto breed extrinsic
		#[pallet::weight(1000)]
		pub fn auto_breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyIndexOf<T>,
			kitty_id_2: KittyIndexOf<T>,
			_nonce: u32,
			_solution: u128,
		) -> DispatchResult {
			// ran from offchain worker
			ensure_none(origin)?;
			let kitty1 = orml_nft::Pallet::<T>::tokens(Self::class_id(), kitty_id_1)
				.ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = orml_nft::Pallet::<T>::tokens(Self::class_id(), kitty_id_2)
				.ok_or(Error::<T>::InvalidKittyId)?;
			Self::do_breed(
				kitty1.owner,
				kitty1.data,
				kitty2.data,
				kitty_id_1,
				kitty_id_2,
			)
		}
	}
	#[pallet::validate_unsigned]
	impl<T: Config> frame_support::unsigned::ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match *call {
				Call::auto_breed(kitty_id_1, kitty_id_2, nonce, solution) => {
					if Self::validate_solution(kitty_id_1, kitty_id_2, nonce, solution) {
						if nonce != Self::auto_breed_nonce() {
							return InvalidTransaction::BadProof.into();
						}
						AutoBreedNonce::<T>::mutate(|nonce| *nonce = nonce.saturating_add(1));
						ValidTransaction::with_tag_prefix("kitties")
							.longevity(64_u64)
							.propagate(true)
							.build()
					} else {
						InvalidTransaction::BadProof.into()
					}
				}
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	fn kitties(owner: &T::AccountId, kitty_id: KittyIndexOf<T>) -> Option<Kitty> {
		orml_nft::Pallet::<T>::tokens(Self::class_id(), kitty_id).and_then(|x| {
			if x.owner == *owner {
				Some(x.data)
			} else {
				None
			}
		})
	}
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed().0,
			&sender,
			<frame_system::Pallet<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}
	fn combine_dna(dna1_byte: u8, dna2_byte: u8, selector: u8) -> u8 {
		(!selector & dna1_byte) | (selector & dna2_byte)
	}
	fn do_breed(
		owner: <T>::AccountId,
		kitty1: Kitty,
		kitty2: Kitty,
		kitty_id_1: KittyIndexOf<T>,
		kitty_id_2: KittyIndexOf<T>,
	) -> DispatchResult {
		ensure!(kitty1.gender() != kitty2.gender(), Error::<T>::SameGender);
		let kitty1_dna = kitty1.0;
		let kitty2_dna = kitty2.0;

		let selector = Self::random_value(&owner);
		let mut child_dna = [0u8; 16];
		for i in 0..kitty1_dna.len() {
			child_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}
		let bred_kitty = Kitty(child_dna);
		let bred_kitty_id =
			orml_nft::Pallet::<T>::mint(&owner, Self::class_id(), Vec::new(), bred_kitty.clone())?;
		// Emit event
		Self::deposit_event(Event::KittyBred(
			owner,
			bred_kitty_id,
			bred_kitty,
			kitty_id_1,
			kitty_id_2,
		));
		Ok(())
	}
	fn validate_solution(
		kitty_id_1: KittyIndexOf<T>,
		kitty_id_2: KittyIndexOf<T>,
		nonce: u32,
		solution: u128,
	) -> bool {
		let payload = (kitty_id_1, kitty_id_2, nonce, solution);
		let hash = payload.using_encoded(blake2_128);
		let hash_value = u128::from_le_bytes(hash);
		let difficulty = T::DefaultDifficulty::get();
		hash_value < (u128::max_value() / difficulty as u128)
	}
	fn run_offchain_worker() -> Result<(), ()> {
		let mut lock =
			StorageLock::<'_, BlockAndTime<frame_system::Pallet<T>>>::with_block_deadline(
				&b"kitties/lock"[..],
				1,
			);
		let _guard = lock.try_lock().map_err(|_| ())?;
		let random_seed = sp_io::offchain::random_seed();
		let mut rng = ChaChaRng::from_seed(random_seed);
		// this only supports if kitty_count <= u32::max_value()
		let kitty_count =
			TryInto::<u32>::try_into(orml_nft::Pallet::<T>::next_token_id(Self::class_id()))
				.map_err(|_| ())?;
		if kitty_count == 0 {
			return Ok(());
		}
		const MAX_ITERATIONS: u128 = 500;
		let nonce = Self::auto_breed_nonce();
		let mut remaining_iterations: u128 = MAX_ITERATIONS;
		let (kitty_1, kitty_2) = loop {
			let kitty_id_1: KittyIndexOf<T> = (rng.next_u32() % kitty_count).into();
			let kitty_id_2: KittyIndexOf<T> = (rng.next_u32() % kitty_count).into();
			let kitty_1 = orml_nft::Pallet::<T>::tokens(Self::class_id(), kitty_id_1).ok_or(())?;
			let kitty_2 = orml_nft::Pallet::<T>::tokens(Self::class_id(), kitty_id_2).ok_or(())?;
			if kitty_1.data.gender() != kitty_2.data.gender() {
				break (kitty_id_1, kitty_id_2);
			}
			remaining_iterations -= 1;
			if remaining_iterations == 0 {
				return Err(());
			}
		};
		let solution_prefix = rng.next_u32() as u128;
		for i in 0..remaining_iterations {
			let solution: u128 = (solution_prefix << 32) + i;
			if Self::validate_solution(kitty_1, kitty_2, nonce, solution) {
				let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
					Call::<T>::auto_breed(kitty_1, kitty_2, nonce, solution).into(),
				);
				break;
			}
		}
		Ok(())
	}
}
