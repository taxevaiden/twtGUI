# twtxt crate

Helper crate for parsing + loading twtxt feeds, with most of the twtxt v2 extensions supported.

The Archive Feeds extension is not implemented as falls outside the scope of this crate, however it is trivial to implement the extension in your own client. The `prev` field is provided in the `Metadata` struct. Just read the docs: https://twtxt.dev/exts/archive-feeds.html

Mainly used by twtGUI.
