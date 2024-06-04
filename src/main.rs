use audiotags::Tag;
use std::{fs::File, path::Path};
use serde::Deserialize;
use std::io::prelude::*;
use std::env;
use walkdir::WalkDir;


#[derive(Deserialize)]
struct lrclib {
    id: i32,
    name: String,
    trackName: String,
    artistName: String,
    albumName: String,
    duration: f64,
    instrumental: bool,
    plainLyrics: Option<String>,
    syncedLyrics: Option<String> // option can either contain type of "None" ()

}

fn description(){
    println!("rustlrc is a simple command line tool that fetches lyrics for tracks in a given directory using the lrclib.net API to fetch lyrics.");
    println!("Usage: rustlrc <file_path>");
    println!("Example: rustlrc /home/user/Music");
}

fn get_track_tags(file_path: &str) -> (String, String, String, String){

    println!("{}",file_path);
    let mut tag = Tag::new().read_from_path(file_path).unwrap();

    let title = match tag.title() {
        Some(t) => t.to_string().replace(" ", "+"),
        None => "".to_string()
    };

    let artist = match tag.artist() {
        Some(a) => a.to_string().replace(" ", "+"),
        None => "".to_string()
    };
    let album = match tag.album_title() {
        Some(a) => a.to_string().replace(" ", "+"),
        None => "".to_string()
    };
    let duration = match tag.duration() {
        Some(d) => d.to_string(),
        None => "".to_string()
    };
    
    return (artist, title, album, duration);
}

fn get_lyrics(artist: String, title: String, album: String, duration: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}", artist, title, album, duration);
    let mut res = reqwest::blocking::get(url)?;

    let mut lyrics = String::new();

    if res.status().is_success() {
        let jsoninfo = res.json::<lrclib>().expect("");
        println!("Found Lyrics for the song: {}", title.replace("+", " "));
        lyrics = match jsoninfo.syncedLyrics {
            Some(erm) => erm,
            None => match jsoninfo.plainLyrics{
                Some(erm) => erm,
                None => "".to_string()
            }
        };
       
    }

    Ok(lyrics)
}

fn collect_tracks(dir: String) -> Vec<String> {
    let mut tracks: Vec<String> = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        

        let f_path = match entry.into_path().to_str() {
            Some(f) => f.to_string(),
            None => "".to_string()
        };

        if f_path.ends_with(".flac"){
            tracks.push(f_path);
        }
    }
    return tracks;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut fail = 0;
    let mut found = 0;
    let mut total_tracks=0;
    match args.len() {
        1 => {
            description();
            return;
        },
        2 => {
            let tracks = collect_tracks(args[1].to_string());
             total_tracks = tracks.len();
         

            for file_path in tracks {
                //sets the output to the track name + .lrc
                let output = file_path.replace(".flac", ".lrc");
                //checks if the file already exists
                if Path::new(&output).exists() {
                    println!("Lyrics for {} already exist.", file_path);
                    continue;
                }
                let tags = get_track_tags(&file_path);
                let lyrics = match get_lyrics(tags.0, tags.1, tags.2, tags.3){
                    Ok(l) => l,
                    Err(e)=>"".to_string() 
                };

                if lyrics == "".to_string() {
                    println!("No Lyrics Found for {}", file_path);
                    fail += 1;
                    continue;
                }
                //write lyrics to file
                let path = Path::new(&output);
                let display = path.display();
                
                let mut file = match File::create(&path) {
                    Err(why) => panic!("couldn't create {}: {}", display, why),
                    Ok(file) => file,
                };

                match file.write_all(lyrics.as_bytes()) {
                    Err(why) => panic!("couldn't write to {}: {}", display, why),
                    Ok(_) => println!("successfully wrote to {}", display),
                }
                found+=1;
            }
        },
        _ => {
            println!("Too many arguments provided.");
            description();
            return;
        } 
    }
    println!("-----------------------");
    println!("Total tracks processed: {}", total_tracks);
    println!("Total tracks with lyrics found: {}", found);
    println!("Total tracks with lyrics missing: {}", fail);
  
}


