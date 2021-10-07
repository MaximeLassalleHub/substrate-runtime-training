#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::Randomness};
use frame_system::pallet_prelude::*;
use sp_io::hashing::blake2_128;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd, One},
	ArithmeticError,
};

pub use pallet::*;
#[cfg(test)]
mod tests;
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum KittyGender {
	Male,
	Female,
}
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
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
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy;
	}
	/// Stores all the kitties. Key is (user, kitty_id).
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::KittyIndex,
		Kitty,
		OptionQuery,
	>;

	/// Stores the next kitty Id.
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", T::KittyIndex = "KittyIndex")]
	pub enum Event<T: Config> {
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		/// A kitty is bred. \[owner, kitty_id, kitty, kitty_parent_1_, kitty_parent_2\]
		KittyBred(
			T::AccountId,
			T::KittyIndex,
			Kitty,
			T::KittyIndex,
			T::KittyIndex,
		),
		/// A kitty is transferred. \[from, to, kitty_id, kitty\]
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex, Kitty),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameGender,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new kitty
		#[pallet::weight(1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::get_next_kitty_id()?;
			// Generate a random 128bit value
			let dna: [u8; 16] = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			Kitties::<T>::insert(&sender, kitty_id, kitty.clone());

			// Emit event
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));

			Ok(())
		}
		/// Create a new kitty
		#[pallet::weight(1000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty1 = Self::kitties(&sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Self::kitties(&sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(kitty1.gender() != kitty2.gender(), Error::<T>::SameGender);
			let bred_kitty_id = Self::get_next_kitty_id()?;
			let kitty1_dna = kitty1.0;
			let kitty2_dna = kitty2.0;

			let selector = Self::random_value(&sender);
			let mut child_dna = [0u8; 16];
			for i in 0..kitty1_dna.len() {
				child_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
			}
			let bred_kitty = Kitty(child_dna);
			Kitties::<T>::insert(&sender, bred_kitty_id, &bred_kitty);
			// Emit event
			Self::deposit_event(Event::KittyBred(
				sender,
				bred_kitty_id,
				bred_kitty,
				kitty_id_1,
				kitty_id_2,
			));
			Ok(())
		}
		/// Transfer an owned kitty
		#[pallet::weight(1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Kitties::<T>::try_mutate_exists(sender.clone(), kitty_id, |kitty| -> DispatchResult {
				if sender == to {
					ensure!(kitty.is_some(), Error::<T>::InvalidKittyId);
					return Ok(());
				}
				let transfer_kitty: Kitty = kitty.take().ok_or(Error::<T>::InvalidKittyId)?;
				Kitties::<T>::insert(&to, kitty_id, transfer_kitty);
				let transferred_kitty = Self::kitties(&to, kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
				// Emit event
				Self::deposit_event(Event::KittyTransferred(sender, to, kitty_id, transferred_kitty));
				Ok(())
			})
		}
	}
}
impl<T: Config> Pallet<T> {
	fn get_next_kitty_id() -> Result<T::KittyIndex, DispatchError> {
		NextKittyId::<T>::try_mutate(|next_id| -> Result<T::KittyIndex, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id
				.checked_add(&One::one())
				.ok_or(ArithmeticError::Overflow)?;
			Ok(current_id)
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
}
