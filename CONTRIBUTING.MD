# Contributing

Thank you for showing interest in supporting the development of twtGUI! twtGUI is a solo project at the moment, so I truly appreciate your help!

Please read this before opening a pull request to avoid wasted effort.

## Getting Started

As detailed in the readme, you'll need Rust and Cargo installed. You can install them with [Rustup.](https://rustup.rs/)

    git clone https://github.com/taxevaiden/twtGUI
    cd twtGUI
    cargo run            # debug build
    cargo run --release  # optimized build (slower to compile, faster to run)
    
Cargo should automatically install any dependencies.

## Before Contributing

Because this project is under active development and may introduce breaking changes, you should open an issue first before working on anything significant. Then we can discuss whether it fits twtGUI's direction and prevent any conflicts with ongoing work.

For small changes (typos, minor bugs with obvious fixes, documentation), you can go straight to opening a pull request.

Some good places to start are:
- Issues labeled with `good first issue`
- A missing/unfinished implementation of the [twtxt v2 specification's extensions](https://twtxt.dev/)

## Code

Please run the following before submitting a pull request:

    cargo fmt
    cargo clippy

Pull requests should pass both `cargo fmt` and `cargo clippy` without any warnings or errors before being merged.

## Pull Request Guidelines

To ensure things are simple and maintainable:

- Please keep them small and focused (i probably won't have time to review large PRs... i'm sorry...)
- Target the `main` branch
- Describe your changes and why you're making them
- Reference any relevant issues

## License

By submitting a pull request, you agree to license your work under the [MIT License](LICENSE).
