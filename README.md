# OpenCV webcam effects

This is a simple app to fetch as stream of your webcam and apply some simple effects.
It is written in Rust using the **opencv-rust** crate.

## Building and running

To build this, you need to have [Rust nightly installed](https://www.rust-lang.org/tools/install).

After setting up `rust-nightly` you can run this project using `cargo run --bin opencv` or `cargo run --bin imageproc` 
for each of the two implementations.

For the effects checkboxes in the OpenCV implementation, you need to click the paint brush button in the toolbar:

![screenshot](./screenshot.png)
