use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};
use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::common::{fill_results, ResourceData, ResourceResult};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct CoinbaseData {
    pub currency: String,
    pub rates: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct CoinbaseResponse {
    pub data: CoinbaseData,
}

pub async fn get_coinbase_prices(
    resources: &Vec<ResourceData>,
    results: &mut HashMap<String, Vec<ResourceResult>>,
    currency: String,
) -> Result<()> {
    let url = Url::parse_with_params(
        "https://api.coinbase.com/v2/exchange-rates",
        &[("currency", currency.clone())],
    )?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let string = String::from_utf8(body)?;
    let value: CoinbaseResponse = serde_json::from_str(&string)?;

    let pair_prices: HashMap<String, String> = value
        .data
        .rates
        .into_iter()
        .filter_map(|(asset, price)| match price.parse::<f64>() {
            Ok(price_as_number) => {
                let price = (1.0 / price_as_number).to_string();
                let pair = format!("{}{}", asset, currency);
                Some((pair, price))
            }
            Err(_) => None,
        })
        .collect();

    fill_results(resources, results, pair_prices)?;

    Ok(())
}
