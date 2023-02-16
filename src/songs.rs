use colored::Colorize;
use reqwest;
use reqwest::header::USER_AGENT;
use serde_json;
use std::fmt::Debug;
use std::fs::read;
use std::path::PathBuf;
use chrono::NaiveDate;
use chrono::Days;
use chrono::offset::Local;
use std::thread;
use std::time::Duration;
use home::home_dir;

const SAVED_DATA_FILENAME: &'static str = "AppData\\Local\\artists.json";

struct Track {
    title: String,
    artist: String,
    released: NaiveDate,
}

impl Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0: <30} {1: <30} {2: <30}", self.title.bright_green(), self.artist.bright_blue(), self.released.to_string().red())
    }
}

fn file_path() -> PathBuf {
    home_dir().unwrap().join(SAVED_DATA_FILENAME)
}

pub fn recent_songs(days: u64) {
    let artists_saved_file = read(file_path()).unwrap_or(vec!['{' as u8, '}' as u8]);
    let artists_json = serde_json::from_slice::<serde_json::Value>(artists_saved_file.as_slice()).unwrap();

    let artists_obj = artists_json.as_object().unwrap();

    let minimum_date = Local::now().date_naive()
        .checked_sub_days(
            Days::new(days)
        ).unwrap();

    let mut found_songs = false;
    for (n,artist_name) in artists_obj.keys().enumerate() {
        let artist_id = artists_obj.get(artist_name).unwrap().as_str().unwrap();
        let tracks = tracks_by_artist(artist_id, artist_name);
        let recent_tracks = tracks.iter().filter(|&track|
            track.released >= minimum_date
        );

        recent_tracks.for_each(|track| {
            println!("{:?}", track);
            found_songs = true;
        });

        println!("Searched {}/{} artists", n+1, artists_obj.len());

        thread::sleep(Duration::from_millis(900));
    }

    if !found_songs {
        println!("No recent songs found!");
    }
}

fn tracks_by_artist(artist_id: &str, artist_name: &str) -> Vec<Track> {
    let mut res: reqwest::blocking::Response;

    // for pagination
    let mut offset_i = 0;
    let mut all_releases_json: Vec<serde_json::Value> = vec![];

    'req: loop {
        let url = format!("http://musicbrainz.org/ws/2/release?artist={}&fmt=json&limit=100&inc=recordings&offset={}", artist_id, offset_i);

        res = reqwest::blocking::Client::new()
            .get(&url)
            .header(USER_AGENT, "new-music/0.1.1")
            .send()
            .expect("API request failed");

        if res.status() == 503 {
            println!("Possible rate limiting. Temporarily pausing...");
            thread::sleep(Duration::from_millis(10000));
        } else {
            let json_text = res.text().unwrap();
            let json: serde_json::Value = serde_json::from_str(&json_text).expect("API returned invalid data");
            let release_count = json.as_object().unwrap()["release-count"].as_u64().unwrap();
            offset_i += 100;

            all_releases_json.push(json);

            if offset_i >= release_count {
                break 'req;
            }
        }
    }
    let empty_vec = vec![];
    let all_tracks: Vec<Track> = all_releases_json.iter().map(|json| {
        json["releases"].as_array().unwrap().iter().map(
            |release|
                release["media"][0]["tracks"].as_array().unwrap_or(&empty_vec).iter().map(
                    |track| {
                        let track_data = track.as_object().unwrap();
    
                        let date_str = track_data["recording"]["first-release-date"].as_str().unwrap_or("2000-01-01");
                        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or(NaiveDate::default());
    
                        Track {
                            title: track_data["title"].as_str().unwrap().replace("\"", ""),
                            artist: artist_name.to_string(),
                            released: date
                        }
                    }
                )
        ).flatten()
    }).flatten().collect();
    

    all_tracks
}
