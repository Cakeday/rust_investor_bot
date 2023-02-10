use apca::{ApiInfo, Client as AlpacaClient};
use dotenv::dotenv;
use flexi_logger::{FileSpec, Logger, Duplicate};
use reqwest::Client;
use log::{info};

mod alpaca_client;
mod csv_handler;
mod zacks_api;

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

    let client = Client::new();
    let api_info = ApiInfo::from_env().unwrap();
    let alpaca_client = AlpacaClient::new(api_info);

    let cookie = zacks_api::get_initial_zacks_cookie(&client).await;
    info!("here is the cookie: {:?}", cookie);
    zacks_api::login_on_zacks(&client, &cookie).await;
    zacks_api::get_run_screen_data(&client, &cookie).await;

    let csv_list = zacks_api::get_csv_list(&client, &cookie).await;



    let zacks_buys = csv_handler::parse_csv(csv_list);

    alpaca_client::get_account(&alpaca_client).await;
    alpaca_client::are_markets_open(&alpaca_client).await;
    let valid_zacks_buys =
        alpaca_client::get_valid_stocks_from_list(&alpaca_client, zacks_buys).await;

    alpaca_client::balance_portfolio(&alpaca_client, valid_zacks_buys).await;

}
