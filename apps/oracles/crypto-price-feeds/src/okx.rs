use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::common::{fill_results, ResourceData, ResourceResult};
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct OKXInstrument {
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct OKXInstrumentResponse {
    pub code: String,
    pub data: Vec<OKXInstrument>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct OKXTickerData {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "idxPx")]
    pub idx_px: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct OKXTickerResponse {
    pub code: String,
    pub data: Vec<OKXTickerData>,
}

pub async fn get_okx_symbols() -> Result<Vec<String>> {
    let url = Url::parse("https://www.okx.com/api/v5/public/instruments?instType=SPOT")?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let body_as_string = String::from_utf8(body)?;
    let okx_response: OKXInstrumentResponse = serde_json::from_str(&body_as_string)?;
    let symbols: Vec<String> = okx_response
        .data
        .into_iter()
        .map(|symbol| symbol.inst_id)
        .collect();
    Ok(symbols)
}

pub async fn fetch_okx_price(symbol: String) -> Option<(String, String)> {
    let url = format!(
        "https://www.okx.com/api/v5/market/index-tickers?instId={}",
        symbol
    );
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

    let response: OKXTickerResponse = serde_json::from_str(&body_as_string).ok()?;
    let data = response.data.first()?;

    Some((data.inst_id.clone(), data.idx_px.clone()))
}

pub async fn get_okx_prices(
    resources: &Vec<ResourceData>,
    results: &mut HashMap<String, Vec<ResourceResult>>,
) -> Result<()> {
    let symbols = get_okx_symbols().await?;

    let mut prices = HashMap::new();
    let mut futures = FuturesUnordered::new();

    // Spawn all fetches concurrently
    for symbol in symbols {
        futures.push(fetch_okx_price(symbol));
    }

    // Collect results as they complete
    while let Some(result) = futures.next().await {
        if let Some((symbol, price)) = result {
            prices.insert(symbol.replace("-", ""), price);
        }
    }

    fill_results(resources, results, prices)?;

    Ok(())
}
