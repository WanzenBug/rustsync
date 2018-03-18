extern crate futures;
extern crate walkdir;


mod error;
mod local;

use futures::Stream;
use std::path::PathBuf;
pub use error::{Error, ErrorKind, Result};
pub use local::{LocalDrain, LocalSource};

type SyncFuture<T> = Box<futures::Future<Item = T, Error = Error>>;

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct BasicFileInfo {
    pub path: PathBuf,
    pub kind: FileType,
}

pub trait SyncSource: Stream<Item = BasicFileInfo, Error = Error> {
    fn fetch_file(&self, info: BasicFileInfo) -> SyncFuture<Box<futures::io::AsyncRead>>;
}

pub trait SyncDrain {
    fn check_destination(&self, info: BasicFileInfo) -> SyncFuture<Option<BasicFileInfo>>;
    fn write_file(&self, info: BasicFileInfo) -> SyncFuture<Box<futures::io::AsyncWrite>>;
}

pub struct Syncer<S, D>
where
    S: SyncSource,
    D: SyncDrain, {
    source: S,
    drain: D,
}

pub struct SyncerFuture<'a> {
    future: Box<'a + futures::Future<Item = (), Error = Error>>,
}

impl<'a> futures::Future for SyncerFuture<'a> {
    type Item = ();
    type Error = Error;

    fn poll(&mut self, cx: &mut futures::task::Context) -> futures::Poll<Self::Item, Self::Error> {
        self.future.poll(cx)
    }
}

impl<S, D> Syncer<S, D> where
    S: SyncSource,
    D: SyncDrain, {

    pub fn new(source: S, drain: D) -> Self {
        Syncer { source, drain }
    }

    pub fn future<'a>(&'a mut self) -> SyncerFuture<'a> {
        use futures::{FutureExt, StreamExt};

        let source = &mut self.source;

        let drain = &mut self.drain;

        let for_each = source
            .and_then(move |src_info| {
                let inner = drain.check_destination(src_info.clone());
                inner.map(|dest_info| (src_info, dest_info))
            })
            .for_each(|res| {
                println!("{:?}", res);
                Ok(())
            })
            .map(|s| ());

        SyncerFuture {
            future: Box::new(for_each),
        }
    }
}
