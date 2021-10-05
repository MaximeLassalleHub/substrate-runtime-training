use super::*;

use crate as kitties;
use sp_core::H256;
use frame_support::{parameter_types,assert_ok,assert_noop};
use sp_runtime::{
    traits::{BlakeTwo256,IdentityLookup}, testing::Header,
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
            RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet,Storage},
            KittiesModule: kitties::{Pallet, Call,Storage, Event<T>},
        }
);
parameter_types! {
    pub const BlockHashCount: u64= 250;
    pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
    type  BaseCallFilter = ();
    type  BlockWeights = ();
    type  BlockLength = ();
    type  DbWeight = ();
    type  Origin = Origin;
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}
impl pallet_randomness_collective_flip::Config for Test {}
impl Config for Test {
    type Event = Event;
}
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t : sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    t.execute_with(|| System::set_block_number(1));
    t
}
#[test]
fn can_create(){
    new_test_ext().execute_with(|| {
        // assert Kitty create call ends up OK
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // create u8 , 16 Kitty manually upon on random_value specs
        let kitty = Kitty([59,250,138,82,209,39,141,109,163,238,183,145,235,168,18,122]);
        assert_eq!(KittiesModule::kitties(100,0), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(),1);
        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyCreated(100,0,kitty)));
    });
}

#[test]
fn gender(){
    assert_eq!(Kitty([0; 16]).gender(), KittyGender::Male);
    assert_eq!(Kitty([1; 16]).gender(), KittyGender::Female);
}
#[test]
fn can_breed(){
    new_test_ext().execute_with(|| {
        // create first parent on extrinsic index 0
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        // set extrinsic index 0 so next Kitty is not same gender than before
        System::set_extrinsic_index(1);
        // crete 2nd parent with opposite gender
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        
        // assert Invalid Kitty Id on Breed
        assert_noop!(KittiesModule::breed(Origin::signed(100),0,9),Error::<Test>::InvalidKittyId);
        // validate parents have opposite genders
        assert_noop!(KittiesModule::breed(Origin::signed(100),0,0),Error::<Test>::SameGender);
        // validate account does not own one of the kitties for breeding
        assert_noop!(KittiesModule::breed(Origin::signed(101),0,1),Error::<Test>::InvalidKittyId);

        assert_ok!(KittiesModule::breed(Origin::signed(100),0,1));

        // create u8 , 16 Kitty manually upon on random_value specs
        let kitty = Kitty([59,250,138,82,209,39,141,109,163,238,183,145,235,168,18,122]);
        assert_eq!(KittiesModule::kitties(100,2), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(),3);
        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyBred(100u64,2u32  ,kitty,0u32,1u32)));
    }); 
}
