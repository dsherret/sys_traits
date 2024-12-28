use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::BaseFsOpen;
use crate::BaseFsReadDir;
use crate::FileType;
use crate::FsDirEntry;
use crate::FsFile;
use crate::FsFileSetLen;
use crate::FsFileSetPermissions;
use crate::FsMetadataValue;
use crate::OpenOptions;

// == FsOpenBoxed ==

pub struct BoxedFsFile(pub Box<dyn FsFile + 'static>);

impl io::Read for BoxedFsFile {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.0.read(buf)
  }
}

impl io::Seek for BoxedFsFile {
  #[inline]
  fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
    self.0.seek(pos)
  }
}

impl io::Write for BoxedFsFile {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.0.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> io::Result<()> {
    self.0.flush()
  }
}

impl FsFileSetLen for BoxedFsFile {
  #[inline]
  fn fs_file_set_len(&mut self, size: u64) -> io::Result<()> {
    self.0.fs_file_set_len(size)
  }
}

impl FsFileSetPermissions for BoxedFsFile {
  #[inline]
  fn fs_file_set_permissions(&mut self, perm: u32) -> io::Result<()> {
    self.0.fs_file_set_permissions(perm)
  }
}

impl FsFile for BoxedFsFile {}

pub trait FsOpenBoxed {
  fn fs_open_boxed(
    &self,
    path: &Path,
    open_options: &OpenOptions,
  ) -> io::Result<BoxedFsFile>;
}

impl<TFile: FsFile + 'static, T: BaseFsOpen<File = TFile>> FsOpenBoxed for T {
  fn fs_open_boxed(
    &self,
    path: &Path,
    open_options: &OpenOptions,
  ) -> io::Result<BoxedFsFile> {
    self
      .base_fs_open(path, open_options)
      .map(|file| BoxedFsFile(Box::new(file)))
  }
}

// == FsReadDirBoxed ==

#[derive(Debug)]
pub struct BoxedFsMetadataValue(pub Box<dyn FsMetadataValue>);

impl FsMetadataValue for BoxedFsMetadataValue {
  #[inline]
  fn file_type(&self) -> FileType {
    self.0.file_type()
  }

  #[inline]
  fn len(&self) -> u64 {
    self.0.len()
  }

  #[inline]
  fn accessed(&self) -> io::Result<SystemTime> {
    self.0.accessed()
  }

  #[inline]
  fn changed(&self) -> io::Result<SystemTime> {
    self.0.changed()
  }

  #[inline]
  fn created(&self) -> io::Result<SystemTime> {
    self.0.created()
  }

  #[inline]
  fn modified(&self) -> io::Result<SystemTime> {
    self.0.modified()
  }

  #[inline]
  fn dev(&self) -> io::Result<u64> {
    self.0.dev()
  }

  #[inline]
  fn ino(&self) -> io::Result<u64> {
    self.0.ino()
  }

  #[inline]
  fn mode(&self) -> io::Result<u32> {
    self.0.mode()
  }

  #[inline]
  fn nlink(&self) -> io::Result<u64> {
    self.0.nlink()
  }

  #[inline]
  fn uid(&self) -> io::Result<u32> {
    self.0.uid()
  }

  #[inline]
  fn gid(&self) -> io::Result<u32> {
    self.0.gid()
  }

  #[inline]
  fn rdev(&self) -> io::Result<u64> {
    self.0.rdev()
  }

  #[inline]
  fn blksize(&self) -> io::Result<u64> {
    self.0.blksize()
  }

  #[inline]
  fn blocks(&self) -> io::Result<u64> {
    self.0.blocks()
  }

  #[inline]
  fn is_block_device(&self) -> io::Result<bool> {
    self.0.is_block_device()
  }

  #[inline]
  fn is_char_device(&self) -> io::Result<bool> {
    self.0.is_char_device()
  }

  #[inline]
  fn is_fifo(&self) -> io::Result<bool> {
    self.0.is_fifo()
  }

  #[inline]
  fn is_socket(&self) -> io::Result<bool> {
    self.0.is_socket()
  }

  #[inline]
  fn file_attributes(&self) -> io::Result<u32> {
    self.0.file_attributes()
  }
}

#[derive(Debug)]
struct MappedMetadataFsDirEntry<T: FsDirEntry + 'static>(T);

impl<T: FsDirEntry + 'static> FsDirEntry for MappedMetadataFsDirEntry<T> {
  type Metadata = BoxedFsMetadataValue;

  #[inline]
  fn file_name(&self) -> Cow<OsStr> {
    self.0.file_name()
  }

  #[inline]
  fn file_type(&self) -> io::Result<FileType> {
    self.0.file_type()
  }

  #[inline]
  fn metadata(&self) -> io::Result<Self::Metadata> {
    self
      .0
      .metadata()
      .map(|metadata| BoxedFsMetadataValue(Box::new(metadata)))
  }

  #[inline]
  fn path(&self) -> Cow<Path> {
    self.0.path()
  }
}

#[derive(Debug)]
pub struct BoxedFsDirEntry(
  Box<dyn FsDirEntry<Metadata = BoxedFsMetadataValue>>,
);

impl BoxedFsDirEntry {
  pub fn new<T: FsDirEntry + 'static>(entry: T) -> Self {
    Self(Box::new(MappedMetadataFsDirEntry(entry)))
  }
}

impl FsDirEntry for BoxedFsDirEntry {
  type Metadata = BoxedFsMetadataValue;

  #[inline]
  fn file_name(&self) -> Cow<OsStr> {
    self.0.file_name()
  }

  #[inline]
  fn file_type(&self) -> io::Result<FileType> {
    self.0.file_type()
  }

  #[inline]
  fn metadata(&self) -> io::Result<Self::Metadata> {
    self.0.metadata()
  }

  #[inline]
  fn path(&self) -> Cow<Path> {
    self.0.path()
  }
}

pub trait FsReadDirBoxed {
  fn fs_read_dir_boxed(
    &self,
    path: &Path,
  ) -> io::Result<Box<dyn Iterator<Item = io::Result<BoxedFsDirEntry>>>>;
}

impl<T: BaseFsReadDir> FsReadDirBoxed for T {
  fn fs_read_dir_boxed(
    &self,
    path: &Path,
  ) -> io::Result<Box<dyn Iterator<Item = io::Result<BoxedFsDirEntry>>>> {
    let iter = self.base_fs_read_dir(path)?;
    Ok(Box::new(
      iter.map(|result| result.map(BoxedFsDirEntry::new)),
    ))
  }
}
