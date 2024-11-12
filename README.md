# Welcome to RetroDungeon
This is an old-school turn based roguelike written in rust, without an engine. It featrues a custom ECS implementation, event driven interactions between entities and high degree of serialization of unit types, spawn tables, and interactions. Rendering and UI is done with [slint](https://github.com/slint-ui/slint).

All spawn tables, all entities, all event interactions are specified in yaml format game files. In the future, player spell are also going to be tweakable in this fasion, as well as possibly the map generation. 

I wrote this project to learn about entity component systems and to learn the rust programming language. For a look at what my first attempts looked like, check out the [rust-game](https://github.com/somantics/rust-game) project. This project represents whan I could do after rewriting the base after a 6 month internship at a mobile game company, Turborilla. 

## Build instructions
These instructions have been tested on rust 1.82. 

For environment with access to bash, simply run the build.sh script which will compile with cargo and move the game data to the proper folder. 
You may run into issues building the default skia backend, in which case you may edit the script or build manually FemtoVG backed. 

NOTE: build.sh is the one you want, not build.rs, which is used internally by cargo.

### Manual build (or specifying alternative backend)

When building manually, please speficy a backend you wish to use, either skia or FemtoVG. skia is prefered, though may be more annoying to build. 
```
cargo build --release --features skia
```

If you encounter issues with the skia backend during build, you may instead opt for FemtoVG. Either works fine for this project.
```
cargo build --release --features femtovg
```

Enabling both features leaves the decision of which to use up to the slint runtime. 
```
cargo build --release --features "femtovg skia"
```
After compiling with cargo you need to copy the contents of the data folder into a data folder in your build directory (target/release/ by default).

You can also modify the bash build script by simply replacing the line that calls cargo with the lines above. 

## Dependencies
Femtosvg requires OpenGL2.0. 
Skia may require additional build tools. 

## Platform support
Slint boasts support for Windows, linux, mac, and more. 

Tested on linux on both wayland and x11. (Manjaro 24, openSuse Leap 15.6, openSuse Tumbleweed (which version?), Ubuntu 24.04) 
