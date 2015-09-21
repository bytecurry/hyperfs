# HyperFS
A simple HTTP server for static files.

HyperFS is a very simple http server that simply serves the files in the directory
it is run from. It is primarily intended as a development tool similar to using
`python -m SimpleHTTPServer`. But it is a single lightweight executable written in rust.

To be honest, part of the motivation was to just so I could try out rust.

## Usage

To use simple cd into the directory with the files you want to server and run

```bash
$ hyperfs >port<
```

where `>port<` is the port you want to listen on. If not supplied it will listen on port
3000.

## Installation

You can either download the binary from the latest release (currently only 64-bit linux version
is available). Or you can build from source using

```bash
$ cargo build --release
```
and the resulting binary will be at 'target/release/hyperfs'.
