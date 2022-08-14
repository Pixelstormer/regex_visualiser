# Regex Visualiser

[![dependency status](https://deps.rs/repo/github/Pixelstormer/regex_visualiser/status.svg)](https://deps.rs/repo/github/Pixelstormer/regex_visualiser)
[![Build Status](https://github.com/Pixelstormer/regex_visualiser/workflows/CI/badge.svg)](https://github.com/Pixelstormer/regex_visualiser/actions?workflow=CI)

A program made using [eframe](https://github.com/emilk/egui/tree/master/eframe) and [egui](https://github.com/emilk/egui/) to visualise the structure of regular expressions and the way they match text.

## Building

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.

### Compiling for the web

Install [jq](https://stedolan.github.io/jq/download/).

Make sure you are using the latest version of stable rust by running `rustup update`.

Regex Visualiser can be compiled to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and published as a web page. For this you need to set up some tools. There are a few simple scripts that help you with this:

```sh
./setup_web.sh
./start_server.sh
./build_web.sh --optimize --open
```

* `setup_web.sh` installs the tools required to build for web
* `start_server.sh` starts a local HTTP server so you can test before you publish
* `build_web.sh` compiles the code to WASM and puts it in the `docs/` folder (see below) and `--open` opens the result in your default browser.

The finished web app is found in the `docs/` folder (this is so that it can easily be published with [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)). It consists of three files:

* `index.html`: A few lines of HTML, CSS and JS that loads the app.
* `regex_visualiser_bg.wasm`: What the Rust code compiles to.
* `regex_visualiser.js`: Auto-generated binding between Rust and JS.

You can check out the published app at <https://pixelstormer.github.io/regex_visualiser/>.

### Web testing/development

Open `index.html#dev` to disable caching, which makes development easier.
