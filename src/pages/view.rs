use iced::{
    Element, Length, Task,
    widget::{button, column, row, scrollable, text_input},
};

use crate::config::AppConfig;
use crate::utils::{Tweet, build_timeline, download_file, parse_twtxt};

pub struct ViewPage {
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
    pub fn new(config: &AppConfig) -> Self {
        Self {
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
                if let Ok(url) = url::Url::parse(&self.composer.clone()) {
                    if let Some(host) = url.host_str() {
                        println!("Host: {}", host);
                        self.tweets = parse_twtxt(host, data.as_str());
                    } else {
                        self.tweets = parse_twtxt("unknown", data.as_str());
                    }
                } else {
                    self.tweets = parse_twtxt("unknown", data.as_str());
                }

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

    pub fn view(&self) -> Element<'_, Message> {
        let timeline = build_timeline(&self.tweets);

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
