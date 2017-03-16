# Local search algoritms implementation in Rust to Emscripten

This is part of a **Work in progress** I'm doing for an Artificial Intelligence
course.

In particular, I can't promise the algorithms are correct, since I've only done
this for a few days, though I hope so!

Also, the interface is quite ugly. I'm sorry.

The program consists of a Rust file compiled to emscripten, an index.html file,
and a TypeScript file that compiles to JavaScript for the application event
handling UI.

Some known gotchas:

 * It's ugly (for now at least, I haven't bothered a lot in designing it, I just
   wanted a damn chess on screen).

 * Only shows the best state for Local Beam Search and Genetic Algorithm.

 * Could be more memory efficient (just read the TODOs in `src/main.rs`).

## Requirements

 * [Rust nightly](http://rustup.rs/).
 * TypeScript compiler: `nmp install -g typescript`.
 * Emscripten toolchain ([Varies by system][emscripten]).

To test the code you only need Rust nightly. Running `cargo test` should work.

For running it:

```console
$ cargo build --target asmjs-unknown-emscripten
$ firefox ./target/asmjs-unknown-emscripten/debug/index.html
```

[emscripten]: https://kripken.github.io/emscripten-site/docs/getting_started/downloads.html
