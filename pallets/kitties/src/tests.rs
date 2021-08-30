use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

//测试创建kitty 
#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(3), 100));
		/*let kitty_id = KittiesCount::<Test>::get();
		assert_eq!(
			Some(kitty_id),
			None
		);*/
	});
}

#[test]
fn create_failed_when_index_max() {
	new_test_ext().execute_with(|| {
		let amount = 10000;
		assert_ok!(KittyModule::create(Origin::signed(1), amount.clone()));
		let kitty_id = Some(KittiesCount::<Test>::get());
		KittiesCount::<Test>::set(Some(65535));
		assert_noop!(
			KittyModule::create(Origin::signed(1), amount.clone()),
			Error::<Test>::KittiesCountOverflow
		);
	})
}
