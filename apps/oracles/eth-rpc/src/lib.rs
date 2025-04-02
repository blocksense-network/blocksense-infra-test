use alloy::{
    hex::ToHexExt,
    primitives::{address, Address, Bytes, U256},
    providers::ProviderBuilder,
    sol,
};
use anyhow::{Context, Result};
use blocksense_sdk::{
    http::http_post_json,
    oracle::{DataFeedResult, DataFeedResultValue, Payload, Settings},
    oracle_component,
};
use prettytable::{format, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    YnETHx,
    "src/abi/YnETHx.json"
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEthCallParams {
    data: String,
    from: String,
    to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEthCall {
    pub jsonrpc: String,
    pub method: String,
    pub id: u64,
    pub params: (RequestEthCallParams, String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseEthCallError {
    pub message: String,
    pub code: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseEthCall {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub error: Option<ResponseEthCallError>,
    pub result: Option<String>,
}

fn convert(x: [u8; 32]) -> ([u8; 16], [u8; 16]) {
    let arr: [[u8; 16]; 2] = unsafe { std::mem::transmute(x) };
    (arr[0], arr[1])
}

impl ResponseEthCall {
    pub fn result_as_u256(&self) -> Result<U256> {
        match &self.result {
            Some(v) => Ok(U256::from_str(v)?),
            None => {
                let message = if let Some(err) = &self.error {
                    err.message.clone()
                } else {
                    "Missing error".to_string()
                };
                Err(anyhow::anyhow!(message))
            }
        }
    }

    pub fn result_as_f64(&self) -> Result<f64> {
        let x = self.result_as_u256()?;
        let non_zero_bits = x.bit_len();
        println!("Number of bits = {non_zero_bits}");
        if non_zero_bits > f64::MANTISSA_DIGITS as usize {
            println!("f64 is not big enough to accuratly represent integer with {non_zero_bits} non zero bits");
        }
        if non_zero_bits > u128::BITS as usize {
            return Err(anyhow::anyhow!(
                "u128 is not big enough to fit integer with {non_zero_bits} non zero bits"
            ));
        }
        let be: [u8; 32] = x.to_be_bytes();
        let (_a, b) = convert(be);
        let u = u128::from_be_bytes(b);
        Ok(u as f64)
    }
}

impl RequestEthCall {
    pub fn latest(calldata: &Bytes, contract_address: &Address, id: u64) -> RequestEthCall {
        let params = RequestEthCallParams {
            data: calldata.0.encode_hex_upper_with_prefix(),
            from: "0x0000000000000000000000000000000000000000".to_string(),
            to: contract_address.to_string(),
        };

        RequestEthCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_call".to_string(),
            id,
            params: (params, "latest".to_string()),
        }
    }
}

async fn fetch_all(_resourses: &[FeedConfig]) -> Result<ResponseEthCall> {
    //let rpc_url = "https://rpc.eth.gateway.fm".parse()?;
    let rpc_url: Url = "https://eth.llamarpc.com".parse()?;
    let yn_eth_x_contract_address = address!("0x657d9ABA1DBb59e53f9F3eCAA878447dCfC96dCb");

    // Create a provider, in order to create instance of the contract
    let provider = ProviderBuilder::new().on_http(rpc_url.clone());
    let contact = YnETHx::new(yn_eth_x_contract_address, provider);
    let shares = U256::from(1_000_000_000_000_000_000_u64);

    let x = contact.convertToAssets(shares);
    let calldata = x.calldata().clone();
    let eth_call = RequestEthCall::latest(&calldata, &yn_eth_x_contract_address, 12345);

    let value: ResponseEthCall = http_post_json(rpc_url.as_str(), eth_call).await?;
    Ok(value)
}

fn process_results(resourses: &[FeedConfig], results: &ResponseEthCall) -> Result<Payload> {
    let mut payload: Payload = Payload::new();
    let value = match results.result_as_f64() {
        Ok(x) => DataFeedResultValue::Numerical(x),
        Err(e) => DataFeedResultValue::Error(e.to_string()),
    };
    payload.values.push(DataFeedResult {
        id: resourses[0].feed_id.to_string(),
        value,
    });
    Ok(payload)
}

fn print_results(_resourses: &[FeedConfig], payload: &Payload) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Feed ID").style_spec("bc"),
        Cell::new("Method").style_spec("bc"),
        Cell::new("Value").style_spec("bc"),
    ]));

    for p in payload.values.iter() {
        let id = p.id.clone();
        let value = format!("{:?}", p.value);
        table.add_row(Row::new(vec![
            Cell::new(&id.to_string()).style_spec("r"),
            Cell::new("YnETHx -> convertToAssets").style_spec("r"),
            Cell::new(&value).style_spec("r"),
        ]));
    }

    table.printstd();
}

#[oracle_component]
async fn oracle_request(settings: Settings) -> Result<Payload> {
    println!("Starting oracle component - YnETHx Terminal");
    let resources = get_resources_from_settings(&settings)?;
    let results = fetch_all(&resources).await?;
    let payload = process_results(&resources, &results)?;
    print_results(&resources, &payload);
    Ok(payload)
}

#[derive(Deserialize, Debug)]
pub struct Pair {
    pub base: String,
    pub quote: String,
}

#[derive(Deserialize, Debug)]
struct FeedConfig {
    #[serde(default)]
    pub feed_id: u64,
}

fn get_resources_from_settings(settings: &Settings) -> Result<Vec<FeedConfig>> {
    let mut config: Vec<FeedConfig> = Vec::new();
    for feed_setting in &settings.data_feeds {
        let mut feed_config = serde_json::from_str::<FeedConfig>(&feed_setting.data)
            .context("Couldn't parse data feed")?;
        feed_config.feed_id = feed_setting.id.parse::<u64>()?;
        config.push(feed_config);
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::U256;

    use crate::ResponseEthCall;

    #[test]
    fn test_deserializion_of_error() {
        let raw_response = r#"{"jsonrpc":"2.0","id":100020,"error":{"code":3,"data":"0x4e487b710000000000000000000000000000000000000000000000000000000000000032","message":"execution reverted: panic: array out-of-bounds access (0x32)"}}"#;
        let value: ResponseEthCall = serde_json::from_str(raw_response).unwrap();
        assert_eq!(value.jsonrpc, "2.0");
        assert_eq!(value.id, Some(100020));
        assert!(value.error.is_some());
        let e = value.error.unwrap();
        assert_eq!(e.code, 3);
        assert_eq!(
            e.message,
            "execution reverted: panic: array out-of-bounds access (0x32)"
        );
    }

    #[test]
    fn test_deserializion_of_error_2() {
        let raw_response =
            r#"{"jsonrpc":"","id":null,"error":{"message":"method  not supported","code":-32603}}"#;
        let value: ResponseEthCall = serde_json::from_str(raw_response).unwrap();
        assert!(value.error.is_some());
    }

    #[test]
    fn test_deserializion_of_result() {
        let raw_response = r#"{"jsonrpc":"2.0","id":100,"result":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#;
        let value: ResponseEthCall = serde_json::from_str(raw_response).unwrap();
        assert_eq!(value.jsonrpc, "2.0");
        assert_eq!(value.id, Some(100));
        assert!(value.error.is_none());
        assert!(value.result.is_some());
        assert_eq!(value.result_as_u256().unwrap(), U256::ZERO);
    }
}
