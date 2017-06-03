extern crate rustsync;
extern crate clap;
extern crate pbr;
use std::path::MAIN_SEPARATOR;
use std::io::Stdout;
use std::thread;

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
    let progress = SyncProgress(pbr::MultiBar::new());

    rustsync::sync(src, drain, progress).expect("Something went wrong");
}

pub struct SyncProgress(pbr::MultiBar<Stdout>);

impl rustsync::SyncProgress for SyncProgress {
    type FileProgress = FileProgress;

    fn next_sync<'a>(&mut self, item: &rustsync::SyncableInfo<'a>) -> Self::FileProgress {
        let size = match item {
            &rustsync::SyncableInfo::File {size: x, ..} => x,
            _ => 0,
        };
        let mut pbr = self.0.create_bar(size);
        pbr.message(&format!("{}: ", &item.get_path().to_string_lossy()));
        pbr.set_units(pbr::Units::Bytes);
        FileProgress(pbr)
    }
}
impl Drop for SyncProgress {
    fn drop(&mut self) {
        self.0.listen();
    }
}

pub struct FileProgress(pbr::ProgressBar<pbr::Pipe>);

impl rustsync::FileProgress for FileProgress {
    fn update(&mut self, transferred: u64) {
        self.0.set(transferred);
    }
}

impl Drop for FileProgress {
    fn drop(&mut self) {
        self.0.finish();
    }
}