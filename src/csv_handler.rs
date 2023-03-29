use csv;
use log::{info, warn};
use serde::Deserialize;
use std::{process, fs, env};

#[derive(Debug, Deserialize)]
pub struct Stock {
    pub company_name: String,
    pub symbol: String,
    pub zacks_rank: String,
    pub industry_rank: String,
    pub momentum: String,
}

#[derive(Debug, Deserialize)]
pub struct ZacksBuys {
    pub list: Vec<Stock>,
}

pub fn read_csv() -> String {
    let csv_file = env::var("CSV_FILE").expect("No .env var for Zacks referer URL!");
    let file_path = &format!("{}", csv_file);

    info!("file_path: {}", file_path);

    let csv_string = match fs::read_to_string(file_path) {
        Ok(csv_string) => csv_string,
        Err(e) => {
            warn!("Error reading csv: {}", e);
            process::exit(1)
        }
    };

    // fs::remove_file(file_path);

    csv_string
}

pub fn parse_csv(zacks_list: String) -> ZacksBuys {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .double_quote(false)
        .escape(Some(b'\\'))
        .from_reader(zacks_list.as_bytes());

    // println!("{:?}", reader.headers());

    let mut structured_stock_vector = ZacksBuys { list: Vec::new() };

    for record in reader.records() {
        let record = record.unwrap();
        let record = Stock {
            company_name: record[0].to_string(),
            symbol: record[1].to_string(),
            zacks_rank: record[2].to_string(),
            industry_rank: record[3].to_string(),
            momentum: record[4].to_string(),
        };
        structured_stock_vector.list.push(record);
    }

    // println!("{:#?}", structured_stock_vector);
    // println!("{:#?}", structured_stock_vector.list.len());

    structured_stock_vector
}
