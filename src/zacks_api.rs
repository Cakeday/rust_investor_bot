use log::info;
use reqwest::header::{
    HeaderMap, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE,
    COOKIE, HOST, ORIGIN, REFERER, SET_COOKIE, UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};
use reqwest::multipart;
use reqwest::multipart::{Form, Part};
use reqwest::Client;

use std::env;

pub async fn get_initial_zacks_cookie(client: &Client) -> String {
    let resp = client.get(env::var("ZACKS_REFERER_URL").expect("No .env var for Zacks referer URL!"))
        .send().await.unwrap();

    let cookie = resp.headers().get(SET_COOKIE).unwrap();
    let cookie = String::from(cookie.to_str().unwrap());
    let cookie = cookie.split(' ').next().unwrap().to_string();
    cookie
}

pub async fn login_on_zacks(client: &Client, cookie: &String) {
    let multipart_form = Form::new()
        .text(
            "username",
            env::var("ZACKS_USERNAME").expect("No .env var for Zacks username!"),
        )
        .text(
            "password",
            env::var("ZACKS_PASSWORD").expect("No .env var for Zacks password!"),
        );

    let boundary = multipart_form.boundary().to_owned();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "*/*".parse().unwrap());
    headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(CONTENT_LENGTH, "275".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        format!("multipart/form-data; boundary={boundary}")
            .parse()
            .unwrap(),
    );

    let host = env::var("ZACKS_HOST").expect("No .env var for Zacks host!");
    let screener_url = env::var("ZACKS_SCREENER_URL").expect("No .env var for Zacks screener URL!");
    let referer_url = env::var("ZACKS_REFERER_URL").expect("No .env var for Zacks referer URL!");

    headers.insert(HOST, host.parse().unwrap());
    headers.insert(ORIGIN, screener_url.parse().unwrap());
    headers.insert(REFERER, referer_url.parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36".parse().unwrap());

    let login_cookie = format!("{} CURRENT_POS=my_screen", cookie);

    headers.insert(COOKIE, login_cookie.parse().unwrap());

    info!("cookie...: {:#?}", login_cookie);

    let screener_url = env::var("ZACKS_SCREENER_URL").expect("No .env var for Zacks screener URL!");

    let response = client
        .post(format!("{screener_url}/login_action.php"))
        .multipart(multipart_form)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    info!("{:?}", response)
}

pub async fn get_run_screen_data(client: &Client, cookie: &str) {
    let multipart_form = multipart::Form::new()
        .part("is_only_matches", Part::text("0"))
        .part("is_premium_exists", Part::text("0"))
        .part("is_edit_view", Part::text("0"))
        .part("saved_screen_name", Part::text("Top Industry and Momentum"))
        .part("tab_id", Part::text("1"))
        .part("start_page", Part::text("1"))
        .part("no_of_rec", Part::text("100"))
        .part("sort_col", Part::text("2"))
        .part("sort_type", Part::text("ASC"))
        .part("p_items[]", Part::text("15005"))
        .part("p_item_name[]", Part::text("Zacks Rank"))
        .part("p_item_key[]", Part::text("0"))
        .part("operator[]", Part::text("8"))
        .part("value[]", Part::text("1"))
        .part("p_items[]", Part::text("15025"))
        .part("p_item_name[]", Part::text("Zacks Industry Rank"))
        .part("p_item_key[]", Part::text("1"))
        .part("operator[]", Part::text("7"))
        .part("value[]", Part::text("132"))
        .part("p_items[]", Part::text("15040"))
        .part("p_item_name[]", Part::text("Momentum Score"))
        .part("p_item_key[]", Part::text("4"))
        .part("operator[]", Part::text("19"))
        .part("value[]", Part::text("A"))
        .part("config_params", Part::text("[]"))
        .part("load_scr_id", Part::text("170751"));

    let boundary = multipart_form.boundary().to_owned();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "*/*".parse().unwrap());
    headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(CONTENT_LENGTH, "2702".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        format!("multipart/form-data; boundary={boundary}")
            .parse()
            .unwrap(),
    );

    let host = env::var("ZACKS_HOST").expect("No .env var for Zacks host!");
    let screener_url = env::var("ZACKS_SCREENER_URL").expect("No .env var for Zacks screener URL!");
    let referer_url = env::var("ZACKS_REFERER_URL").expect("No .env var for Zacks referer URL!");

    headers.insert(HOST, host.parse().unwrap());
    headers.insert(ORIGIN, screener_url.parse().unwrap());
    headers.insert(REFERER, referer_url.parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert(UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());

    headers.insert(COOKIE, cookie.parse().unwrap());

    client
        .post(format!("{screener_url}/getrunscreendata.php"))
        .headers(headers)
        .multipart(multipart_form)
        .send()
        .await
        .unwrap();
}

pub async fn get_csv_list(client: &Client, cookie: &str) -> String {
    let host = env::var("ZACKS_HOST").expect("No .env var for Zacks host!");
    let screener_url = env::var("ZACKS_SCREENER_URL").expect("No .env var for Zacks screener URL!");

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".parse().unwrap());
    headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(CONTENT_LENGTH, "2702".parse().unwrap());
    headers.insert(HOST, host.parse().unwrap());
    headers.insert(ORIGIN, screener_url.parse().unwrap());
    headers.insert(REFERER, env::var("ZACKS_REFERER_URL").expect("No .env var for Zacks referer URL!").parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36".parse().unwrap());

    headers.insert(COOKIE, cookie.parse().unwrap());

    let csv_list = client
        .get(format!("{screener_url}/export.php"))
        .headers(headers)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    info!("resp from csv: {:#?}", csv_list);
    csv_list
}
