# Regex Visualiser

[![dependency status](https://deps.rs/repo/github/Pixelstormer/regex_visualiser/status.svg)](https://deps.rs/repo/github/Pixelstormer/regex_visualiser)
[![Build Status](https://github.com/Pixelstormer/regex_visualiser/workflows/CI/badge.svg)](https://github.com/Pixelstormer/regex_visualiser/actions?workflow=CI)

An application made using [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) and [egui](https://github.com/emilk/egui/) to visualise the structure of regular expressions and the way they match text. Currently in active development, with no guarantees about stability or usability.

## Running

### Github Pages

The [Github Actions](https://docs.github.com/en/actions) workflow at `.github/workflows/pages.yml` is used to compile the app to Wasm and deploy it to [Github Pages](https://docs.github.com/en/pages) on demand - You can check out the deployed app at <https://pixelstormer.github.io/regex_visualiser/>.

### Native

No precompiled binaries are available yet, so if you wish to run the app natively, you must manually download and compile it. Instructions to do this can be found in the **Building** section below.

## License

Regex Visualiser is licensed under the Apache License Version 2.0, as detailed in the `LICENSE` file.

## Building

### Native

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

### Wasm

Regex Visualiser can be compiled to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and published as a web page. This is done using [Trunk](https://trunkrs.dev/):

1. Run `cargo install --locked trunk` to install Trunk.
2. Run `trunk serve` to build the web app and serve it to a local web server.
3. Open http://127.0.0.1:8080/index.html#dev to view the served application.

The finished web app is found in the `dist/` folder.
Besides the files that are copied verbatim from the `wasm/` and `wasm/icons/` folders, it includes 3 generated files:

* `index.html`: A few lines of HTML, CSS and JS that loads the app.
* `regex_visualiser_bg.wasm`: What the Rust code compiles to.
* `regex_visualiser.js`: Auto-generated bindings between Rust and JS.

### Service Worker Caching

A service worker (See `./wasm/sw.js`) is used to cache the web app so it can be loaded and ran even while offline.
During development however, this cache could potentially return a stale build, so `./wasm/index.html` contains a snippet of code that allows you to disable this caching by appending `#dev` to the URL (As with the URL given in Step 3 above).
