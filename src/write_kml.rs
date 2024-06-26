use itertools::Itertools;
use kml::{
    types::{
        ColorMode, Element, Icon, IconStyle, KmlDocument, KmlVersion, LabelStyle, Pair, Placemark,
        Style, StyleMap,
    },
    Kml, KmlWriter,
};
use std::collections::HashMap;
use std::{
    fs::{self, File},
    path::Path,
};

use crate::model::{CoffeeMapConfig, IOError};

pub const CUP_STYLE_ID: &str = "icon-1534-0288D1";

pub fn generate_kml_documents(
    config: &CoffeeMapConfig,
    placemarks: Vec<Placemark>,
) -> Result<(), IOError> {
    let mut chunk_id = 0;

    for placemarks_chunk in &placemarks.into_iter().chunks(config.kml_batch_size) {
        let placemarks_chunked = placemarks_chunk.into_iter().collect::<Vec<Placemark>>();

        let filename = format!("{}_chunk_{}.kml", &config.output_prefix, chunk_id);
        generate_kml_document(placemarks_chunked, config.output_folder.clone(), filename)?;

        chunk_id += 1;
    }

    Ok(())
}

pub fn generate_kml_document(
    placemarks: Vec<Placemark>,
    folder: String,
    filename: String,
) -> Result<(), IOError> {
    let mut attrs = HashMap::<String, String>::new();
    attrs.insert(
        "xmlns".to_string(),
        "http://www.opengis.net/kml/2.2".to_string(),
    );
    let name_tag = Kml::Element(Element {
        name: "name".to_string(),
        attrs: HashMap::<String, String>::new(),
        content: Some(String::clone(&filename)),
        children: vec![],
    });

    let style_tags = generate_styles();

    let mut elements = vec![name_tag];
    elements.extend(style_tags.into_iter());
    elements.extend(
        placemarks
            .into_iter()
            .map(|placemark| Kml::Placemark(placemark)),
    );

    let doc = Kml::Document {
        attrs: HashMap::<String, String>::new(),
        elements,
    };

    let document = KmlDocument {
        version: KmlVersion::V22,
        attrs,
        elements: vec![doc],
    };

    fs::create_dir_all(Path::new(&folder)).map_err(|err| IOError::CreateMissingDirectories(err))?;

    let mut file = File::create(format!("{}/{}", folder, filename))
        .map_err(|err| IOError::KMLFileCreation(err))?;

    let mut writer = KmlWriter::from_writer(&mut file);

    writer
        .write(&Kml::KmlDocument(document))
        .map_err(|err| IOError::KMLWriteError(err))?;

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
