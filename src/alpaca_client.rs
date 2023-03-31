use super::csv_handler::ZacksBuys;
use apca::{
    api::v2::{
        account,
        asset::{self},
        clock, order,
        position::{self, Position},
        positions,
    },
    Client,
};
use futures::{stream, StreamExt};
use log::info;
use std::collections::HashSet;

pub async fn get_account(client: &Client) -> account::Account {
    client.issue::<account::Get>(&()).await.unwrap()
}

pub async fn get_positions(client: &Client) -> Vec<Position> {
    client.issue::<positions::Get>(&()).await.unwrap()
}

pub async fn are_markets_open(client: &Client) -> bool {
    let response = client.issue::<clock::Get>(&()).await.unwrap();
    info!("are the markets open? : {:?}", response.open);
    response.open
}

pub async fn balance_portfolio(client: &Client, valid_zacks_buys: Vec<asset::Asset>) {
    let account = get_account(client).await;

    info!("account deets: {:#?}", account);

    let buying_power = (account.equity.to_f64().unwrap() * 0.98) as i32;
    info!("current buying power: {buying_power}");

    let rebalance_threshold = (buying_power as f64 * 0.001).round() as i32;

    let desired_allocation = buying_power / valid_zacks_buys.len() as i32;

    info!(
        "buying power: {}... rebalance threshold: {}... desired allocation: {}...",
        buying_power, rebalance_threshold, desired_allocation
    );

    /*
     * Create 2 hashmaps for open and buys mapping the asset symbol to the asset itself.
     * Loop through both hashmaps.
     *
     * Vec<LIQUIDATE>       open
     * Vec<SELL>            buys | open
     * Vec<BUY>             buys | open
     * Vec<BUY_COMPLETE>    buys
     *
     * - Loop through open_positions.
     *      - If a symbol is not in zacks buys, push into Vec<LIQUIDATE>
     *      - If a symbol is in zacks_buys AND amount > (desired_allocation + rebalance_threshold), push into Vec<SELL>
     *      - If a symbol is in zacks_buys AND amount < (desired_allocation + rebalance_threshold), push into Vec<BUY>
     * - Loop through zacks_buys.
     *      - If a symbol is not in open_positions, push into Vec<BUY_COMPLETE>
     *
     * Loop through each vector sequentially.
     * Create a stream of orders for each Vec, wait until all completed before moving on to next stage.
     *
     */

    let mut valid_zacks_buys_map = HashSet::new();
    for asset in &valid_zacks_buys {
        valid_zacks_buys_map.insert(asset.id);
    }

    let open_positions = get_positions(client).await;
    let mut open_positions_map = HashSet::new();
    for position in &open_positions {
        open_positions_map.insert(position.asset_id);
    }

    let mut liquidate: Vec<Position> = Vec::new();
    let mut sell: Vec<Position> = Vec::new();
    let mut buy: Vec<Position> = Vec::new();

    // For all assets to buy that we dont have a current position in
    let mut buy_complete: Vec<asset::Asset> = Vec::new();

    for position in open_positions {
        if !valid_zacks_buys_map.contains(&position.asset_id) {
            liquidate.push(position);
        } else {
            let market_val = match &position.market_value {
                Some(val) => val,
                None => {
                    info!("Couldn't get the number");
                    continue;
                }
            };

            let is_within_rebalance_threshold =
                ((market_val - desired_allocation).to_f64().unwrap().abs())
                    < rebalance_threshold.into();

            if is_within_rebalance_threshold {
                // info!("{:#?} is within rebalance threshold", position.symbol);
                // info!("{:#?}", (market_val));
                continue;
            };

            match market_val > &desired_allocation.into() {
                true => sell.push(position),
                false => buy.push(position),
            }
        }
    }

    for asset in valid_zacks_buys {
        if !open_positions_map.contains(&asset.id) {
            buy_complete.push(asset)
        }
    }

    /**
     * Make the api calls here
     */

    const CONCURRENT_REQUESTS: usize = 5;

    let liquidated_positions = stream::iter(liquidate)
        .map(|position| {
            let symbol = asset::Symbol::Sym(position.symbol);
            async move { client.issue::<position::Delete>(&symbol).await }
        })
        .buffered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    info!(
        "Liquidated positions length: {:#?}\n
            Liquidated positions: {:#?}",
        liquidated_positions.len(),
        liquidated_positions
    );

    let balanced_down_positions = stream::iter(sell)
        .map(|position| {
            let symbol = asset::Symbol::Sym(position.symbol);
            let current_price = position.market_value.unwrap();
            let notional_sell_amount = current_price - desired_allocation;

            let request_builder = order::OrderReqInit {
                type_: order::Type::Market,
                ..Default::default()
            }
            .init(
                symbol.to_string(),
                order::Side::Sell,
                order::Amount::notional(notional_sell_amount),
            );
            async move { client.issue::<order::Post>(&request_builder).await }
        })
        .buffered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    info!(
        "Balanced down positions length: {:#?}\n
            Balanced down positions: {:#?}",
        balanced_down_positions.len(),
        balanced_down_positions
    );

    let balanced_up_positions = stream::iter(buy)
        .map(|position| {
            let symbol = asset::Symbol::Sym(position.symbol);
            let current_price = position.market_value.unwrap();
            let notional_buy_amount = desired_allocation - current_price.to_integer();

            let request_builder = order::OrderReqInit {
                type_: order::Type::Market,
                ..Default::default()
            }
            .init(
                symbol.to_string(),
                order::Side::Buy,
                order::Amount::notional(notional_buy_amount),
            );
            async move { client.issue::<order::Post>(&request_builder).await }
        })
        .buffered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    info!(
        "Balanced up positions length: {:#?}\n
        Balanced up positions: {:#?}",
        balanced_up_positions.len(),
        balanced_up_positions
    );

    let completely_bought_positions = stream::iter(buy_complete)
        .map(|position| {
            let symbol = asset::Symbol::Sym(position.symbol);
            let notional_buy_amount = desired_allocation;

            let request_builder = order::OrderReqInit {
                type_: order::Type::Market,
                ..Default::default()
            }
            .init(
                symbol.to_string(),
                order::Side::Buy,
                order::Amount::notional(notional_buy_amount),
            );
            async move { client.issue::<order::Post>(&request_builder).await }
        })
        .buffered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    info!(
        "Completely bought length: {:#?}\n
        Completely bought positions: {:#?}",
        completely_bought_positions.len(),
        completely_bought_positions
    );
}

pub async fn get_valid_stocks_from_list(
    client: &Client,
    list_struct: ZacksBuys,
) -> Vec<asset::Asset> {
    const CONCURRENT_REQUESTS: usize = 5;

    let bodies = stream::iter(list_struct.list)
        .map(|stock| {
            let symbol = asset::Symbol::Sym(stock.symbol);
            async move {
                let response = client.issue::<asset::Get>(&symbol).await;
                response
            }
        })
        .buffered(CONCURRENT_REQUESTS);

    let assets_vector = bodies.collect::<Vec<_>>().await;

    info!("all assets from alpaca length: {:#?}", assets_vector.len());

    let filtered_assets: Vec<asset::Asset> = assets_vector
        .into_iter()
        .filter_map(|asset| asset.ok())
        .filter(|asset| asset.tradable && asset.fractionable)
        .collect();

    info!("length of filtered assets: {:#?}", filtered_assets.len());

    filtered_assets
}
