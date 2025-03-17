use crate::types::{ActiveOption, Config, CounterOfferProposal};
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

// contract info
pub const CONTRACT_NAME: &str = "vault_contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Only INSTANTIATOR_ADDR can call the InstantiateMsg
pub const INSTANTIATOR_ADDR: &str = "contract1";

// Minimum duration between calls to unbond collateral during liquidation
pub const STAKE_LIQUIDATION_INTERVAL: u64 = 60 * 60 * 24 * 30;

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This stores the state for the active liquidity request option
pub const LIQUIDITY_REQUEST_STATE: Item<Option<ActiveOption>> =
    Item::new("LIQUIDITY_REQUEST_STATE");

// This stores max allowed conter offers
pub const MAX_COUNTER_OFFERS: usize = 10;

// Define the indexes for counter offers
pub struct CounterOfferIndexes<'a> {
    pub amount: MultiIndex<'a, (u128, Addr), CounterOfferProposal, Addr>, // ✅ Use u128
}

// Implement IndexList for CounterOfferIndexes
impl<'a> IndexList<CounterOfferProposal> for CounterOfferIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CounterOfferProposal>> + '_> {
        Box::new(vec![&self.amount as &dyn Index<CounterOfferProposal>].into_iter())
    }
}

// Define the IndexedMap
pub fn counter_offer_list<'a>(
) -> IndexedMap<'a, Addr, CounterOfferProposal, CounterOfferIndexes<'a>> {
    IndexedMap::new(
        "COUNTER_OFFER_LIST",
        CounterOfferIndexes {
            amount: MultiIndex::new(
                |_pk, d| (d.amount.u128(), d.proposer.clone()), // ✅ Convert Uint128 to u128
                "COUNTER_OFFER_LIST",
                "COUNTER_OFFER_LIST__amount",
            ),
        },
    )
}
