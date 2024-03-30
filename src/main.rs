use katana_stream::ECTCafeResult;
use kml::types::Placemark;
use model::CoffeeMapConfig;
use terminal_gui::LogCounts;

use reqwest::blocking;

use std::collections::{HashMap, HashSet};
use std::env;



use crate::katana_stream::KatanaStream;
use crate::model::{CoffeeMapError, PlacemarkComputation, SearchTerm};

use superconsole::SuperConsole;

mod cache;
mod google_places;
mod katana_stream;
mod model;
mod terminal_gui;
mod write_kml;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let google_api_key = &args[1];

    let config = CoffeeMapConfig {
        kml_batch_size: 1000,
        katana_search_depth: 12,
        katana_requests_per_second: 40,
        cache_folder: Some("./kml/cache/".to_string()),
        output_folder: "./kml/30_03_3/".to_string(),
        output_prefix: "placemarks".to_string(),
    };

    let cache = match &config.cache_folder {
        Some(folder) => cache::make_existing_placemarks_hashmap(folder.clone()),
        None => HashMap::<String, Placemark>::new(),
    };

    let placemarks = crawl_cafes(&config, google_api_key, &cache)
        .into_iter()
        .map(|computation| computation.to_placemark())
        .collect::<Vec<Placemark>>();

    config
        .cache_folder
        .as_ref()
        .map(|folder| cache::update(folder.clone(), cache, &placemarks));

    write_kml::generate_kml_documents(&config, placemarks);

    Ok(())
}

fn crawl_cafes(
    config: &CoffeeMapConfig,
    google_api_key: &String,
    cached_search_terms: &HashMap<String, Placemark>,
) -> HashSet<PlacemarkComputation> {
    let client = blocking::Client::new();

    let mut computation_log = LogCounts::new();
    let mut superconsole = SuperConsole::new()
        .ok_or_else(|| anyhow::anyhow!("Not a TTY"))
        .unwrap();

    let placemarks = KatanaStream::new(&config)
        .filter_map(|katana_result| {
            let placemark_result =
                process_katana_result(katana_result, google_api_key, &client, cached_search_terms);

            computation_log = computation_log.update(&placemark_result);
            superconsole.render(&computation_log.make_component());

            placemark_result.ok()
        })
        .collect::<HashSet<PlacemarkComputation>>();

    superconsole.finalize(&computation_log.make_component());

    placemarks
}

fn process_katana_result(
    katana_result: Result<ECTCafeResult, CoffeeMapError>,
    google_api_key: &String,
    client: &blocking::Client,
    searchterm_to_placemark: &HashMap<String, Placemark>,
) -> Result<PlacemarkComputation, CoffeeMapError> {
    let katana_cafe = katana_result?;

    let search_term = make_searchterm(katana_cafe);
    let search_term_str = search_term.extract_str();

    if let Some(existing_placemark) = searchterm_to_placemark.get(search_term_str) {
        let cloned_placemark = existing_placemark.clone();

        Ok(PlacemarkComputation::FromCache(
            search_term,
            cloned_placemark,
        ))
    } else {
        google_places::query(&client, search_term_str.clone(), google_api_key.clone()).map({
            |google_place| {
                let placemark = google_place.to_placemark();

                PlacemarkComputation::FromGoogleQuery(search_term, placemark)
            }
        })
    }
}

fn make_searchterm(katana_cafe: ECTCafeResult) -> SearchTerm {
    match katana_cafe {
        ECTCafeResult {
            details: Some(cafe_details),
            endpoint: _,
        } => {
            let search_string = format!("{} {}", &cafe_details.name, &cafe_details.address);
            SearchTerm::CafeDetails(search_string)
        }
        ECTCafeResult {
            details: _,
            endpoint,
        } => {
            let search_string = endpoint
                .path_segments()
                .unwrap()
                .nth(1)
                .unwrap()
                .to_string()
                .replace("-", " ");
            SearchTerm::UrlFragment(search_string)
        }
    }
}
