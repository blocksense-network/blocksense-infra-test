use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};

use serde::Deserialize;
use url::Url;

use crate::common::PairPriceData;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct BinanceUsPrice {
    pub symbol: String,
    pub price: String,
}

// Binance US provides wrong price for pair BTC/USD. For the USDT stable coin the price is correct.
// It might mean other pairs with fiat quote might be incorrect
pub async fn get_binance_us_prices() -> Result<PairPriceData> {
    let url = Url::parse("https://api.binance.us/api/v3/ticker/price")?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let string = String::from_utf8(body)?;
    let values: Vec<BinanceUsPrice> = serde_json::from_str(&string)?;
    let response: PairPriceData = values
        .into_iter()
        .filter(|value| !value.symbol.ends_with("USD"))
        .map(|value| (value.symbol, value.price))
        .collect();

    Ok(response)
}
