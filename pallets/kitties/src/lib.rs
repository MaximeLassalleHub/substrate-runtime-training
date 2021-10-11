#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Randomness, Currency, ExistenceRequirement},
	transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;
use sp_io::hashing::blake2_128;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use pallet::*;

#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking; 
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum KittyGender {
	Male,
	Female,
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone,Copy, RuntimeDebug, PartialEq, Eq)]
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
	pub trait Config: frame_system::Config + orml_nft::Config<TokenData = Kitty, ClassData = ()>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
	}
	pub type KittyIndexOf<T> = <T as orml_nft::Config>::TokenId;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
			let kitty_id =orml_nft::Pallet::<T>::mint(&sender, Self::class_id(), Vec::new(), kitty.clone())?;
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
			ensure!(kitty1.gender() != kitty2.gender(), Error::<T>::SameGender);
			let kitty1_dna = kitty1.0;
			let kitty2_dna = kitty2.0;

			let selector = Self::random_value(&sender);
			let mut child_dna = [0u8; 16];
			for i in 0..kitty1_dna.len() {
				child_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
			}
			let bred_kitty = Kitty(child_dna);
			let bred_kitty_id = orml_nft::Pallet::<T>::mint(
				&sender,
				Self::class_id(),
				Vec::new(),
				bred_kitty.clone(),
			)?;
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
			orml_nft::TokensByOwner::<T>::contains_key(&sender,(Self::class_id(), kitty_id)),Error::<T>::NotOwner);
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
					ensure!(max_price >= price , Error::<T>::PriceTooLow);
					orml_nft::Pallet::<T>::transfer(&owner,&sender,(Self::class_id(),kitty_id))?;
					T::Currency::transfer(&sender, &owner, price,ExistenceRequirement::KeepAlive)?;
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
	}
}
impl<T: Config> Pallet<T> {
	fn kitties(owner:&T::AccountId, kitty_id: KittyIndexOf<T>)->Option<Kitty>{
		orml_nft::Pallet::<T>::tokens(Self::class_id(),kitty_id).and_then(|x|{
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
}
