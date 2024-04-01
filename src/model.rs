use kml::types::Placemark;
use serde_json::Value;
use std::{hash::Hasher, io};

use std::hash::Hash;

pub struct CoffeeMapConfig {
    pub kml_batch_size: usize,
    pub katana_search_depth: u8,
    pub katana_requests_per_second: u8,
    pub cache_folder: Option<String>,
    pub output_folder: String,
    pub output_prefix: String,
}

#[derive(Debug)]
pub enum PipelineError {
    GoogleHTTPError(String),
    GooglePlaceNotFoundError(String),
    GoogleJsonParseError(String),
    KatanaJsonParseError(serde_json::Error),
    KatanaEndpointParseError(Value),
    KatanaIOError(io::Error),
}

#[derive(Debug)]
pub enum IOError {
    SuperConsoleNotTTY,
    KMLFileCreation(io::Error),
    CreateMissingDirectories(io::Error),
    KMLWriteError(kml::Error),
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

    fn get_placemark(&self) -> &Placemark {
        match self {
            Self::FromCache(_, placemark) => &placemark,
            Self::FromGoogleQuery(_, placemark) => &placemark,
        }
    }

    pub fn get_id(&self) -> Option<&String> {
        self.get_placemark().attrs.get("id")
    }

    pub fn get_search_term(&self) -> &SearchTerm {
        match &self {
            Self::FromCache(searchterm, _) => searchterm,
            Self::FromGoogleQuery(searchterm, _) => searchterm,
        }
    }
}

impl Hash for PlacemarkComputation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl PartialEq for PlacemarkComputation {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Eq for PlacemarkComputation {}
