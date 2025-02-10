use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::common::PairPriceData;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct GeminiResponse {
    pub last: String,
}

pub async fn get_gemini_symbols() -> Result<Vec<String>> {
    let url = Url::parse("https://api.gemini.com/v1/symbols")?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let body_as_string = String::from_utf8(body)?;
    let symbols: Vec<String> = serde_json::from_str(&body_as_string)?;
    let symbols: Vec<String> = symbols
        .into_iter()
        .filter(|symbol| !symbol.ends_with("perp"))
        .map(|symbol| symbol.to_ascii_uppercase())
        .collect();

    Ok(symbols)
}

pub async fn fetch_gemini_price(symbol: String) -> Option<(String, String)> {
    let url = format!("https://api.gemini.com/v1/pubticker/{}", symbol);
    let url = Url::parse(&url).ok()?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await.ok()?;
    if *resp.status() != 200 {
        return None;
    }

    let body = resp.into_body();
    let body_as_string = String::from_utf8(body).ok()?;
    let response: GeminiResponse = serde_json::from_str(&body_as_string).ok()?;

    Some((symbol, response.last))
}

pub async fn get_gemini_prices() -> Result<PairPriceData> {
    let symbols = get_gemini_symbols().await?;

    let mut prices: PairPriceData = HashMap::new();
    let mut futures = FuturesUnordered::new();

    // Spawn all fetches concurrently
    for symbol in symbols {
        futures.push(fetch_gemini_price(symbol));
    }

    // Collect results as they complete
    while let Some(result) = futures.next().await {
        if let Some((symbol, price)) = result {
            prices.insert(symbol, price);
        }
    }

    Ok(prices)
}
