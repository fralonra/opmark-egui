# opmark-egui

[![Latest version](https://img.shields.io/crates/v/opmark-egui.svg)](https://crates.io/crates/opmark-egui)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

An experimental presentation application based on [OpMark](https://crates.io/crates/opmark), powered by [egui](https://crates.io/crates/egui). It's still in pre-alpha stage.

## Usage

### Installation

```bash
cargo install opmark-egui
```

### Running with OpMark document

Find a `./index.opmark` to make presentations:

```bash
opmark-egui
```

Sepecify a file to use:

```bash
opmark-egui examples/test.opmark
```

### Build a standalone binary file

```bash
opmark-egui -s examples/test.opmark
```

The above command will generate an executable bundling your OpMark file and other assets (only images are supported for now) as well.

You can execute this file directly without copying assets and refering them:

```bash
./opmark
```

## Credits

Inspired by [easymark](https://github.com/emilk/egui/tree/master/egui_demo_lib/src/easy_mark).
