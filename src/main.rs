use katana_stream::{ECTCafeDetails, ECTCafeResult};
use kml::{
    types::{
        ColorMode, Element, Geometry, Icon, IconStyle, KmlDocument, KmlVersion, LabelStyle, Pair,
        Placemark, Point, Style, StyleMap,
    },
    Kml, KmlWriter,
};
use model::CoffeeMapConfig;
use terminal_gui::LogCounts;

use reqwest::blocking;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader};
use std::{collections::HashMap, future, thread, time::Duration};
use std::{env, error::Error};
use std::{fs::File, process::ChildStdout};
use std::{
    ops::Deref,
    process::{Command, Stdio},
};
use std::{path::Path, rc::Rc};
use url::Url;

use crate::katana_stream::KatanaStream;
use crate::model::{CoffeeMapError, PlacemarkComputation, SearchTerm};

use std::convert::TryInto;
use superconsole::components::bordering::{Bordered, BorderedSpec};
use superconsole::{Component, Dimensions, DrawMode, Lines, SuperConsole};

mod cafe_placemarks;
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
        katana_search_depth: 2,
        katana_requests_per_second: 40,
        num_concurrent_katana_fetchers: 10,
        num_concurrent_katana_input_processors: 10,
        nth_item_to_log: 10,
        existing_kml_folder: "./kml".to_string(),
        output_folder: "./ect_30_03_2".to_string(),
        output_prefix: "ect".to_string(),
    };

    let placemarks = crawl_cafes(&config, google_api_key);

    write_kml::generate_kml_documents(&config, placemarks);

    Ok(())
}

#[derive(Debug)]
struct PlacemarkWithProcessInfo {
    searchterm: String,
    placemark: Placemark,
    cached_result: bool,
    queried_from_html_details: bool,
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

fn crawl_cafes(config: &CoffeeMapConfig, google_api_key: &String) -> Vec<Placemark> {
    let cached_searchterm_to_placemark =
        make_existing_placemarks_hashmap(&config.existing_kml_folder);
    let client = blocking::Client::new();

    let mut placemark_results = HashMap::<String, PlacemarkComputation>::new();
    let mut computation_log = LogCounts::new();

    let mut superconsole = SuperConsole::new()
        .ok_or_else(|| anyhow::anyhow!("Not a TTY"))
        .unwrap();

    let placemarks = KatanaStream::new(&config)
        .filter_map(|katana_result| {
            let placemark_result = process_katana_result(
                katana_result,
                google_api_key,
                &client,
                &cached_searchterm_to_placemark,
            );

            computation_log = computation_log.update(&placemark_result);
            superconsole.render(&computation_log.make_component());

            placemark_result.ok()
        })
        .map(|placemark_computation| placemark_computation.to_placemark())
        .collect::<Vec<Placemark>>();

    superconsole.finalize(&computation_log.make_component());

    placemarks
}

fn make_existing_placemarks_hashmap(output_folder: &String) -> HashMap<String, Placemark> {
    let current_kml_folder = Path::new(output_folder.as_str());
    let existing_placemarks = cafe_placemarks::read_placemarks_in_directory(current_kml_folder);

    println!(
        "existing coffee map contains {} entries",
        existing_placemarks.len()
    );

    existing_placemarks
        .into_iter()
        .filter_map(|placemark| {
            let searchterm = placemark.attrs.get("search_term")?;

            Some((searchterm.clone(), placemark))
        })
        .collect()
}

fn make_searchterm(katana_cafe: ECTCafeResult) -> SearchTerm {
    match katana_cafe {
        ECTCafeResult {
            details: Some(cafe_details),
            endpoint,
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
