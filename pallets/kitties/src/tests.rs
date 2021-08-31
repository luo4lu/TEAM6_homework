use crate::{Error, Event, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

//测试创建kitty 
#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		System::assert_has_event(mock::Event::KittyModule(Event::KittyCreate(1,1)));
	});
}

#[test]
fn create_failed_when_index_max() {
	new_test_ext().execute_with(|| {
		KittiesCount::<Test>::put(u32::max_value());
		assert_noop!(
			KittyModule::create(Origin::signed(1)),
			Error::<Test>::KittiesCountOverflow
		);
	})
}

#[test]
fn create_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert_noop!(KittyModule::create(Origin::signed(3)), Error::<Test>::BalanceLitter);
	})
}

#[test]
fn transfer_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::transfer(Origin::signed(1), 2, 1));
	})
}

#[test]
fn transfer_failed_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_noop!(KittyModule::transfer(Origin::signed(2), 3, 1), Error::<Test>::NotOwner);
	})
}

#[test]
fn breed_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::breed(Origin::signed(1), 1, 2));
	})
}

#[test]
fn breed_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_noop!(KittyModule::breed(Origin::signed(1), 1, 1), Error::<Test>::SameParentIndex);
		assert_noop!(KittyModule::breed(Origin::signed(1), 1, 2), Error::<Test>::InvalidKittyIndex);
		assert_ok!(KittyModule::create(Origin::signed(1)));
		KittiesCount::<Test>::put(u32::max_value());
		assert_noop!(KittyModule::breed(Origin::signed(1), 1, 2), Error::<Test>::KittiesCountOverflow);
	})
}

#[test]
fn buy_kitty_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::buy_kitty(Origin::signed(2), 1, 100));
	})
}

#[test]
fn buy_kitty_failed() {
	new_test_ext().execute_with(|| {
		//测试kitty id 无效
		assert_noop!(KittyModule::buy_kitty(Origin::signed(1), 1, 100), Error::<Test>::InvalidKittyIndex);
		//测试owner
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_noop!(KittyModule::buy_kitty(Origin::signed(1), 1, 100), Error::<Test>::FromSameTo);

		assert_noop!(KittyModule::buy_kitty(Origin::signed(2), 1, 1_000_000_000), Error::<Test>::BalanceLitter);
	})
}

#[test]
fn sell_kitty_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::sell_kitty(Origin::signed(1), 2, 1, 100));
	})
}

#[test]
fn sell_kitty_failed() {
	new_test_ext().execute_with(|| {
		//测试kitty id 无效
		assert_noop!(KittyModule::sell_kitty(Origin::signed(1), 2, 1, 100), Error::<Test>::InvalidKittyIndex);
		//测试owner
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_noop!(KittyModule::sell_kitty(Origin::signed(2), 1, 1, 100), Error::<Test>::FromSameTo);
	})
}