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

twtGUI uses a configuration file, `config.ini`, to store user settings. 

| Platform | File path |
|-|-|
| Windows | C:\Users\yourname\AppData\Roaming\taxevaiden\twtGUI\config\config.ini |
| macOS | /Users/yourname/Library/Application Support/com.taxevaiden.twtGUI/config.ini |
| Linux | /home/yourname/.config/twtgui/config.ini |

Here are the available settings:

- `[settings]`
  - `nick`: Your nickname for the client.
  - `twtxt`: The filepath to your twtxt.txt file.
  - `twturl`: The URL of your twtxt.txt file.

There is no settings page yet, so you will have to edit the `config.ini` file manually.

### Metadata

twtGUI supports the Metadata Extension, which allows you to add metadata to your feed.

To do this, you will also have to edit your twtxt.txt file manually.

```
# nick = john
# description = My (awesome) personal feed!
# url = https://example.com/twtxt.txt
# avatar = https://example.com/avatar.png
# following = jane https://example.com/twtxt.txt
# following = joe https://example.com/twtxt.txt

2022-01-01T12:00:00Z Hello, world!
```

For more information about the Metadata Extension, see [the twtxt v2 specification](https://twtxt.dev/exts/metadata.html).

## Running

To run twtGUI, you will need the following:

- Rust
- Cargo

You can install both of these through a tool called [Rustup](https://rust-lang.org/learn/get-started/#:~:text=Rustup%3A%20the%20Rust%20installer%20and%20version%20management%20tool).

After installing, simply clone this repository and run `cargo run` in the root directory.

If you encounter any issues, please open an issue!
