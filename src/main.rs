use itertools::Itertools;
use kml::{
    types::{
        ColorMode, Element, Geometry, Icon, IconStyle, KmlDocument, KmlVersion, LabelStyle, Pair,
        Placemark, Point, Style, StyleMap,
    },
    Kml, KmlWriter,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{collections::HashMap, io::Result};

const CUP_STYLE_ID: &str = "icon-1534-0288D1";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let google_api_key = &args[1];
    let filename = &args[2];

    if let Ok(lines) = read_lines(filename) {
        let client = reqwest::blocking::Client::new();
        let mut count: i32 = 0;
        let mut failures: i32 = 0;
        let placemarks: Vec<Kml> = lines
            .flatten()
            .map(|search_term| {
                count += 1;
                let places = get_google_place_details(&client, search_term, &google_api_key);

                if places.is_empty() {
                    failures += 1;
                }

                println!("count: {} failures: {}", &count, &failures);
                places
            })
            .flatten()
            .map(|place| generate_placemarks(place))
            .map(|placemark| Kml::Placemark::<f64>(placemark))
            .collect();

        generate_kml_documents(placemarks)?
    }

    Ok(())
}

fn generate_kml_documents(placemarks: Vec<Kml>) -> Result<()> {
    let filename_base = "kml/ect";
    let mut chunk_id = 0;

    for placemarks_chunk in &placemarks.into_iter().chunks(1000) {
        let mut attrs = HashMap::<String, String>::new();
        attrs.insert(
            "xmlns".to_string(),
            "http://www.opengis.net/kml/2.2".to_string(),
        );
        let name_tag = Kml::Element(Element {
            name: "name".to_string(),
            attrs: HashMap::<String, String>::new(),
            content: Some(format!("ECT places {}", chunk_id)),
            children: vec![],
        });

        let style_tags = generate_styles();

        let mut elements = vec![name_tag];
        elements.extend(style_tags.into_iter());
        elements.extend(placemarks_chunk.into_iter());

        let doc = Kml::Document {
            attrs: HashMap::<String, String>::new(),
            elements,
        };

        let document = KmlDocument {
            version: KmlVersion::V22,
            attrs,
            elements: vec![doc],
        };

        let mut file = File::create(format!("{}_chunk_{}.kml", filename_base, chunk_id))?;
        chunk_id += 1;

        let mut writer = KmlWriter::from_writer(&mut file);

        writer.write(&Kml::KmlDocument(document));
    }

    Ok(())
}

fn generate_icon_style(id: &String, scale: f64) -> Kml {
    Kml::Style(Style {
        id: Some(id.clone()),
        balloon: None,
        icon: Some(IconStyle {
            id: None,
            scale: 1.0,
            heading: 1.0,
            hot_spot: None,
            icon: Icon {
                href: "https://www.gstatic.com/mapspro/images/stock/503-wht-blank_maps.png"
                    .to_string(),
                attrs: HashMap::<String, String>::new(),
            },
            color: "ffd18802".to_string(),
            color_mode: ColorMode::Normal,
            attrs: HashMap::<String, String>::new(),
        }),
        label: Some(LabelStyle {
            id: None,
            color: "ffd18802".to_string(),
            color_mode: ColorMode::Normal,
            scale,
            attrs: HashMap::<String, String>::new(),
        }),
        line: None,
        poly: None,
        list: None,
        attrs: HashMap::<String, String>::new(),
    })
}

fn generate_styles() -> Vec<Kml> {
    let normal_style_id = "icon-1534-0288D1-normal".to_string();
    let highlight_style_id = "icon-1534-0288D1-highlight".to_string();

    let normal_style = generate_icon_style(&normal_style_id, 0.0);
    let highlight_style = generate_icon_style(&highlight_style_id, 1.1);

    let style_map = Kml::StyleMap(StyleMap {
        id: Some(CUP_STYLE_ID.to_string()),
        pairs: vec![
            Pair {
                key: "normal".to_string(),
                style_url: format!("#{}", normal_style_id),
                attrs: HashMap::<String, String>::new(),
            },
            Pair {
                key: "highlight".to_string(),
                style_url: format!("#{}", highlight_style_id),
                attrs: HashMap::<String, String>::new(),
            },
        ],
        attrs: HashMap::<String, String>::new(),
    });

    vec![normal_style, highlight_style, style_map]
}

fn generate_placemarks(place_with_st: PlaceWithST) -> Placemark {
    let mut attrs = HashMap::<String, String>::new();
    attrs.insert(String::from("search_term"), place_with_st.search_term);
    attrs.insert("id".to_string(), place_with_st.place.id);

    let geometry = Geometry::Point(Point::new(
        place_with_st.place.location.longitude,
        place_with_st.place.location.latitude,
        Some(0.0),
    ));

    Placemark {
        name: Some(place_with_st.place.displayName.text),
        attrs,
        children: vec![Element {
            name: "styleUrl".to_string(),
            attrs: HashMap::<String, String>::new(),
            content: Some(format!("#{}", CUP_STYLE_ID)),
            children: vec![],
        }],
        description: Some(format!(
            r#"{}
            
            {}"#,
            place_with_st.place.googleMapsUri, place_with_st.place.formattedAddress
        )),
        geometry: Some(geometry),
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Location {
    latitude: f64,
    longitude: f64,
}
#[derive(Deserialize, Debug, Clone)]
struct DisplayName {
    text: String,
    languageCode: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Place {
    id: String,
    displayName: DisplayName,
    formattedAddress: String,
    googleMapsUri: String,
    location: Location,
    types: Vec<String>,
}

struct PlaceWithST {
    search_term: String,
    place: Place,
}

fn get_google_place_details(
    client: &reqwest::blocking::Client,
    search_term: String,
    api_key: &str,
) -> Vec<PlaceWithST> {
    let endpoint = "https://places.googleapis.com/v1/places:searchText";
    let request_body = json!({
        "textQuery": search_term
    });

    let req = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", api_key)
        .header(
            "X-Goog-FieldMask",
            "places.displayName,places.id,places.formattedAddress,places.location,places.googleMapsUri,places.types",
        )
        .body(request_body.to_string());

    let res = req.send();

    let body = res.expect("REASON").bytes();

    let v = body.expect("REASON2").to_vec();
    let s = String::from_utf8_lossy(&v);

    let response: Value = serde_json::from_str(&*s).unwrap();

    let cafe_keyword = "cafe".to_string();
    let coffee_shop_keyword = "coffee_shop".to_string();

    let all_places = match response.get("places") {
        Some(Value::Array(place_list)) if place_list.len() > 0 => {
            let places: Vec<Place> = place_list
                .clone()
                .into_iter()
                .filter_map(
                    |place_value| match serde_json::from_value::<Place>(place_value) {
                        Ok(place) => Some(place),
                        Err(_) => {
                            println!("error, response: {} from search term {}", s, search_term);
                            None
                        }
                    },
                )
                .collect();

            let coffee_shops: Vec<Place> = places
                .clone()
                .into_iter()
                .filter(|place| {
                    place.types.contains(&cafe_keyword) | place.types.contains(&coffee_shop_keyword)
                })
                .collect();

            if coffee_shops.is_empty() {
                if let Some(place) = places.into_iter().next() {
                    vec![place]
                } else {
                    vec![]
                }
            } else {
                coffee_shops
            }
        }
        _ => {
            println!("error, response: {} from search term {}", s, search_term);
            vec![]
        }
    };

    all_places
        .into_iter()
        .map(|place| PlaceWithST {
            search_term: search_term.clone(),
            place,
        })
        .collect()
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
