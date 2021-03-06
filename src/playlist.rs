

use std::path::{Path, PathBuf};
use rand::Rng;
use rand::seq::SliceRandom;
use std::io;
use std::fs;


#[derive(Debug)]
pub struct Playlist {
    // playlist songs as originally defined
    songs: Vec<PathBuf>,
    // current order to play files in (index of songs)
    order: Vec<usize>,
    // current song to play (index of songs)
    current_song: Option<usize>,
    // current position in playlist order (index of order)
    current_order: Option<usize>,
}

impl Playlist {
    pub fn new(songs: Vec<PathBuf>) -> Option<Playlist> {
        if songs.is_empty() {
            return None;
        }
        let order = (0..songs.len()).collect();

        Some(Playlist{
            songs,
            order,
            current_song: None,
            current_order: None
        })
    }

    pub fn from_line_delimited_string(files_str: &str) -> Option<Playlist> {
        if files_str.is_empty() {
            return None;
        }
        let songs = files_str.lines().filter(|&p| !p.is_empty()).map(|s| PathBuf::from(s)).collect();
        Playlist::new(songs)
    }

    #[allow(dead_code)]
    pub fn from_line_delimited_file(playlist: &Path) -> io::Result<Option<Playlist>> {
        let files_str = std::fs::read_to_string(playlist)?;
        Ok(Playlist::from_line_delimited_string(&files_str))
    }

    #[allow(dead_code)]
    pub fn from_directory(directory: &Path) -> io::Result<Option<Playlist>> {
        let mut files = Vec::<PathBuf>::new();
        Playlist::scan_directory(directory, &mut files)?;
        files.sort();
        Ok(Playlist::new(files))
    }

    fn scan_directory(directory: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
        if directory.is_dir() {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    Playlist::scan_directory(&path, found)?;
                }
                else if let Some(ext) = path.extension()
                    .map(|osstr| osstr.to_string_lossy())
                    .map(|s| s.to_lowercase())
                {
                    let ext = &ext;

                    if (cfg!(feature = "mp3") && ext == "mp3")
                    || (cfg!(feature = "vorbis") && ext == "ogg")
                    || (cfg!(feature = "flac") && ext == "flac")
                    || (cfg!(feature = "wav") && ext == "wav")
                    {
                        found.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Option<&Path> {
        self.current_song.map(|cs| self.songs[cs].as_path())
    }

    #[allow(dead_code)]
    pub fn prev(&mut self) {
        if let Some(ord) = self.current_order {
            if ord == 0 {
                self.current_order = None;
                self.current_song = None;
            }
            else {
                self.current_order = Some(ord - 1);
                self.current_song = Some(self.order[ord - 1]);
            }
        }
    }

    #[allow(dead_code)]
    pub fn next(&mut self) {
        if let Some(ord) = self.current_order {
            if ord + 1 >= self.songs.len() {
                self.current_order = None;
                self.current_song = None;
            }
            else {
                self.current_order = Some(ord + 1);
                self.current_song = Some(self.order[ord + 1]);
            }
        }
    }

    #[allow(dead_code)]
    pub fn first(&mut self) {
        self.current_order = Some(0);
        self.current_song = Some(self.order[0]);
    }

    #[allow(dead_code)]
    pub fn last(&mut self) {
        let end = self.songs.len() - 1;
        self.current_order = Some(end);
        self.current_song = Some(self.order[end]);
    }

    #[allow(dead_code)]
    pub fn playlist_order(&mut self) {
        self.order = (0..self.songs.len()).collect();
        self.current_order = self.current_song;
    }

    #[allow(dead_code)]
    pub fn shuffle<R>(&mut self, rng: &mut R)
        where R: Rng + ?Sized
    {
        self.order.shuffle(rng);
        if let Some(song) = self.current_song {
            // order has been scrambled; find the index for the current song (it is unique and guaranteed to exist)
            self.current_order = Some(self.order.iter().position(|&o| o == song).expect("Unable to find current song in shuffled order"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_load() {
        let playlist = Playlist::from_line_delimited_string("first\nsecond\nthird\nfourth\nfifth");
        assert!(playlist.is_some());
        let playlist = playlist.unwrap();
        let songs: Vec<&str> = playlist.songs.iter().map(|p| p.to_str().unwrap()).collect();
        assert_eq!(songs, vec!["first", "second", "third", "fourth", "fifth"])
    }

    #[test]
    fn test_next() {
        let playlist = Playlist::from_line_delimited_string("first\nsecond\nthird\nfourth\nfifth");
        assert!(playlist.is_some());
        let mut playlist = playlist.unwrap();
        assert_eq!(playlist.current(), None);
        playlist.first();
        assert_eq!(playlist.current(), Some(Path::new("first")));
        playlist.next();
        assert_eq!(playlist.current(), Some(Path::new("second")));
        playlist.next();
        assert_eq!(playlist.current(), Some(Path::new("third")));
        playlist.next();
        assert_eq!(playlist.current(), Some(Path::new("fourth")));
        playlist.next();
        assert_eq!(playlist.current(), Some(Path::new("fifth")));
        playlist.next();
        assert_eq!(playlist.current(), None);
    }

    #[test]
    fn test_prev() {
        let playlist = Playlist::from_line_delimited_string("first\nsecond\nthird\nfourth\nfifth");
        assert!(playlist.is_some());
        let mut playlist = playlist.unwrap();
        assert_eq!(playlist.current(), None);
        playlist.last();
        assert_eq!(playlist.current(), Some(Path::new("fifth")));
        playlist.prev();
        assert_eq!(playlist.current(), Some(Path::new("fourth")));
        playlist.prev();
        assert_eq!(playlist.current(), Some(Path::new("third")));
        playlist.prev();
        assert_eq!(playlist.current(), Some(Path::new("second")));
        playlist.prev();
        assert_eq!(playlist.current(), Some(Path::new("first")));
        playlist.prev();
        assert_eq!(playlist.current(), None);
    }

    #[test]
    fn test_shuffle() {
        let playlist = Playlist::from_line_delimited_string("first\nsecond\nthird\nfourth\nfifth");
        assert!(playlist.is_some());
        let mut playlist = playlist.unwrap();

        let mut rng: StdRng = SeedableRng::seed_from_u64(123456789);
        playlist.first();
        playlist.next();
        playlist.shuffle(&mut rng);

        let songs: Vec<&str> = playlist.order.iter().map(|&o| playlist.songs[o].to_str().unwrap()).collect();
        assert_eq!(songs, vec!["first", "fifth", "third", "fourth", "second"]);
        assert_eq!(playlist.current(), Some(Path::new("second")));
        playlist.next();
        assert_eq!(playlist.current(), None); // End of playlist, restart from start
    }

    #[test]
    fn test_playlist_order() {
        let playlist = Playlist::from_line_delimited_string("first\nsecond\nthird\nfourth\nfifth");
        assert!(playlist.is_some());
        let mut playlist = playlist.unwrap();

        let mut rng: StdRng = SeedableRng::seed_from_u64(123456789);
        playlist.first();
        playlist.shuffle(&mut rng); // new order: "first", "fifth", "third", "fourth", "second"
        playlist.next(); // Move to "fifth"
        playlist.playlist_order();

        let songs: Vec<&str> = playlist.order.iter().map(|&o| playlist.songs[o].to_str().unwrap()).collect();
        assert_eq!(songs, vec!["first", "second", "third", "fourth", "fifth"]);
        assert_eq!(playlist.current_order, Some(4));
        assert_eq!(playlist.current_song, Some(4));
        assert_eq!(playlist.current(), Some(Path::new("fifth"))); // Still on "fifth"

        playlist.next();
        assert_eq!(playlist.current(), None); // End of playlist, restart from start
    }

    #[test]
    fn test_directory_loading() {
        let mut cd =  std::env::current_dir().unwrap();
        cd.push("test");
        cd.push("scan");

        let testcases = [
            #[cfg(feature = "mp3")] "1.mp3",
            #[cfg(feature = "wav")] "2.wav",
            #[cfg(feature = "flac")] "3.flac",
            #[cfg(feature = "vorbis")] "4.ogg",
            #[cfg(feature = "mp3")] "sub1/1.MP3",
            #[cfg(feature = "wav")] "sub1/2.WAV",
            #[cfg(feature = "flac")] "sub1/3.FLAC",
            #[cfg(feature = "vorbis")] "sub1/4.OGG",
            #[cfg(feature = "mp3")] "sub2/1.mp3",
            #[cfg(feature = "wav")] "sub2/2.wav",
            #[cfg(feature = "flac")] "sub2/3.flac",
            #[cfg(feature = "vorbis")] "sub2/4.ogg",
        ];
        let mut expected: Vec<PathBuf> = testcases.iter().map(|&p| [&cd, &PathBuf::from(p)].iter().collect()).collect();
        expected.sort();

        let playlist = Playlist::from_directory(&cd);
        assert!(playlist.is_ok());
        let playlist = playlist.unwrap();
        if expected.is_empty() {
            assert!(playlist.is_none());
            return;
        }

        assert!(playlist.is_some());
        assert_eq!(playlist.unwrap().songs, expected);
    }
}