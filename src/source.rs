use ::{Result, SyncableInfo};
use std::{io};


pub trait SourceSyncable {
    type Reader: io::Read;

    fn info(&self) -> Result<SyncableInfo>;
    fn reader(&mut self) -> Result<Self::Reader>;
}

struct BoxedSourceSyncable<S>(S) where S: SourceSyncable;

impl<S> SourceSyncable for BoxedSourceSyncable<S> where S: 'static + SourceSyncable {
    type Reader = Box<io::Read>;

    fn info(&self) -> Result<SyncableInfo> {
        self.0.info()
    }

    fn reader(&mut self) -> Result<Self::Reader> {
        Ok(Box::new(self.0.reader()?) as Box<io::Read>)
    }
}

impl<S> SourceSyncable for Box<S> where S: 'static + ? Sized + SourceSyncable {
    type Reader = S::Reader;

    fn info(&self) -> Result<SyncableInfo> {
        use std::ops::Deref;
        self.deref().info()
    }

    fn reader(&mut self) -> Result<Self::Reader> {
        use std::ops::DerefMut;
        self.deref_mut().reader()
    }
}

pub trait Source {
    type Iter: Iterator<Item=Result<Self::Syncable>>;
    type Syncable: SourceSyncable;

    fn iter_file_info(&mut self) -> Result<Self::Iter>;
}

struct BoxedSource<S>(S) where S: 'static + Source;

impl<S> Source for BoxedSource<S> where S: 'static + Source {
    type Iter = Box<Iterator<Item=Result<Self::Syncable>>>;
    type Syncable = Box<SourceSyncable<Reader=Box<io::Read>>>;

    fn iter_file_info(&mut self) -> Result<Self::Iter> {
        self.0.iter_file_info().map(|i| {
            let i = i.map(|x| x.map(|y| Box::new(BoxedSourceSyncable(y)) as Box<SourceSyncable<Reader=Box<io::Read>>>));
            Box::new(i) as Box<Iterator<Item=_>>
        })
    }
}

impl<S> Source for Box<S> where S: 'static + ? Sized + Source {
    type Iter = S::Iter;
    type Syncable = S::Syncable;

    fn iter_file_info(&mut self) -> Result<Self::Iter> {
        use std::ops::DerefMut;
        self.deref_mut().iter_file_info()
    }
}

type SourceSyncableBox = Box<SourceSyncable<Reader=Box<io::Read>>>;
type SourceVec<I, S> = Vec<Box<Source<Iter=I, Syncable=S>>>;

#[derive(Default)]
pub struct MultiSource(SourceVec<Box<Iterator<Item=Result<SourceSyncableBox>>>, SourceSyncableBox>);

impl MultiSource {
    pub fn new() -> Self {
        MultiSource(Vec::new())
    }

    pub fn add_source<S>(&mut self, source: S) where S: 'static + Source {
        let boxed = Box::new(BoxedSource(source)) as Box<Source<Iter=Box<Iterator<Item=Result<Box<SourceSyncable<Reader=Box<io::Read>>>>>>, Syncable=SourceSyncableBox>>;
        self.0.push(boxed);
    }
}

impl Source for MultiSource {
    fn iter_file_info(&mut self) -> Result<Self::Iter> {
        let iters: Result<Vec<_>> = self.0.iter_mut().map(|x| x.iter_file_info()).collect();
        Ok(Box::new(iters?.into_iter().flat_map(|x| x)) as Self::Iter)
    }
    type Iter = Box<Iterator<Item=Result<SourceSyncableBox>>>;
    type Syncable = Box<SourceSyncable<Reader=Box<io::Read>>>;
}