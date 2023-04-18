use std::process::Command;

fn main() {
    let libfiles = ["stdin", "termio"];
    for f in libfiles {
        println!("cargo:rerun-if-changed=linklibs/{}.bc", f);
        Command::new("clang")
            .args([
                "-c",
                &format!("linklibs/{}.c", f),
                "-o",
                &format!("linklibs/{}.bc", f),
                "-emit-llvm",
                "-O2",
            ])
            .spawn()
            .expect("Couldn't build");
    }
}
