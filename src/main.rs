
mod playlist;

use playlist::Playlist;
use std::env;
use std::io::BufReader;
use rand;
use std::time::{Duration, SystemTime};

fn main() {
    let mut dir = env::home_dir().unwrap();
    dir.push("Music");
    println!("{:?}", dir);

    let mut playlist = Playlist::from_directory(&dir).unwrap().unwrap();
    let mut rng = rand::thread_rng();

    println!("{:?}", playlist);
    let device = rodio::default_output_device().unwrap();

    loop {
        let mut current = playlist.current();
        if current.is_none() {
            playlist.shuffle(&mut rng);
            playlist.first();
            current = playlist.current();
        }
        let current = current.unwrap();
        let file = std::fs::File::open(current).unwrap();
        println!("{:?}", current);
        let now = SystemTime::now();
        let sink = rodio::play_once(&device, BufReader::new(file)).unwrap();
        sink.sleep_until_end();

        let elapsed = now.elapsed().unwrap();
        let min_run = Duration::from_secs(5);
        if elapsed < min_run {
            println!("ERR {:?} elapsed", elapsed);
        }
        else {
            println!("{:?} elapsed", elapsed);
        }

        playlist.next();
    }
}
