License: Unlicense
## DESIGN KITTY PALLET
# Calls
* fn create
* fn breed(kitty_id_1: KittyIndex,kitty_id_2: KittyIndex)
* fn transfer(to:AccountId,kitty_id:KittyIndex)
* fn buy(kitty_id: KittyIndex)
# Types
* enum Gender {
    Male,
    Female,
}
* struct Kitty
    * dna: u128
    * currency_id: CurrencyId
    * price: Balance
# Errors
InvalidKittyId,
SameGender,
InsufficientBalance
# Storages
* Kitties: double_map KittyIndex,AccountId => Option<Kitty>
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
* KittyBought
    * kitty_id: KittyIndex
    * kitty: Kitty
    * seller: AccountId
    * buyer: AccountId


