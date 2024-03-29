use std::{collections::HashMap, fs, path::Path};

use kml::{
    types::{Element, Geometry, Placemark, Point},
    Kml, KmlDocument,
};

use crate::google_places::{DisplayName, GooglePlace, Location};
use crate::write_kml::CUP_STYLE_ID;

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
