pub mod player_ctl {

    enum QueryKey {
        XesamTitle,
        XesamArtist,
        XesamAlbum,
        MprisArtUrl,
        MprisLength,
        PlayerName,
        Volume,
        Position,
        Status,
        Shuffle,
    }

    enum PlayerAction {
        PlayPause,
        Stop,
        Next,
        Previous,
        Position(u32),
        Volume(f32),
        Shuffle(bool),
    }

    pub enum PlayerStatus {
        Playing,
        Paused,
        Stopped,
    }

    use std::process::Command;

    pub fn get_artist() -> String {
        query_data(QueryKey::XesamArtist)
    }

    pub fn get_title() -> String {
        query_data(QueryKey::XesamTitle)
    }

    pub fn get_image_url() -> String {
        query_data(QueryKey::MprisArtUrl)
    }

    pub fn get_album() -> String {
        query_data(QueryKey::XesamAlbum)
    }

    pub fn get_length() -> u32 {
        let length = query_data(QueryKey::MprisLength);
        length.parse().unwrap_or(0) / 1000000
    }

    pub fn get_volume() -> f32 {
        let volume = query_data(QueryKey::Volume);
        volume.parse().unwrap_or(0.5)
    }

    pub fn get_position() -> u32 {
        let position = query_data(QueryKey::Position);
        position.parse::<f32>().unwrap_or(0.0) as u32
    }

    pub fn get_shuffle() -> bool {
        let shuffle = query_data(QueryKey::Shuffle);
        shuffle.contains("On")
    }

    pub fn get_status() -> PlayerStatus {
        let status = query_data(QueryKey::Status);
        match status.as_str() {
            "Playing" => PlayerStatus::Playing,
            _ => PlayerStatus::Stopped,
        }
    }

    pub fn is_spotify() -> bool {
        let players = query_data(QueryKey::PlayerName);
        for player in players.trim().split(',') {
            if player.contains("spotify") {
                return true;
            }
        }
        false
    }

    pub fn play_pause() {
        player_action(PlayerAction::PlayPause);
    }

    pub fn stop() {
        player_action(PlayerAction::Stop);
    }

    pub fn next() {
        player_action(PlayerAction::Next);
    }

    pub fn previous() {
        player_action(PlayerAction::Previous);
    }

    pub fn position(time: u32) {
        let song_length = get_length();
        if !(0 < time && time < song_length) {
            println!(
                "Position should be between 0 and song length ({}), will not change position.",
                song_length
            )
        }
        player_action(PlayerAction::Position(time));
    }

    pub fn shuffle(should_shuffle: bool) {
        player_action(PlayerAction::Shuffle(should_shuffle));
    }

    pub fn volume(volume: f32) {
        if !(0.0..=1.0).contains(&volume) {
            println!(
                "Volume should be between 0.0 and 1.0, will not change volume.\nGot: {}",
                volume
            );
            return;
        }
        player_action(PlayerAction::Volume(volume));
    }

    fn query_data(key: QueryKey) -> String {
        let mut command = Command::new("playerctl");
        match key {
            QueryKey::XesamArtist => command.args(["metadata", "xesam:artist"]),
            QueryKey::XesamTitle => command.args(["metadata", "xesam:title"]),
            QueryKey::XesamAlbum => command.args(["metadata", "xesam:album"]),
            QueryKey::MprisArtUrl => command.args(["metadata", "mpris:artUrl"]),
            QueryKey::MprisLength => command.args(["metadata", "mpris:length"]),
            QueryKey::PlayerName => command.arg("-l"),
            QueryKey::Volume => command.arg("volume"),
            QueryKey::Position => command.arg("position"),
            QueryKey::Status => command.arg("status"),
            QueryKey::Shuffle => command.arg("shuffle"),
        };
        let result = command.output();
        match result {
            Ok(value) => String::from_utf8(value.stdout)
                .unwrap_or("".to_string())
                .trim()
                .to_string(),
            Err(_) => "".to_string(),
        }
    }

    fn player_action(action: PlayerAction) {
        let mut command = Command::new("playerctl");
        match action {
            PlayerAction::PlayPause => command.arg("play-pause"),
            PlayerAction::Stop => command.arg("stop"),
            PlayerAction::Next => command.arg("next"),
            PlayerAction::Previous => command.arg("previous"),
            PlayerAction::Volume(volume) => command.args(["volume", &volume.to_string()]),
            PlayerAction::Shuffle(should_shuffle) => match should_shuffle {
                true => command.args(["shuffle", "on"]),
                false => command.args(["shuffle", "off"]),
            },
            PlayerAction::Position(position) => command.args(["position", &position.to_string()]),
        };
        if command.spawn().is_err() {
            println!("Could not execute player action");
        }
    }
}

pub mod image {
    use reqwest::get;
    use serde::Deserialize;
    use serde_json;
    use std::env::temp_dir;
    use std::fs::{create_dir, File};
    use std::io::{BufReader, Write};
    use std::path::{Path, PathBuf};
    use tokio::main;

    use super::player_ctl::get_image_url;

    #[derive(Deserialize)]
    struct Config {
        download_location: PathBuf,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                download_location: temp_dir(),
            }
        }
    }

    fn fetch_config() -> Config {
        let user_config = Path::new("~/.config/daspotwidget");
        if !user_config.exists() {
            let _ = create_dir(user_config);
        };
        let mut config_file = PathBuf::from(user_config);
        config_file.push("config.json");
        if !config_file.exists() {
            Config::default()
        } else {
            let file = File::open(config_file).unwrap();
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader).unwrap();
            config
        }
    }

    async fn download_image(url: &str) -> PathBuf {
        let mut file_path = fetch_config().download_location;
        let file_name = url.split('/').last().unwrap();
        file_path.push(file_name);
        if !file_path.exists() {
            let data = get(url).await.unwrap().bytes().await.unwrap();
            let mut file = File::create(&file_path).unwrap();
            let _ = file.write_all(&data);
        }
        file_path
    }

    #[main]
    pub async fn get_image() -> Option<PathBuf> {
        let url = get_image_url();
        if url.starts_with("http") {
            Some(download_image(&url).await)
        } else if url.starts_with("file://") {
            let mut path = PathBuf::new();
            path.push(url.strip_prefix("file://").unwrap());
            Some(path)
        } else if !url.is_empty() {
            println!("Url type not supported: {}", url);
            None
        } else {
            None
        }
    }
}
