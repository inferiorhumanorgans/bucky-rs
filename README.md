bucky-rs
========

`bucky` is a library for making structure driven charts (a riff on D3's data driven documents).  `bucky` reinterprets the [d3](https://d3js.org) Javascript library in a rusty manner.  In many ways the interfaces and feature sets are similar, but `bucky` aims to leverage rust's much broader trait system and much broader standard library.

`bucky` is primarily useful where one wants to generate pretty, static images without having to run in an environment without Javascript or without a DOM implementation.  For example rendering graphs to insert into a PDF or to view outside of a browser context.

A typical workflow could involve: a bit of rust to generate the SVG document followed rsvg-covert to convert the SVG to a PDF or raster image.  ImageMagick appears to be better at converting PDFs to PNGs than rsvg-convert is at converting SVGs to PNGs.

`bucky` is pre-alpha quality, may corrode your pipes if used in production, and is foremost an dive into learning Rust and experimenting with what a charting API could look like in rust.  Use at your own risk per the terms of the GNU GPL v3 or later.

### TODO
* There are currently copies of [date-iterator](https://github.com/kosta/date-iterator) and [minidom-rs](https://gitlab.com/xmpp-rs/minidom-rs) in tree.  Changes should be merged upstream.
* Additional tests
* Work down the TODO comments
* Migrate assertions to proper error handling
