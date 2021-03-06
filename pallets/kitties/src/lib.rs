#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{Randomness, ReservableCurrency, Currency, ExistenceRequirement}};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{AtLeast32Bit, MaybeDisplay, Bounded};
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
    use sp_std::fmt::Debug;

    #[derive(Encode, Decode)]
    pub struct Kitty(pub [u8;16]);
    //type KittyIndex = u32;

    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type KittyIndex: Parameter + Member + MaybeSerializeDeserialize + Debug + Default + MaybeDisplay + AtLeast32Bit
        + Copy + Encode;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type KittyDepositBase: Get<BalanceOf<Self>>;
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreate(T::AccountId, T::KittyIndex),
        KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
        KittySale(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
    }

    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        SameParentIndex,
        InvalidKittyIndex,
        BalanceLitter,
        FromSameTo,
        NotKittySale,
    }

    type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

    #[pallet::storage]
    #[pallet::getter(fn kitties_price)]
    pub type KittiesPrice<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    //id = T::KittyIndex::get();
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id
                },
                None => {
                    0u32.into()
                }
            };
            let deposit = T::KittyDepositBase::get();
            T::Currency::reserve(&who,deposit.clone()).map_err(|_| Error::<T>::BalanceLitter)?;
            let dna = Self::random_value(&who);

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            Owner::<T>::insert(kitty_id, Some(who.clone()));

            KittiesCount::<T>::put(kitty_id+1u32.into());

            Self::deposit_event(Event::KittyCreate(who, kitty_id));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

            Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;
            
            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id
                },
                None => {
                    1u32.into()
                }
            };

            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;

            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
            Owner::<T>::insert(kitty_id, Some(who.clone()));

            KittiesCount::<T>::put(kitty_id+1u32.into());

            Self::deposit_event(Event::KittyCreate(who, kitty_id));

            Ok(())
        }

        //??????kitty
        #[pallet::weight(0)]
        pub fn buy_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            
            ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyIndex);
            let from = Owner::<T>::get(kitty_id).unwrap();
            ensure!(who.clone() != from, Error::<T>::FromSameTo);

            let price = Self::kitties_price(kitty_id).ok_or(Error::<T>::NotKittySale)?;
            //??????????????????balance????????????????????????
            let reserve = T::KittyDepositBase::get();
           
            T::Currency::reserve(&who, reserve).map_err(|_| Error::<T>::BalanceLitter)?;
            T::Currency::unreserve(&from, reserve); 
            T::Currency::transfer(
                &who,
                &from,
                price,
                ExistenceRequirement::KeepAlive,
            )?;
            KittiesPrice::<T>::remove(kitty_id);
            Owner::<T>::insert(kitty_id, Some(who.clone()));

            Self::deposit_event(Event::KittyTransfer(from, who, kitty_id));

            Ok(())
        }

        //??????kitty
        #[pallet::weight(0)]
        pub fn sell_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex, amount: Option<BalanceOf<T>>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::FromSameTo);

            KittiesPrice::<T>::mutate_exists(kitty_id, |p| *p = Some(amount));
            Self::deposit_event(Event::KittySale(who, kitty_id, amount));

            Ok(())
        }

    }
    
    impl<T: Config> Pallet<T> {
        pub fn random_value(sender: &T::AccountId)-> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }
    }
}

