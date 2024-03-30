use std::collections::HashMap;
use std::{fs, path::Path};

use kml::{types::Placemark, Kml, KmlDocument};
use std::io;


use crate::write_kml;

const CACHE_FILENAME: &str = "cache.kml";

pub fn update(
    cache_folder: String,
    cache: HashMap<String, Placemark>,
    new_placemarks: &Vec<Placemark>,
) -> io::Result<()> {
    let mut new_cache = HashMap::clone(&cache);

    for placemark in new_placemarks {
        let search_term = placemark
            .attrs
            .get(&"search_term".to_string())
            .map(|search_term| search_term.clone());

        search_term.map(|search_term| new_cache.insert(search_term, placemark.clone()));
    }

    write_kml::generate_kml_document(
        new_cache.into_values().collect::<Vec<Placemark>>(),
        cache_folder,
        CACHE_FILENAME.to_string(),
    )
}

pub fn make_existing_placemarks_hashmap(cache_folder: String) -> HashMap<String, Placemark> {
    let current_kml_folder = Path::new(cache_folder.as_str());
    let existing_placemarks = read_placemarks_in_directory(current_kml_folder);

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

pub fn read_placemarks_in_directory<P: AsRef<Path>>(path: P) -> Vec<Placemark> {
    fs::read_dir(path)
        .unwrap()
        .map(|file_path| read_placemarks_from_file(file_path.unwrap().path()))
        .flatten()
        .collect()
}

pub fn read_placemarks_from_file<P>(path: P) -> Vec<Placemark>
where
    P: AsRef<Path>,
{
    let kml_string = fs::read_to_string(path).unwrap();
    let kml: Kml = kml_string.parse().unwrap();

    parse_placemarks(kml).unwrap()
}

fn parse_placemarks(kml: Kml) -> Option<Vec<Placemark>> {
    let kml_elements = match kml {
        Kml::KmlDocument(KmlDocument {
            version: _,
            attrs: _,
            elements,
        }) => Some(elements),
        _ => None,
    }?;

    let kml_document_elements = match kml_elements.as_slice() {
        [Kml::Document { attrs: _, elements }] => Some(elements),
        _ => None,
    }?;

    let placemarks = kml_document_elements
        .into_iter()
        .filter_map(|element| match element {
            Kml::Placemark(placemark) => Some(placemark.to_owned()),
            _ => None,
        })
        .into_iter()
        .collect();

    Some(placemarks)
}
