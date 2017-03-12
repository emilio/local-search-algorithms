use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("emscripten") {
        let dest = env::var("OUT_DIR").unwrap();
        let dest = Path::new(&dest);

        // Dump it in the same directory as the executable.
        let dest = dest.parent().unwrap().parent().unwrap().parent().unwrap();

        Command::new("tsc")
            .arg("--module")
            .arg("amd")
            .arg("src/application.ts")
            .arg("--outFile")
            .arg(dest.join("application.js"))
            .output()
            .expect("Couldn't run tsc, or it failed!");

        Command::new("cp")
            .arg("src/index.html")
            .arg(dest.join("index.html"))
            .output()
            .expect("Couldn't copy the index.html file!");
    }
}
