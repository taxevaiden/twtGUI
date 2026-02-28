# twtGUI

A graphical client for the twtxt protocol

## What's supported?

- Sending tweets
- Fetching (and caching) feeds
- Following other feeds
- **Some** of the [twtxt v2 specification](https://twtxt.dev)
  - Mentions
  - [Twt Hash Extension](https://twtxt.dev/exts/twt-hash.html)
  - [Twt Subject Extension](https://twtxt.dev/exts/twt-subject.html) (You can only see if a tweet is a reply for now)
  - [Metadata Extension](https://twtxt.dev/exts/metadata.html)

If you're someone whose twtxt.txt only follows the twtxt v1 specification, this client's great for you!

If you're someone whose twtxt.txt follows the twtxt v2 specification, expect some features to be missing.

## Configuration

twtGUI uses a `config.toml` file to store user settings and follow information.  
The file is automatically created on first launch if it does not already exist.

There is currently no dedicated settings page in twtGUI, so you will have to edit `config.toml` manually to change settings like your nickname or the path to your twtxt.txt.

However, you can manage the feeds you follow through the Following page.

If you need to edit `config.toml`, you can find it in your system’s configuration directory:

| Platform | File path                                                                     |
|-         |-                                                                              |
| Windows  | C:\Users\yourname\AppData\Roaming\taxevaiden\twtGUI\config\config.toml        |
| macOS    | /Users/yourname/Library/Application Support/com.taxevaiden.twtGUI/config.toml |
| Linux    | /home/yourname/.config/twtgui/config.toml                                     |

The configuration is divided into two main sections: `[metadata]` and `[paths]`.

---

### `[metadata]`

Contains information about you and your twtxt identity.

- `nick`  
  Your nickname displayed in the client.

- `urls`  
  A list of public URLs pointing to your `twtxt.txt` file.  
  Typically this contains a single URL.

- `follows`  
  A list of feeds you follow. Each entry contains:
  - `text` — The display name of the feed.
  - `url` — The feed’s `twtxt.txt` URL.

#### Example

```toml
[metadata]
nick = "taxevaiden"
urls = ["https://example.com/twtxt.txt"]

[[metadata.follows]]
text = "someone"
url = "https://someone.dev/twtxt.txt"
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

### Metadata Extension

twtGUI supports the **twtxt Metadata Extension**, which allows you to include additional information about your feed at the top of your `twtxt.txt` file.

Metadata is written as comment lines (`#`) before your posts.  
This must be edited manually in your `twtxt.txt` file.

Example:

```
# nick = john
# description = My (awesome) personal feed!
# url = https://example.com/twtxt.txt
# avatar = https://example.com/avatar.png
# following = 2
# follow = jane https://example.com/twtxt.txt
# follow = joe https://example.com/twtxt.txt

2022-01-01T12:00:00Z Hello, world!
```

#### Common Fields

- `nick` — Your display name.
- `description` — A short bio or description of your feed.
- `url` — The public URL of your `twtxt.txt`.
- `avatar` — A URL to your profile image.
- `following` — The number of feeds you follow.
- `follow` — A feed you follow (`name` + `url`).

Metadata must appear at the top of the file, before any posts.

For full details, see the official [twtxt v2 Metadata specification](https://twtxt.dev/exts/metadata.html).

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
