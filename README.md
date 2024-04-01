# coffee_map

This repository generates a map like the following:

![Image cannot be loaded](/assets/example.png)

You can find the actual map [here](https://www.google.com/maps/d/edit?mid=1VM-v9bgc7FdRIzAWzl3ksTIkHcxAlqs&usp=drive_link) which can be loaded directly into google maps on your phone.

# Details
This program generates a KML map by executing the following steps:

1. Use Katana to scrape cafes from [](europeancoffeetrip.com).
1. Check the cache to see if a KML placemark already exists.
1. If not, look up these cafes with the text-search based [google places API](https://developers.google.com/maps/documentation/places/web-service/text-search),
1. Deduplicate and batch the results into one or many kml files.

# How to use
1. Install [Nix](https://nixos.org/) with your favourite package manager.
1. Clone the repository.
1. Generate a [google API key](https://developers.google.com/maps/documentation/places/web-service/get-api-key).
1. Load the development anvironment by running: `nix develop`.
1. Start the program by running: `cargo run <YOU_GOOGLE_PLACES_API_KEY>`.



