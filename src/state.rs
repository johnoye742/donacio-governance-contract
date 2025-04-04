use cosmwasm_std::Addr;
use cw_storage_plus::Item;
// should probably just store their contract addresses
pub const FUNDRAISERS: Item<Vec<Addr>> = Item::new("fundraisers");
