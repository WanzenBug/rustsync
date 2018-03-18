extern crate rustsync;

extern crate clap;

extern crate futures;

fn main() {
    let parser = clap::App::new("rustsync")
        .arg(clap::Arg::with_name("SRC").required(true))
        .arg(clap::Arg::with_name("DEST").required(true));

    let matches = parser.get_matches();
    let src = matches
        .value_of_os("SRC")
        .expect("No src, should be caught in parser");
    let dest = matches
        .value_of_os("DEST")
        .expect("No dest, should be caught in parser");

    let sync_src = rustsync::LocalSource::new(src);
    let sync_dest = rustsync::LocalDrain::new(dest);

    let mut sync = rustsync::Syncer::new(sync_src, sync_dest);
    use futures::IntoFuture;
    let mut pool = futures::executor::ThreadPool::new();

    let exit = match pool.run(sync.future()) {
        Ok(()) => {
            println!("Success");
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(exit)
}
