use serde_json::Value;
use std::io;
use url::Url;

pub struct CoffeeMapConfig {
    pub kml_batch_size: usize,
    pub katana_search_depth: u8,
    pub katana_requests_per_second: u8,
    pub num_concurrent_katana_fetchers: u8,
    pub num_concurrent_katana_input_processors: u8,
    pub nth_item_to_log: i32,
    pub existing_kml_folder: String,
    pub output_folder: String,
    pub output_prefix: String,
}

#[derive(Debug)]
pub enum CoffeeMapError {
    GoogleHTTPError(String),
    GooglePlaceNotFoundError(String),
    GoogleJsonParseError(String),
    KatanaJsonParseError(serde_json::Error),
    KatanaEndpointParseError(Value),
    KatanaIOError(io::Error),
}

#[derive(Debug)]
pub enum CoffeeMapLogMsg {
    CoffeeMapError(CoffeeMapError),
    Warning(String),
    NoCafeHtmlDetails(Url),
    ExistingPlacemark(String),
}
