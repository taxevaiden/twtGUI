use chrono::{DateTime, Local, Utc};
use data_encoding::BASE32_NOPAD;
use iced::{
    Background, Border, Color, Length, Padding, Pixels, font,
    widget::{
        Column, Image, column, container,
        image::Handle,
        markdown::{self, Highlight},
        rich_text, row, space, span,
    },
};

use regex::Regex;

use bytes::Bytes;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

use directories::ProjectDirs;

fn cache_root() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("com", "taxevaiden", "twtGUI")
        .ok_or("Could not determine project directories")?;

    let dir = proj.cache_dir();

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    Ok(dir.to_path_buf())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub hash: String,
    pub reply_to: Option<String>, // reply is a tweet hash, defined by something like (#abc1234) at the beginning of the tweet
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub author: String,
    pub content: String,

    #[serde(skip, default = "default_avatar")]
    pub avatar: Handle,

    #[serde(skip)]
    pub md_items: Vec<markdown::Item>,
}

#[derive(Debug, Clone)]
pub struct TweetNode {
    pub index: usize,
    pub children: Vec<TweetNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Metadata {
    pub urls: Vec<String>, // the url(s) of the feed. if the url(s) do not match the url that the user entered in the ViewPage, we should warn the user. we don't redirect them for security reasons
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>, // `type` field, could be "bot" or "rss" (if empty, we assume it's a human managed account)
    pub follows: Vec<Link>,
    pub following: Option<u64>, // number of people they follow; this isn't really needed since we can just do follows.len()
    pub links: Vec<Link>, // urls on their profile (ex. My Github Page: https://github.com/username)
    pub prev: Vec<Link>,
    pub refresh: Option<u64>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            nick: None,
            avatar: None,
            description: None,
            kind: None,
            follows: Vec::new(),
            following: None,
            links: Vec::new(),
            prev: Vec::new(),
            refresh: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedBundle {
    pub tweets: Vec<Tweet>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParsedCache {
    content_hash: String,
    bundle: FeedBundle,
}

fn default_avatar() -> Handle {
    Handle::from_path("assets/default_avatar.png")
}

fn hash_sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn build_threads(tweets: &[Tweet]) -> Vec<TweetNode> {
    let mut children_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();

    let all_hashes: std::collections::HashSet<&str> =
        tweets.iter().map(|t| t.hash.as_str()).collect();

    for (index, tweet) in tweets.iter().enumerate() {
        let is_reply = tweet
            .reply_to
            .as_ref()
            .map(|parent| all_hashes.contains(parent.as_str()))
            .unwrap_or(false);

        if !is_reply {
            roots.push(index);
        } else {
            let parent_hash = tweet.reply_to.as_ref().unwrap();
            children_map
                .entry(parent_hash.clone())
                .or_default()
                .push(index);
        }
    }

    roots
        .into_iter()
        .map(|root_index| construct_node(root_index, tweets, &children_map))
        .collect()
}

fn construct_node(
    index: usize,
    tweets: &[Tweet],
    children_map: &HashMap<String, Vec<usize>>,
) -> TweetNode {
    let children = children_map
        .get(&tweets[index].hash)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|child_index| construct_node(child_index, tweets, children_map))
        .collect();

    TweetNode { index, children }
}

// timestamp must be formatted as RFC3339, with the time truncated/expanded to seconds precision
// it also has to be formatted using the Zulu indicator (Z)

pub fn compute_twt_hash(feed_url: &str, timestamp: &str, text: &str) -> String {
    use blake2::{
        Blake2bVar,
        digest::{Update, VariableOutput},
    };

    let payload = format!("{feed_url}\n{timestamp}\n{text}");

    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(payload.as_bytes());
    let mut result = vec![0u8; 32];
    hasher.finalize_variable(&mut result).unwrap();

    let encoded = BASE32_NOPAD.encode(&result).to_lowercase();

    encoded
        .chars()
        .rev()
        .take(7)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

pub fn parse_twt_contents(raw_content: &str) -> (Option<String>, String) {
    let subject_re = Regex::new(r"^\(#(?P<hash>[a-z0-9]{7})\)\s*").unwrap();
    let mention_re = Regex::new(r"@<(?P<nick>[^\s>]+)(?:\s+(?P<url>[^>]+))?>").unwrap();

    let mut reply_to = None;
    let mut content = raw_content;

    if let Some(cap) = subject_re.captures(content) {
        reply_to = Some(cap["hash"].to_string());
        let end = cap.get(0).unwrap().end();
        content = &content[end..];
    }

    let mut last_end = 0;
    let mut markdown_content = String::new();

    for cap in mention_re.captures_iter(content) {
        let m = cap.get(0).unwrap();

        markdown_content.push_str(&content[last_end..m.start()]);

        let nick = cap.name("nick").unwrap().as_str();
        let url = cap.name("url").map(|m| m.as_str()).unwrap_or(nick);

        markdown_content.push_str(&format!("[@{}]({})", nick, url));

        last_end = m.end();
    }

    markdown_content.push_str(&content[last_end..]);

    let final_markdown = markdown_content
        .replace("\\u2028", "  \n")
        .trim()
        .to_string();

    (reply_to, final_markdown)
}

pub fn parse_metadata(input: &str) -> Option<Metadata> {
    let mut metadata = Metadata::default();

    for line in input.lines() {
        let Some(stripped) = line.strip_prefix('#') else {
            continue;
        };
        let Some((key, value)) = stripped.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim();

        match key {
            "url" => metadata.urls.push(value.to_string()),

            "nick" => metadata.nick = Some(value.to_string()),

            "avatar" => metadata.avatar = Some(value.to_string()),

            "description" => metadata.description = Some(value.to_string()),

            "type" => metadata.kind = Some(value.to_string()),

            "follow" => {
                // format: nick url
                if let Some((nick, url)) = value.rsplit_once(' ') {
                    metadata.follows.push(Link {
                        text: nick.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

            "following" => {
                if let Ok(num) = value.parse::<u64>() {
                    metadata.following = Some(num);
                }
            }

            "link" => {
                // format: text url
                if let Some((text, url)) = value.rsplit_once(' ') {
                    metadata.links.push(Link {
                        text: text.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

            "prev" => {
                // format: text url
                if let Some((text, url)) = value.rsplit_once(' ') {
                    metadata.prev.push(Link {
                        text: text.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

            "refresh" => {
                if let Ok(num) = value.parse::<u64>() {
                    metadata.refresh = Some(num);
                }
            }

            _ => {}
        }
    }

    // Return None if nothing was actually parsed
    if metadata == Metadata::default() {
        None
    } else {
        Some(metadata)
    }
}

pub fn parse_tweets(author: &str, url: &str, avatar: Option<Handle>, input: &str) -> Vec<Tweet> {
    let author_name = author.to_string();

    input
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (timestamp_str, raw_content) = line.split_once('\t')?;
            let (reply_to, display_content) = parse_twt_contents(raw_content);
            let items = markdown::parse(&display_content).collect();

            Some(Tweet {
                hash: compute_twt_hash(url, timestamp_str, raw_content),
                reply_to,
                timestamp: DateTime::parse_from_rfc3339(timestamp_str)
                    .ok()?
                    .with_timezone(&Utc),
                author: author_name.clone(),
                url: url.to_string(),
                avatar: avatar
                    .clone()
                    .unwrap_or_else(|| Handle::from_path("assets/default_avatar.png")),
                content: display_content,
                md_items: items,
            })
        })
        .collect()
}

pub fn build_threaded_feed<'a, M, F>(
    threads: &'a [TweetNode],
    tweets: &'a [Tweet],
    on_link: F,
) -> Column<'a, M>
where
    M: 'a,
    F: Fn(String) -> M + Copy + 'a,
{
    let mut col = column!().spacing(24);

    for thread in threads {
        col = col.push(render_tweet_node(thread, tweets, on_link, 0));
    }

    col
}

fn render_tweet_node<'a, M, F>(
    node: &TweetNode,
    tweets: &'a [Tweet],
    on_link: F,
    depth: usize,
) -> Column<'a, M>
where
    M: 'a,
    F: Fn(markdown::Uri) -> M + Copy + 'a,
{
    let reg = font::Font::with_name("Iosevka Aile");
    let mut bold = font::Font::with_name("Iosevka Aile");
    bold.weight = font::Weight::Bold;

    let tweet = &tweets[node.index];
    let bg = iced::Theme::CatppuccinMocha.palette().background;
    let code_bg = Color::from_rgb(bg.r * 0.75, bg.g * 0.75, bg.b * 0.75);

    let content = markdown::view(
        &tweet.md_items,
        markdown::Settings::with_text_size(
            Pixels(12.0),
            markdown::Style {
                font: reg,
                link_color: Color::from_rgb(0.4, 0.6, 1.0),
                inline_code_font: reg,
                inline_code_color: Color::from_rgb(0.85, 0.85, 0.85),
                inline_code_highlight: Highlight {
                    background: Background::Color(code_bg),
                    border: Border::default(),
                },
                inline_code_padding: Padding::from(2.0),
                code_block_font: reg,
            },
        ),
    )
    .map(on_link);

    let avatar_img = Image::new(tweet.avatar.clone())
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(40.0))
        .border_radius(20);

    let formatted_time = tweet
        .timestamp
        .with_timezone(&Local)
        .format("%h %-d %Y %-I:%M %p");

    let header = rich_text![
        span(&tweet.author).font(bold).link(tweet.url.clone()),
        span(" - "),
        span(formatted_time.to_string()),
        span(" "),
        span(tweet.hash.clone())
    ]
    .on_link_click(on_link);

    let tweet_view = container(
        row![
            avatar_img,
            column![header, container(content)]
                .padding([4.0, 0.0])
                .spacing(4)
        ]
        .spacing(8),
    )
    .padding(4);

    let mut thread_column = column![tweet_view].spacing(8);

    let mut sorted_children = node.children.clone();
    sorted_children.sort_by_key(|child| tweets[child.index].timestamp);

    for reply in &sorted_children {
        let indented_reply = row![
            space().width(20),
            render_tweet_node(reply, tweets, on_link, depth + 1)
        ];
        thread_column = thread_column.push(indented_reply);
    }

    thread_column
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    content: String,
    metadata: CacheMetadata,
}

// Used for download_file
fn get_txt_cache_path(url: &str) -> Result<PathBuf, String> {
    let hash = hash_sha256_str(url);
    let mut path = cache_root()?;
    path.push(format!("{hash}.json"));
    Ok(path)
}

// Used for download_binary
fn get_bin_cache_paths(url: &str) -> Result<(PathBuf, PathBuf), String> {
    let hash = hash_sha256_str(url);
    let dir = cache_root()?;

    let mut data_path = dir.clone();
    data_path.push(format!("{hash}.bin"));

    let mut meta_path = dir;
    meta_path.push(format!("{hash}.meta"));

    Ok((data_path, meta_path))
}

// Used for download_and_parse_twtxt
fn get_parsed_cache_path(url: &str) -> Result<PathBuf, String> {
    let hash = hash_sha256_str(url);
    let mut path = cache_root()?;
    path.push(format!("{hash}.parsed.json"));
    Ok(path)
}

pub async fn download_binary(url: String) -> Result<Bytes, String> {
    println!("Downloading file from {}", url);
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let (data_path, meta_path) = get_bin_cache_paths(&url)?;

    let metadata: Option<CacheMetadata> = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let mut request = client.get(&url);
    if let Some(ref meta) = metadata {
        if let Some(ref etag) = meta.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = meta.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_mod);
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        println!(
            "304 Not Modified: {}\n\tData: {}\n\tMetadata: {}",
            url,
            data_path.display(),
            meta_path.display()
        );
        let raw_bytes = std::fs::read(&data_path).map_err(|e| e.to_string())?;
        return Ok(Bytes::from(raw_bytes));
    }

    // 200 OK
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let data = response.bytes().await.map_err(|e| e.to_string())?;

    let meta_json = serde_json::to_string(&CacheMetadata {
        etag,
        last_modified,
    })
    .map_err(|e| e.to_string())?;

    println!(
        "200 OK: {}\n\tWriting {} bytes to data {}\n\tWriting {} bytes to metadata {}",
        url,
        data.len(),
        data_path.display(),
        meta_json.len(),
        meta_path.display()
    );

    std::fs::write(&data_path, &data).map_err(|e| e.to_string())?;
    std::fs::write(&meta_path, meta_json).map_err(|e| e.to_string())?;

    Ok(data)
}

pub async fn download_twtxt(url: String) -> Result<String, String> {
    println!("Downloading twtxt.txt from {}", url);
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let cache_path = get_txt_cache_path(&url)?;

    let cached_data: Option<CacheEntry> = std::fs::read_to_string(&cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());

    let mut request = client.get(&url);
    if let Some(ref entry) = cached_data {
        if let Some(ref etag) = entry.metadata.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = entry.metadata.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_mod);
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        println!("304 Not Modified: {}\n\t{}", url, cache_path.display());
        return cached_data
            .map(|e| e.content)
            .ok_or_else(|| "Server returned 304 but no local file found".to_string());
    }

    // 200 OK
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let content = response.text().await.map_err(|e| e.to_string())?;

    let new_entry = CacheEntry {
        content: content.clone(),
        metadata: CacheMetadata {
            etag,
            last_modified,
        },
    };

    let serialized = serde_json::to_string(&new_entry).map_err(|e| e.to_string())?;
    println!(
        "200 OK: {}\n\tWriting {} bytes to {}",
        url,
        serialized.len(),
        cache_path.display()
    );
    std::fs::write(cache_path, serialized).map_err(|e| e.to_string())?;

    Ok(content)
}

// use_nick determines whether the nick provided should be used as the actual display name, or just a fallback if there is no nick in the metadata
// this nick should NOT be in the cache, only the nick provided by the feed's metadata
pub async fn download_and_parse_twtxt(
    nick: String,
    url: String,
    use_nick: bool,
) -> Result<FeedBundle, String> {
    let raw = download_twtxt(url.clone()).await?;
    let raw_hash = hash_sha256_str(&raw);
    let parsed_path = get_parsed_cache_path(&url)?;

    if let Ok(cached_str) = std::fs::read_to_string(&parsed_path) {
        if let Ok(cache) = serde_json::from_str::<ParsedCache>(&cached_str) {
            if cache.content_hash == raw_hash {
                let mut bundle = cache.bundle;
                for tweet in &mut bundle.tweets {
                    tweet.md_items = markdown::parse(&tweet.content).collect();
                }
                return Ok(apply_nick_override(bundle, &nick, use_nick));
            }
        }
    }

    let metadata = parse_metadata(&raw);

    let canonical_nick = metadata
        .as_ref()
        .and_then(|m| m.nick.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            url::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_string))
                .unwrap_or_else(|| nick.clone())
        });

    let tweets = parse_tweets(&canonical_nick, &url, None, &raw);

    let canonical_bundle = FeedBundle { tweets, metadata };

    let cache = ParsedCache {
        content_hash: raw_hash,
        bundle: canonical_bundle.clone(),
    };

    let _ = std::fs::write(parsed_path, serde_json::to_string(&cache).unwrap());

    Ok(apply_nick_override(canonical_bundle, &nick, use_nick))
}

fn apply_nick_override(mut bundle: FeedBundle, nick: &str, use_nick: bool) -> FeedBundle {
    if use_nick {
        for tweet in &mut bundle.tweets {
            tweet.author = nick.to_string();
        }
    }

    bundle
}
