#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::Randomness,
};
use frame_system::pallet_prelude::*;
use sp_runtime::ArithmeticError;
use sp_io::hashing::blake2_128;
pub use pallet::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]);

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_randomness_collective_flip::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	/// Stores all the kitties. Key is (user, kitty_id).
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, T::AccountId,
		Blake2_128Concat, u32,
		Kitty, OptionQuery
	>;

	/// Stores the next kitty Id.
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(T::AccountId, u32, Kitty)
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
 	impl<T:Config> Pallet<T> {

		/// Create a new kitty
		#[pallet::weight(1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// TODO: ensure kitty id does not overflow
			// return Err(ArithmeticError::Overflow.into());

			// Generate a random 128bit value
			let payload = (
				<pallet_randomness_collective_flip::Pallet<T> as Randomness<T::Hash, T::BlockNumber>>::random_seed().0,
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			let dna = payload.using_encoded(blake2_128);

			// Create and store kitty
			let kitty = Kitty(dna);
			let kitty_id = Self::next_kitty_id();
			Kitties::<T>::insert(&sender, kitty_id, kitty.clone());
			NextKittyId::<T>::put(kitty_id + 1);

			// Emit event
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));

			Ok(())
		}
	}
}
