use std::process::Command;

fn main() {
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["./abi.proto"], &["."])
        .unwrap();

    Command::new("cargo").args(&["fmt"]).output().unwrap();
    println!("cargo:rerun-if-changed=abi.proto");
}
