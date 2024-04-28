# CoCtus

Command line tool that allows you to play [Clash of Code](https://www.codingame.com/multiplayer/clashofcode) style problems locally on your computer.

## Demo

<div align=center>

[![coctus demo](https://github.com/Andriamanitra/coctus/assets/10672443/518bb6ca-6059-4866-a69a-e381aa31cc82)](https://asciinema.org/a/656708)

</div>

## Usage

Detailed [user guide](https://github.com/Andriamanitra/coctus/wiki/User-guide) is available in the wiki.


## Installation

The program has only been tested on Linux and Windows. Other platforms may or may not work!

### (Option 1) Install latest release from crates.io
You may need to install `pkg-config` and `libssl-dev` or equivalent for this to work (`apt install pkg-config libssl-dev` on Ubuntu).
```
$ cargo install coctus
```

### (Option 2) Download latest release as a pre-built executable

Download the latest binary for your operating system from [releases](https://github.com/Andriamanitra/coctus/releases) and extract it somewhere on your `$PATH`.

### (Option 3) Build from source (recommended for developers)

This method requires `git` and a relatively recent (1.73+ should work) version of the Rust toolchain.
```
$ git clone https://github.com/Andriamanitra/coctus
$ cargo install --path=.
```


## Contributing

Use [Github issues](https://github.com/Andriamanitra/coctus/issues) for bug reports and features requests.
Pull requests are also welcome, but please open an issue beforehand to discuss bigger changes.
