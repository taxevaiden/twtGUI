use chrono::Local;
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, text, text_input},
};

use crate::config::AppConfig;
use crate::utils::{Tweet, parse_twtxt};

pub struct ViewPage {
    config: AppConfig,
    composer: String,
    success: bool,
    fetched: String,
    tweets: Vec<Tweet>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    ViewPressed,
    DownloadFinished(Result<String, String>),
}

impl ViewPage {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: config.clone(),
            composer: config.settings.twturl.clone(),
            success: false,
            fetched: String::new(),
            tweets: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
                Task::none()
            }

            Message::ViewPressed => {
                self.tweets.clear();
                self.fetched = String::new();
                self.success = false;
                println!("Downloading...");
                Task::perform(
                    download_file(self.composer.clone()),
                    Message::DownloadFinished,
                )
            }

            Message::DownloadFinished(Ok(data)) => {
                self.success = true;
                self.fetched = data.clone();
                println!("{}", data);
                self.tweets = parse_twtxt(self.composer.clone().as_str(), data.as_str());
                self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                Task::none()
            }
            Message::DownloadFinished(Err(e)) => {
                self.success = false;
                self.fetched = format!("Error: {}", e);
                println!("{}", e);
                Task::none()
            }
        }
    }

    // fn refresh_timeline(&mut self) {
    //     self.tweets.clear();

    //     let path = Path::new(&self.config.settings.twtxt);
    //     // let file = File::open(&path).unwrap();
    //     // let reader = io::BufReader::new(file);

    //     // for line in reader.lines().flatten() {
    //     //     let parts: Vec<&str> = line.splitn(2, '\t').collect();
    //     //     if parts.len() == 2 {
    //     //         if let Ok(ts) = DateTime::parse_from_rfc3339(parts[0]) {
    //     //             self.tweets.push(Tweet {
    //     //                 timestamp: ts.with_timezone(&Utc),
    //     //                 author: self.config.settings.nick.clone(),
    //     //                 content: parts[1].to_string(),
    //     //             });
    //     //         }
    //     //     }
    //     // }
    //     //
    //     if let Ok(content) = std::fs::read_to_string(path) {
    //         self.tweets =
    //             parse_twtxt(self.config.settings.nick.clone().as_str(), content.as_str()).clone();

    //         self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    //     }
    // }

    pub fn view(&self) -> Element<'_, Message> {
        let timeline = self.tweets.iter().fold(column!().spacing(2), |col, tweet| {
            let formatted = format!(
                "{} {}: {}",
                tweet
                    .timestamp
                    .with_timezone(&Local)
                    .format("%m/%d/%Y %-I:%M %p"),
                tweet.author,
                tweet.content
            );

            col.push(container(text(formatted)).padding(4).width(Length::Fill))
        });
        //

        let scroll = scrollable(timeline).height(iced::Length::Fill);

        let composer = row![
            text_input("https://example.com/twtxt.txt", &self.composer)
                .on_input(Message::ComposerChanged)
                .padding(8),
            button("View")
                .on_press(Message::ViewPressed)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![composer, scroll]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8)
            .into()
    }
}

async fn download_file(url: String) -> Result<String, String> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}
