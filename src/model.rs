use kml::types::Placemark;
use serde_json::Value;
use std::io;


use superconsole::{
    components::{bordering::BorderedSpec, splitting::SplitKind, Bordered, Split},
    Component, Dimensions, Direction, DrawMode, Line, Lines,
};

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

#[derive(Clone)]
pub enum SearchTerm {
    UrlFragment(String),
    CafeDetails(String),
}

impl SearchTerm {
    pub fn extract_str(&self) -> &String {
        match self {
            SearchTerm::UrlFragment(str) => str,
            SearchTerm::CafeDetails(str) => str,
        }
    }
}

pub enum PlacemarkComputation {
    FromCache(SearchTerm, Placemark),
    FromGoogleQuery(SearchTerm, Placemark),
}

impl PlacemarkComputation {
    pub fn to_placemark(self) -> Placemark {
        match self {
            Self::FromCache(_, placemark) => placemark,
            Self::FromGoogleQuery(_, placemark) => placemark,
        }
    }
}

struct TableColumn {
    values: Vec<String>,
}

impl Component for TableColumn {
    fn draw_unchecked(&self, _dimensions: Dimensions, _mode: DrawMode) -> anyhow::Result<Lines> {
        let lines = Lines(
            self.values
                .to_owned()
                .into_iter()
                .map(|value| vec![value].try_into().unwrap())
                .collect::<Vec<Line>>(),
        );

        Ok(lines)
    }
}

#[derive(Clone)]
pub struct LogCounts {
    cached_with_url: i32,
    cached_with_cafe_details: i32,
    queried_with_url: i32,
    queried_with_cafe_details: i32,
    google_http_errors: i32,
    place_not_found_errors: i32,
    google_json_parse_errors: i32,
    katana_json_parse_errors: i32,
    katana_endpoint_parse_errors: i32,
    katana_io_errors: i32,
}

impl LogCounts {
    pub fn update(&self, placemark: &Result<PlacemarkComputation, CoffeeMapError>) -> LogCounts {
        let mut updated = LogCounts::clone(&self);

        match placemark {
            Ok(PlacemarkComputation::FromCache(SearchTerm::CafeDetails(_), _)) => {
                updated.cached_with_cafe_details += 1
            }
            Ok(PlacemarkComputation::FromCache(SearchTerm::UrlFragment(_), _)) => {
                updated.cached_with_url += 1
            }
            Ok(PlacemarkComputation::FromGoogleQuery(SearchTerm::CafeDetails(_), _)) => {
                updated.queried_with_cafe_details += 1
            }
            Ok(PlacemarkComputation::FromGoogleQuery(SearchTerm::UrlFragment(_), _)) => {
                updated.queried_with_url += 1
            }
            Err(CoffeeMapError::GoogleHTTPError(_)) => updated.google_http_errors += 1,
            Err(CoffeeMapError::GooglePlaceNotFoundError(_)) => updated.place_not_found_errors += 1,
            Err(CoffeeMapError::GoogleJsonParseError(_)) => updated.google_json_parse_errors += 1,
            Err(CoffeeMapError::KatanaJsonParseError(_)) => updated.katana_json_parse_errors += 1,
            Err(CoffeeMapError::KatanaEndpointParseError(_)) => {
                updated.katana_endpoint_parse_errors += 1
            }
            Err(CoffeeMapError::KatanaIOError(_)) => updated.katana_io_errors += 1,

            _ => {}
        };

        updated
    }

    pub fn new() -> LogCounts {
        LogCounts {
            cached_with_url: 0,
            cached_with_cafe_details: 0,
            queried_with_url: 0,
            queried_with_cafe_details: 0,
            google_http_errors: 0,
            place_not_found_errors: 0,
            google_json_parse_errors: 0,
            katana_json_parse_errors: 0,
            katana_endpoint_parse_errors: 0,
            katana_io_errors: 0,
        }
    }

    pub fn make_component(&self) -> Box<Split> {
        let stat_names = vec![
            "cached_with_url",
            "cached_with_cafe_details",
            "queried_with_url",
            "queried_with_cafe_details",
            "google_http_errors",
            "place_not_found_errors",
            "google_json_parse_errors",
            "katana_json_parse_errors",
            "katana_endpoint_parse_errors",
            "katana_io_errors",
        ]
        .into_iter()
        .map(|stat_name| stat_name.to_string())
        .collect::<Vec<String>>();

        let stat_values = vec![
            self.cached_with_url,
            self.cached_with_cafe_details,
            self.queried_with_url,
            self.queried_with_cafe_details,
            self.google_http_errors,
            self.place_not_found_errors,
            self.google_json_parse_errors,
            self.katana_json_parse_errors,
            self.katana_endpoint_parse_errors,
            self.katana_io_errors,
        ]
        .into_iter()
        .map(|stat_name| stat_name.to_string())
        .collect::<Vec<String>>();

        let left_column = TableColumn { values: stat_names };
        let left_component = Bordered::new(left_column, BorderedSpec::default());

        let right_column = TableColumn {
            values: stat_values,
        };
        let right_component = Bordered::new(right_column, BorderedSpec::default());

        Box::new(Split::new(
            vec![Box::new(left_component), Box::new(right_component)],
            Direction::Horizontal,
            SplitKind::Adaptive,
        ))
    }
}
