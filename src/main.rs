extern crate rustsync;
extern crate clap;
extern crate pbr;
use std::path::MAIN_SEPARATOR;

fn main() {
    let matches = clap::App::new("rustsync")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(clap::Arg::with_name("SRC")
            .required(true))
        .arg(clap::Arg::with_name("DEST")
            .required(true))
        .get_matches();


    let source_path = matches.value_of("SRC").expect("Clap should have caught missing values!");
    let dest_path = matches.value_of("DEST").expect("Clap should have caught missing values!");
    let src = rustsync::local::LocalSource::new(source_path);
    let drain = rustsync::local::LocalDrain::new(dest_path);

    let sync = rustsync::Syncer::new(src, drain);
    sync.sync().expect("Something went wrong");
}
