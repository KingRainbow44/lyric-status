use std::error::Error;
use config::Config;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::tungstenite::{connect, Message};
use url::Url;

#[derive(Deserialize)]
struct Settings {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub reload: Option<bool>,
    pub lyrics: Vec<String>,
    pub interval: u32 // In seconds.
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Emoji {
    pub id: String,
    pub name: String
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusUpdateMessage {
    pub cmd: String,

    pub show_game: Option<bool>,
    pub status: Option<String>,
    pub emoji: Option<Emoji>,
    pub expires_time: Option<u32>,
    pub message: Option<String>,
}

static WEBSOCKET_URL: &str = "ws://127.0.0.1:6463/rpc?v=1";

/// Reads the configuration file.
fn read_config() -> Result<Settings, Box<dyn Error>> {
    Ok(
        Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?
            .try_deserialize::<Settings>()?
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse the configuration file.
    let settings = read_config()?;
    let reload = settings.reload;

    // Connect to the websocket.
    let uri = Url::parse(WEBSOCKET_URL)?;
    let (mut ws_stream, _) = connect(uri)?;

    // Send the songs to the websocket on an interval.
    loop {
        let mut duration = Duration::from_secs(
            settings.interval as u64);
        let mut prefix = settings.prefix
            .clone().unwrap_or("".to_string());
        let mut suffix = settings.suffix
            .clone().unwrap_or("".to_string());
        
        if reload.unwrap_or(false) {
            let settings = read_config()?;
            duration = Duration::from_secs(settings.interval as u64);
            prefix = settings.prefix.unwrap_or("".to_string());
            suffix = settings.suffix.unwrap_or("".to_string());
        }
        
        for lyric in &settings.lyrics {
            // Prepare the websocket message.
            let message = StatusUpdateMessage {
                cmd: "status".to_string(),
                show_game: None,
                status: None,
                emoji: None,
                expires_time: None,
                message: Some(format!("{}{}{}", prefix, lyric, suffix)),
            };
            let message = serde_json::to_string(&message)?;

            // Send the message to the websocket.
            ws_stream.send(Message::Text(message.clone()))?;
            println!("Sent message: {}", message);

            sleep(duration).await;
        }
    }
}
