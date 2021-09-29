License: Unlicense
## DESIGN KITTY PALLET
# Calls
* fn create
* fn breed
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
* Kitties: double_map KittyId,AccountId => Option<Kitty>
* NextKittyId: KittyId
# Events
* KittyCreated
    * kitty_id: KittyId
    * kitty: Kitty
* KittyTransfered
    * from: AccountId
    * to: AccountId
    * kitty_id: KittyId
    * kitty: Kitty
* KittyBought
    * seller: AccountId
    * buyer: AccountId
    * kitty_id: KittyId
    * kitty: Kitty

