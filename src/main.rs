use apca::{ApiInfo, Client as AlpacaClient};
use dotenv::dotenv;
use flexi_logger::{FileSpec, Logger, Duplicate};
use log::{info, warn};
use std::process;

mod alpaca_client;
mod csv_handler;

#[tokio::main]
async fn main() {
    dotenv().ok();

    Logger::try_with_str("info")
        .unwrap() // Write all messages
        // use a simple filename without a timestamp
        .log_to_file(FileSpec::default().suppress_timestamp())
        // do not truncate the log file when the program is restarted
        .append()
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .unwrap();

    let api_info = ApiInfo::from_env().unwrap();
    let alpaca_client = AlpacaClient::new(api_info);

    let csv_list = csv_handler::read_csv();

    let zacks_buys = csv_handler::parse_csv(csv_list);

    info!("Zacks Buys: {:#?}", zacks_buys);

    if zacks_buys.list.len() < 1 {
        warn!("There were no buys in zacks buys!");
        process::exit(1);
    }

    alpaca_client::get_account(&alpaca_client).await;
    alpaca_client::are_markets_open(&alpaca_client).await;
    let valid_zacks_buys =
        alpaca_client::get_valid_stocks_from_list(&alpaca_client, zacks_buys).await;

    alpaca_client::balance_portfolio(&alpaca_client, valid_zacks_buys).await;

}
