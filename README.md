# Cwiid Library for Rust

This library provides an api to communicate with the Nintendo Wii Wiimote.

The library is based on the [Cwiid C library](https://github.com/abstrakraft/cwiid). 

Currently only the Wiimote itself is supported but accessories are planned for future versions.

The current version is an untested alpha release. Nothing is tested and documented.

The core struct of this library is the WiiMote struct. It represents 1 Wiimote control per instance. A new controller can be connected by calling either WiiMote::new() to get the a randomly selected Wiimote or the WiiMote::find(..) function to find a specific Wiimote.