#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Randomness};
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
      //  type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreate(T::AccountId, T::KittyIndex),
        KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex)
    }

    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        SameParentIndex,
        InvalidKittyIndex,
        BalanceLitter,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    //创建时质押token 对应AccountId
    #[pallet::storage]
    #[pallet::getter(fn get_balance)]
    pub type BalanceToAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        T::Balance,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    //id = T::KittyIndex::get();
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id
                },
                None => {
                    1u32.into()
                }
            };
            let dna = Self::random_value(&who);

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            Owner::<T>::insert(kitty_id, Some(who.clone()));
            BalanceToAccount::<T>::insert(who.clone(), amount);

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
                   // let index = T::KittyIndex::get();
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

        //买入kitty
        #[pallet::weight(0)]
        pub fn buy_kitty(origin: OriginFor<T>, from: T::AccountId, kitty_id: T::KittyIndex, amount: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            //判断账户中的balance大于等于交易费用
            let account_balance = Self::get_balance(who.clone());
            ensure!(amount < account_balance, Error::<T>::BalanceLitter);
            let from_balance = Self::get_balance(from.clone());

            //对应的账户增加balance
            BalanceToAccount::<T>::insert(who.clone(), account_balance-amount);
            BalanceToAccount::<T>::insert(from.clone(), from_balance+amount);

            Owner::<T>::insert(kitty_id, Some(from.clone()));

            Self::deposit_event(Event::KittyTransfer(who, from, kitty_id));

            Ok(())
        }

        //卖出kitty
        #[pallet::weight(0)]
        pub fn sell_kitty(origin: OriginFor<T>, to: T::AccountId, kitty_id: T::KittyIndex, amount: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            //判断账户中的balance大于等于交易费用
            let to_balance = Self::get_balance(to.clone());
            ensure!(amount > to_balance, Error::<T>::BalanceLitter);
            let account_balance = Self::get_balance(who.clone());

            //对应的账户增加balance
            BalanceToAccount::<T>::insert(who.clone(), account_balance+amount);
            BalanceToAccount::<T>::insert(to.clone(), to_balance-amount);

            Owner::<T>::insert(kitty_id, Some(to.clone()));

            Self::deposit_event(Event::KittyTransfer(who, to, kitty_id));

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

