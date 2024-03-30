use std::{
    io::{Lines},
    io::{BufRead, BufReader},
    process::{ChildStdout, Command, Stdio},
};

use crate::model::CoffeeMapConfig;
use crate::model::CoffeeMapError;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone)]
pub struct ECTCafeDetails {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct ECTCafeResult {
    pub endpoint: Url,
    pub details: Option<ECTCafeDetails>,
}

pub struct KatanaStream {
    reader_lines: Lines<BufReader<ChildStdout>>,
}

impl KatanaStream {
    pub fn new(config: &CoffeeMapConfig) -> Self {
        let search_depth = config.katana_search_depth.clone().to_string();
        let max_requests_per_second = config.katana_requests_per_second.clone().to_string();

        let katana_args = vec![
            "-u",
            "https://europeancoffeetrip.com/cafe",
            "-mr",
            ".*/cafe/.*",
            "-d",
            &search_depth,
            "-rl",
            &max_requests_per_second,
            "-silent",
            "-jsonl", //"-c 20",
                      //"-p 20",
        ];
        let stdout = spawn_katana_process(katana_args).unwrap();
        let reader = BufReader::new(stdout);

        Self {
            reader_lines: reader.lines(),
        }
    }
}

fn spawn_katana_process(args: Vec<&str>) -> Option<ChildStdout> {
    Command::new("katana")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| {
            println!("{:#?}", err);
            err
        })
        .ok()?
        .stdout
}

impl Iterator for KatanaStream {
    type Item = Result<ECTCafeResult, CoffeeMapError>;

    fn next(&mut self) -> Option<Result<ECTCafeResult, CoffeeMapError>> {
        let ect_cafe_details = self
            .reader_lines
            .next()?
            .map_err(CoffeeMapError::KatanaIOError)
            .and_then(parse_katana_output);

        Some(ect_cafe_details)
    }
}

fn parse_katana_output(json_string: String) -> Result<ECTCafeResult, CoffeeMapError> {
    let katana_json: Value = serde_json::from_str(json_string.as_str())
        .map_err(|err| CoffeeMapError::KatanaJsonParseError(err))?;

    let endpoint = parse_katana_endpoint(&katana_json).ok_or(
        CoffeeMapError::KatanaEndpointParseError(katana_json.clone()),
    )?;
    let details = parse_cafe_details(&katana_json);

    Ok(ECTCafeResult { endpoint, details })
}

fn parse_katana_endpoint(katana_json: &Value) -> Option<Url> {
    let endpoint_json = katana_json.get("request")?.get("endpoint")?;
    let endpoint_string = serde_json::from_value::<String>(endpoint_json.clone()).ok()?;
    Url::parse(endpoint_string.as_str()).ok()
}

fn parse_cafe_details(katana_json: &Value) -> Option<ECTCafeDetails> {
    let html_body_json = katana_json.get("response")?.get("body")?.clone();

    let html_body = serde_json::from_value::<String>(html_body_json).ok()?;
    let html = Html::parse_document(html_body.as_str());

    let name = parse_name(&html)?;
    let address = parse_address(&html)?;

    Some(ECTCafeDetails { name, address })
}

fn parse_address(html: &Html) -> Option<String> {
    let address_selector = Selector::parse(r#"div[class="cafe-address"]"#).ok()?;
    html.select(&address_selector)
        .next()?
        .text()
        .next()
        .map(str::trim)
        .map(String::from)
}

fn parse_name(html: &Html) -> Option<String> {
    let address_selector = Selector::parse(r#"h1[class="cafe-name"]"#).ok()?;
    html.select(&address_selector)
        .next()?
        .text()
        .next()
        .map(String::from)
}
