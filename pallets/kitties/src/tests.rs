use super::*;

use crate as kitties;
use frame_support::{assert_noop, assert_ok, parameter_types};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            KittiesModule: kitties::{Pallet, Call,Storage, Event<T>}
        }
);
parameter_types! {
    pub const BlockHashCount: u64= 250;
    pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}
parameter_types! {
    pub const ExistentialDeposit:u64 =1;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}
//impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
    pub static MockRandom: H256 = Default::default();
}
impl Randomness<H256, u64> for MockRandom {
    fn random(_subject: &[u8]) -> (H256, u64) {
        (MockRandom::get(), 0)
    }
}
impl Config for Test {
    type Event = Event;
    type Randomness = MockRandom;
    type KittyIndex = u32;
    type Currency = Balances;
}
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();
    t.execute_with(|| System::set_block_number(1));
    t
}
#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        // assert Kitty create call ends up OK
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // create u8 , 16 Kitty manually upon on random_value specs
        let kitty = Kitty([
            59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122,
        ]);
        assert_eq!(KittiesModule::kitties(100, 0), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 1);
        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyCreated(
            100, 0, kitty,
        )));
    });
}

#[test]
fn gender() {
    assert_eq!(Kitty([0; 16]).gender(), KittyGender::Male);
    assert_eq!(Kitty([1; 16]).gender(), KittyGender::Female);
}
#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        // create first parent on extrinsic index 0
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // change mock random so next kitty has opposite gender
        MockRandom::set(H256::from([2; 32]));
        // crete 2nd parent with opposite gender
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // assert Invalid Kitty Id on Breed
        assert_noop!(
            KittiesModule::breed(Origin::signed(100), 0, 9),
            Error::<Test>::InvalidKittyId
        );
        // validate parents have opposite genders
        assert_noop!(
            KittiesModule::breed(Origin::signed(100), 0, 0),
            Error::<Test>::SameGender
        );
        // validate account does not own one of the kitties for breeding
        assert_noop!(
            KittiesModule::breed(Origin::signed(101), 0, 1),
            Error::<Test>::InvalidKittyId
        );

        assert_ok!(KittiesModule::breed(Origin::signed(100), 0, 1));

        // create u8 , 16 Kitty manually upon on random_value specs
        let kitty = Kitty([
            187, 250, 235, 118, 211, 247, 237, 253, 187, 239, 191, 185, 239, 171, 211, 122,
        ]);
        assert_eq!(KittiesModule::kitties(100, 2), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 3);
        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyBred(
            100u64, 2u32, kitty, 0u32, 1u32,
        )));
    });
}
#[test]
fn can_transfer() {
    new_test_ext().execute_with(|| {
        // create first kitty on Account 100
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // create 2nd kitty on Account 101
        assert_ok!(KittiesModule::create(Origin::signed(101)));
        // assert Invalid Kitty Id if not owned kitty
        assert_noop!(
            KittiesModule::transfer(Origin::signed(100), 100, 1),
            Error::<Test>::InvalidKittyId
        );

        assert_ok!(KittiesModule::transfer(Origin::signed(100), 101, 0));
        let kitty1_transferred =
            KittiesModule::kitties(101, 0).ok_or(Error::<Test>::InvalidKittyId);
        System::assert_last_event(Event::KittiesModule(
            crate::Event::<Test>::KittyTransferred(
                100u64,
                101u64,
                0,
                kitty1_transferred.unwrap() as Kitty,
            ),
        ));

        assert_eq!(KittiesModule::next_kitty_id(), 2);
    });
}

// exercise exchange tests