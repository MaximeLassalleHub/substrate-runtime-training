License: Unlicense
## DESIGN KITTY PALLET
# Calls
* fn create
* fn breed(kitty_id_1: u32,kitty_id_2: u32)
* fn transfer
* sell
# Types
* enum Gender {
    Male,
    Female,
}
* struct Kitty
    * dna: u128
    * currency_id: CurrencyId
    * price: Balance
    * gender: Gender
# Storages
* Kitties: double_map u32,AccountId => Option<Kitty>
* Nextu32: u32
# Events
* KittyCreated
    * kitty_id: u32
    * kitty: Kitty
    * owner: AccountId
* KittyBred
    * kitty_parent_1_: u32
    * kitty_parent_2_: u32
    * kitty_id: u32
    * kitty: Kitty
    * owner: AccountId
* KittyTransfered
    * from: AccountId
    * to: AccountId
    * kitty_id: u32
    * kitty: Kitty
* KittyBought
    * seller: AccountId
    * buyer: AccountId
    * kitty_id: u32
    * kitty: Kitty
# functiu 

