<p align="center">
  <img width="256" alt="The twtGUI logo" src="./assets/icon.svg">
</p>

# twtGUI

A graphical client for twtxt

## What's supported?

- Sending tweets
- Fetching (and caching) feeds
- Following other feeds
- **Some** of the [twtxt v2 specification](https://twtxt.dev)
  - Mentions
  - [Twt Hash Extension](https://twtxt.dev/exts/twt-hash.html)
  - [Twt Subject Extension](https://twtxt.dev/exts/twt-subject.html) (You can only see if a tweet is a reply for now)
  - [Metadata Extension](https://twtxt.dev/exts/metadata.html)

If you're someone whose `twtxt.txt` only follows the twtxt v1 specification, this client's great for you!

If you're someone whose `twtxt.txt` follows the twtxt v2 specification, expect some features to be missing.

## Configuration

twtGUI uses a `config.toml` file to store user settings and follow information.  
The file is automatically created on first launch if it does not already exist.

There is currently no dedicated settings page in twtGUI, so you must edit `config.toml` manually to change settings such as your nickname or the path to your `twtxt.txt` file.

Feeds you follow can be managed through the **Following** page inside twtGUI.

If you need to edit `config.toml`, you can find it in your system’s configuration directory:

| Platform | File path |
|----------|-----------|
| Windows  | `C:\Users\yourname\AppData\Roaming\taxevaiden\twtGUI\config\config.toml` |
| macOS    | `/Users/yourname/Library/Application Support/com.taxevaiden.twtGUI/config.toml` |
| Linux    | `/home/yourname/.config/twtgui/config.toml` |

The configuration file is divided into two main sections:

- `[metadata]`
- `[paths]`

---

### `[metadata]`

Contains information about you and your twtxt identity, along with optional metadata defined by the twtxt Metadata Extension.

These values are stored locally in `config.toml`, and will be automatically written to your `twtxt.txt` file when saved.

- `urls`  
  A list of public URLs pointing to your `twtxt.txt` file.  
  Typically this contains a single URL.

- `nick`  
  Your display name.

- `avatar`  
  A URL pointing to an image, used as your profile picture.

- `description`  
  A short bio or description of your feed.

- `kind`  
  The type of feed.  
  Common values include:
  - `"bot"` — automated account  
  - `"rss"` — RSS-to-twtxt feed  
  If not set, the feed is assumed to be human-managed.

- `follows`  
  A list of feeds you follow (managed inside twtGUI). Each entry contains:
  - `text` — The display name of the feed.
  - `url` — The feed’s `twtxt.txt` URL.

- `following`  
  The number of feeds this feed follows.  
  This is automatically set by the client based on the number of entries in `follows`.

- `links`  
  Additional profile links. Each entry contains:
  - `text` — A label (for example, `"GitHub"`)
  - `url` — The associated URL  
  These may be shown on a user’s profile page.

- `prev`  
  A URL that points to an archived feed.

- `refresh`  
  A suggested refresh interval (in seconds) for how often clients should fetch the feed.

#### Example

```toml
[metadata]
nick = "john"
urls = ["https://example.com/twtxt.txt"]
description = "My personal twtxt feed"
avatar = "https://example.com/avatar.png"
kind = "user"
refresh = 600

prev = ["https://old.example.com/twtxt.txt"]

[[metadata.follows]]
text = "jane"
url = "https://someone.dev/twtxt.txt"

[[metadata.links]]
text = "GitHub"
url = "https://github.com/username"
```

### `[paths]`

Contains paths to your twtxt files.

- `twtxt`  
  The filepath to your local `twtxt.txt` file.
  New posts are appended to this file when you publish a tweet.

#### Example

```toml
[paths]
twtxt = "C:/path/to/twtxt.txt"
```

---

## Running

You will need the following prerequisites:

- Rust
- Cargo

You can install both of these through a tool called [Rustup](https://rust-lang.org/learn/get-started/#:~:text=Rustup%3A%20the%20Rust%20installer%20and%20version%20management%20tool).

After installing, simply clone this repository and run `cargo run` in the root directory. Cargo should automatically install any dependencies.

If you encounter any issues, [please open one!](https://github.com/taxevaiden/twtGUI/issues/new)

## License

twtGUI is licensed under the MIT License. See the [LICENSE](https://github.com/taxevaiden/twtGUI/blob/main/LICENSE) file for details.

twtGUI uses the Iosevka Aile font, which is licensed under the SIL Open Font License. See the [Iosevka](https://github.com/be5invis/Iosevka) repository for details.
