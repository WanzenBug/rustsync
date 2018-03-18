use {BasicFileInfo, Error, FileType, Result};
use std::sync::mpsc;
use std::thread;
use std::path::{Path, PathBuf};

pub struct LocalSource {
    path_queue: mpsc::Receiver<Result<BasicFileInfo>>,
    walker: OffThreadWalker,
}

enum OffThreadWalker {
    Started(thread::JoinHandle<()>),
    Pending(PathBuf, mpsc::SyncSender<Result<BasicFileInfo>>),
    Done,
}

impl OffThreadWalker {
    fn done(&mut self) -> Result<()> {
        let this = ::std::mem::replace(self, OffThreadWalker::Done);

        let res = match this {
            OffThreadWalker::Started(handle) => handle
                .join()
                .map_err(|_| Error::new("Local walker encountered unexpected error")),

            OffThreadWalker::Pending(_, _) => {
                Err(Error::new("Local walker disconnected before executing"))
            }

            OffThreadWalker::Done => Ok(()),
        };

        res
    }

    fn work(&mut self, cx: &mut ::futures::task::Context) {
        let this = ::std::mem::replace(self, OffThreadWalker::Done);

        *self = match this {
            OffThreadWalker::Pending(root, sender) => {
                let waker = cx.waker();
                let handle = thread::spawn(move || OffThreadWalker::walker(root, sender, waker));
                OffThreadWalker::Started(handle)
            }
            x => x,
        };
    }

    fn walker(
        source_location: PathBuf,
        sender: mpsc::SyncSender<Result<BasicFileInfo>>,
        waker: ::futures::task::Waker,
    ) {
        let walk = ::walkdir::WalkDir::new(source_location);

        for item in walk {
            let item = item.map_err(Into::into)
                .and_then(OffThreadWalker::extract_basic_infos);
            sender
                .send(item)
                .expect("Read end of local walker queue closed unexpectedly");
            waker.wake();
        }
        ::std::mem::drop(sender);
        waker.wake();
    }

    fn extract_basic_infos(entry: ::walkdir::DirEntry) -> Result<BasicFileInfo> {
        let path = entry.path().to_owned();

        let kind = if entry.metadata()?.is_file() {
            FileType::File
        } else {
            // TODO: This is not correct, need more types
            FileType::Directory
        };
        Ok(BasicFileInfo { path, kind })
    }
}

impl LocalSource {
    pub fn new<A: AsRef<Path>>(source_location: A) -> LocalSource {
        let (sender, receiver) = mpsc::sync_channel(30);

        let root = source_location.as_ref().to_owned();

        LocalSource {
            path_queue: receiver,

            walker: OffThreadWalker::Pending(root, sender),
        }
    }
}

impl ::SyncSource for LocalSource {
    fn fetch_file(&self, info: BasicFileInfo) -> ::SyncFuture<Box<::futures::io::AsyncRead>> {
        unimplemented!()
    }
}

impl ::futures::Stream for LocalSource {
    type Item = BasicFileInfo;
    type Error = Error;

    fn poll_next(
        &mut self,
        cx: &mut ::futures::task::Context,
    ) -> ::futures::Poll<Option<Self::Item>, Self::Error> {
        self.walker.work(cx);

        match self.path_queue.try_recv() {
            Ok(item) => match item {
                Ok(path) => Ok(::futures::Async::Ready(Some(path))),
                Err(err) => Err(err),
            },
            Err(mpsc::TryRecvError::Empty) => Ok(::futures::Async::Pending),
            Err(mpsc::TryRecvError::Disconnected) => {
                self.walker.done()?;
                Ok(::futures::Async::Ready(None))
            }
        }
    }
}
