use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};

use serde::Deserialize;
use url::Url;

use crate::common::PairPriceData;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct MEXCPrice {
    pub symbol: String,
    pub price: String,
}

pub async fn get_mexc_prices() -> Result<PairPriceData> {
    let url = Url::parse("https://api.mexc.com/api/v3/ticker/price")?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let string = String::from_utf8(body)?;
    let values: Vec<MEXCPrice> = serde_json::from_str(&string)?;
    let response: PairPriceData = values
        .into_iter()
        .map(|value| (value.symbol, value.price))
        .collect();

    Ok(response)
}
