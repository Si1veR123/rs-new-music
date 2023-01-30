use reqwest;
use reqwest::header::USER_AGENT;
use serde_json;
use std::fs::{read, write};

pub fn new_artist(artist: &str) {
    if artist.len() == 0 {
        panic!("No artist name provided");
    }

    let (artist_name, artist_id) = get_artist(artist);

    let artists_saved_file = read("artists.json").unwrap_or(vec!['{' as u8, '}' as u8]);

    let mut artists_json = serde_json::from_slice::<serde_json::Value>(artists_saved_file.as_slice()).unwrap();
    let artists_obj = artists_json.as_object_mut().unwrap();
    
    artists_obj.insert(artist_name, serde_json::Value::String(artist_id));

    write("artists.json", artists_json.to_string()).expect("Couldn't write to file");
}

fn get_artist(artist: &str) -> (String, String) {
    let url = format!("http://musicbrainz.org/ws/2/artist/?query={}&fmt=json", artist);

    let res = reqwest::blocking::Client::new()
        .get(&url)
        .header(USER_AGENT, "new-music/0.1.1")
        .send()
        .expect("API request failed");

    let json_text = res.text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_text).expect("API returned invalid data");

    let artists = json["artists"].as_array().unwrap();
    let first_result = artists.first().expect("No artist found").as_object().unwrap();

    let artist_name = first_result["name"].as_str().unwrap();
    let artist_id = first_result["id"].as_str().unwrap();

    (artist_name.to_string(), artist_id.to_string())
}

pub fn remove_artist(artist: &str) {
    if artist.len() == 0 {
        panic!("No artist name provided");
    }

    let artists_saved_file = read("artists.json").unwrap_or(vec!['{' as u8, '}' as u8]);

    let mut artists_json = serde_json::from_slice::<serde_json::Value>(artists_saved_file.as_slice()).unwrap();
    let artists_obj = artists_json.as_object_mut().unwrap();
    
    artists_obj.remove(artist);

    write("artists.json", artists_json.to_string()).expect("Couldn't write to file");
}
