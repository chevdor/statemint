#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::{Currency, Vec}};
	use frame_system::{pallet_prelude::*, ensure_root};
	use frame_system::Config as SystemConfig;
	use frame_support::sp_runtime::{
		RuntimeDebug,
		traits::{
			AtLeast32BitUnsigned,
		}
	};
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	}


	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
	pub struct AuthorInfo<AccountId, Balance, BlockNumber> {
		who: AccountId,
		deposit: Balance,
		last_block: BlockNumber
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	
	#[pallet::storage]
	#[pallet::getter(fn invulnerables)]
	pub type Invulnerables<T: Config>= StorageValue<_, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn authors)]
	pub type Authors<T: Config> = StorageValue<_, Vec<AuthorInfo<T::AccountId, T::Balance, T::BlockNumber>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_authors)]
	pub type MaxAuthors<T> = StorageValue<_, u32>; 

	#[pallet::storage]
	#[pallet::getter(fn authority_bond)]
	pub type AuthorityBond<T: Config>= StorageValue<_, T::Balance>;

	
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewInvulnerables(Vec<T::AccountId>),
		NewMaxAuthorCount(u32),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MaxAuthors,
		Unknown,
		Permission,
		AlreadyAuthor,
		NotAuthor
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// on init add to aura set at next era 
		// split half the pot to the author per block
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000)]
		fn set_invulnerables(origin: OriginFor<T>, new: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			//TODO chose protective scheme
			ensure_root(origin)?;
			<Invulnerables<T>>::put(&new);
			Self::deposit_event(Event::NewInvulnerables(new));
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		fn set_max_author_count(origin: OriginFor<T>, max_authors: u32) -> DispatchResultWithPostInfo {
			//TODO chose protective scheme
			ensure_root(origin)?;
			<MaxAuthors<T>>::put(&max_authors);
			Self::deposit_event(Event::NewMaxAuthorCount(max_authors));
			// sets max authors
			Ok(().into())

		}

		#[pallet::weight(10_000)]
		fn author_intent(origin: OriginFor<T>, deposit: T::Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(<Authors<T>>::decode_len().ok_or(Error::<T>::Unknown)? + 1 <= MaxAuthors::<T>::get().ok_or(Error::<T>::Unknown)? as usize, Error::<T>::MaxAuthors); 
			let new_author = AuthorInfo {
				who: who.clone(), 
				deposit, 
				last_block: frame_system::Pallet::<T>::block_number()

			};
			//take deposit
			<Authors<T>>::try_mutate(|authors| -> DispatchResult {
				let exists = authors.into_iter().any(|author| author.who == who);
				match exists {
					true => Ok(authors.push(new_author)),
					false => Err(Error::<T>::AlreadyAuthor)?,
				}
			})?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		fn leave_intent(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<Authors<T>>::try_mutate(|authors| -> DispatchResult {
				let index = authors.iter().position(|author| author.who == who).ok_or(Error::<T>::NotAuthor)?;
				authors.remove(index);
				Ok(())
			})?;


			Ok(().into())

		}
	}
}
