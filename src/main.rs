
mod playlist;

use playlist::Playlist;
use std::env;
use std::io::BufReader;
use rand;

fn main() {
    let mut dir = env::home_dir().unwrap();
    dir.push("Music");
    println!("{:?}", dir);

    let mut playlist = Playlist::from_directory(&dir).unwrap().unwrap();
    let mut rng = rand::thread_rng();
    playlist.shuffle(&mut rng);
    playlist.first();
    println!("{:?}", playlist);

    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);

    loop {
        let file = std::fs::File::open(playlist.current()).unwrap();
        println!("{:?}", playlist.current());
        sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
        playlist.next();
        sink.sleep_until_end();
        sink.stop();
    }
}
