# Regex Visualiser

[![dependency status](https://deps.rs/repo/github/Pixelstormer/regex_visualiser/status.svg)](https://deps.rs/repo/github/Pixelstormer/regex_visualiser)
[![Build Status](https://github.com/Pixelstormer/regex_visualiser/workflows/CI/badge.svg)](https://github.com/Pixelstormer/regex_visualiser/actions?workflow=CI)

A program made using [eframe](https://github.com/emilk/egui/tree/master/eframe) and [egui](https://github.com/emilk/egui/) to visualise the structure of regular expressions and the way they match text.

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

The finished web app is found in the `dist/` folder. It consists of three files:

* `index.html`: A few lines of HTML, CSS and JS that loads the app.
* `regex_visualiser_bg.wasm`: What the Rust code compiles to.
* `regex_visualiser.js`: Auto-generated binding between Rust and JS.

You can check out the published app at <https://pixelstormer.github.io/regex_visualiser/>.

### Service Worker Caching

A service worker (See `./wasm/sw.js`) is used to cache the web app so it can be loaded and ran even while offline.
During development however, this cache could potentially return a stale build, so `./wasm/index.html` contains a snippet of code that allows you to disable this caching by appending `#dev` to the URL (As with the URL given in Step 3 above).
