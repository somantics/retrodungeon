#!/bin/bash/

# Use cargo to build the project
# make sure to use features "skia" or features "femtovg"
# this lets cargo know which backend to compile
cargo build --release --features skia

# Copying over game data to the build directory
mkdir target/release/data
cp -r data/* target/release/data
