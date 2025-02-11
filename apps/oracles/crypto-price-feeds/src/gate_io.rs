use anyhow::Result;
use blocksense_sdk::spin::http::{send, Method, Request, Response};

use serde::Deserialize;
use url::Url;

use crate::common::PairPriceData;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct GateIoPrice {
    pub currency_pair: String,
    pub last: String,
}

pub async fn get_gate_io_prices() -> Result<PairPriceData> {
    let url = Url::parse("https://api.gateio.ws/api/v4/spot/tickers")?;

    let mut req = Request::builder();
    req.method(Method::Get);
    req.uri(url);
    req.header("Accepts", "application/json");

    let req = req.build();
    let resp: Response = send(req).await?;

    let body = resp.into_body();
    let string = String::from_utf8(body)?;
    let values: Vec<GateIoPrice> = serde_json::from_str(&string)?;
    let response: PairPriceData = values
        .into_iter()
        .map(|value| (value.currency_pair.replace("_", ""), value.last))
        .collect();

    Ok(response)
}
