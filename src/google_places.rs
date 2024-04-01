use std::collections::HashMap;

use kml::types::{Element, Geometry, Placemark, Point};
use reqwest::blocking;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::model::PipelineError;
use crate::write_kml::CUP_STYLE_ID;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    pub text: String,
    pub language_code: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GooglePlace {
    pub id: String,
    pub display_name: DisplayName,
    pub formatted_address: String,
    pub google_maps_uri: String,
    pub location: Location,
    pub types: Vec<String>,
}

pub struct GooglePlaceResult {
    pub place: GooglePlace,
    pub searchterm: String,
}

impl GooglePlaceResult {
    pub fn to_placemark(self) -> Placemark {
        let searchterm = self.searchterm;
        let place = self.place;

        let mut attrs = HashMap::<String, String>::new();
        attrs.insert(String::from("search_term"), searchterm);
        attrs.insert("id".to_string(), place.id);

        let geometry = Geometry::Point(Point::new(
            place.location.longitude,
            place.location.latitude,
            Some(0.0),
        ));

        Placemark {
            name: Some(place.display_name.text),
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
                place.google_maps_uri, place.formatted_address
            )),
            geometry: Some(geometry),
        }
    }
}

pub fn query(
    client: &blocking::Client,
    searchterm: String,
    api_key: String,
) -> Result<GooglePlaceResult, PipelineError> {
    let endpoint = "https://places.googleapis.com/v1/places:searchText";
    let request_body = json!({
        "textQuery": searchterm.clone()
    });

    let req = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", api_key.as_str())
        .header(
            "X-Goog-FieldMask",
            "places.displayName,places.id,places.formattedAddress,places.location,places.googleMapsUri,places.types",
        )
        .body(request_body.to_string());

    let response = req
        .send()
        .map_err(|err| PipelineError::GoogleHTTPError(format!("{:#?}", err)))?;

    if response.status() != reqwest::StatusCode::OK {
        let error_str = format!(
            "search_term: {}, response code: {:#?}",
            &searchterm,
            response.status()
        );
        return Err(PipelineError::GoogleHTTPError(error_str));
    }

    let body = response
        .text()
        .map_err(|err| PipelineError::GoogleHTTPError(format!("{:#?}", err)))?;

    let response: Value = serde_json::from_str(body.as_str())
        .map_err(|err| PipelineError::GoogleHTTPError(format!("{:#?}", err)))?;

    let places_json = response
        .get("places")
        .ok_or(PipelineError::GooglePlaceNotFoundError(searchterm.clone()))?;

    let mut places = serde_json::from_value::<Vec<GooglePlace>>(places_json.clone())
        .map_err(|err| PipelineError::GoogleJsonParseError(format!("{:#?}", err)))?
        .into_iter();

    let cafe_keyword = "cafe".to_string();
    let coffee_shop_keyword = "coffee_shop".to_string();

    let place = if let Some(coffee_shop) = places
        .clone()
        .filter(|place| {
            place.types.contains(&cafe_keyword) | place.types.contains(&coffee_shop_keyword)
        })
        .next()
    {
        coffee_shop
    } else {
        places
            .next()
            .ok_or(PipelineError::GooglePlaceNotFoundError(searchterm.clone()))?
    };

    Ok(GooglePlaceResult { place, searchterm })
}
