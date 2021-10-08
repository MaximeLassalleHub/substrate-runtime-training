License: Unlicense
## DESIGN KITTY PALLET
# Calls
* fn create
* fn breed(kitty_id_1: KittyIndex,kitty_id_2: KittyIndex)
* fn transfer(to:AccountId,kitty_id:KittyIndex)
* fn set_price(origin, kitty_id: KittyIndex, price: Option<Balance>)
* fn buy(origin, owner: AccountId, kitty_id: KittyIndex, max_price: Balance)
# Types
* enum Gender {
    Male,
    Female,
}
* struct Kitty<u128>
# Errors
InvalidKittyId,
SameGender,
InsufficientBalance
# Storages
* Kitties: double_map KittyIndex,AccountId => Option<Kitty>
* Kitties: map KittyIndex => Option<Balance>
* NextKittyIndex: KittyIndex
# Events
* KittyCreated
    * kitty_id: KittyIndex
    * kitty: Kitty
    * owner: AccountId
* KittyBred
    * owner: AccountId
    * kitty_id: KittyIndex
    * kitty: Kitty
    * kitty_parent_1_: KittyIndex
    * kitty_parent_2_: KittyIndex
* KittyTransferred
    * from: AccountId
    * to: AccountId
    * kitty_id: KittyIndex
    * kitty: Kitty
* KittySold
    * kitty_id: KittyIndex
    * kitty: Kitty
    * old_onwer: AccountId
    * old_owner: AccountId
    * max_price: Balance
* KittyPriceUpdated
    * owner: AccountId
    * kitty_id: KittyIndex
    * kitty: Kitty
    * price: Option<Balance>


