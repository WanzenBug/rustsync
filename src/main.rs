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
            .required(true)
            .multiple(true))
        .arg(clap::Arg::with_name("DEST")
            .required(true))
        .get_matches();


    let source_paths = matches.values_of("SRC").expect("Clap should have caught missing values!");
    let dest_path = matches.value_of("DEST").expect("Clap should have caught missing values!");
    let mut src = rustsync::MultiSource::new();
    for p in source_paths {
        let strip_root = p.ends_with(MAIN_SEPARATOR);
        src.add_source(rustsync::local::LocalSource::with_strip_root_dir(p, strip_root))
    }
    let drain = rustsync::local::LocalDrain::new(dest_path);

    rustsync::sync(src, drain, rustsync::SuppressUpdate{}).expect("Something went wrong");
}
