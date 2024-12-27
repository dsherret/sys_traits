use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Component;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

// this entire module was lazily created... needs way more work

use parking_lot::RwLock;

use crate::*;

#[derive(Debug, Clone)]
pub struct InMemoryFile {
  sys: InMemorySys,
  inner: Arc<RwLock<FileInner>>,
  pos: usize,
}

impl FsFile for InMemoryFile {}

#[derive(Debug)]
struct FileInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
  data: Vec<u8>,
  mode: u32,
}

#[derive(Debug)]
struct File {
  name: String,
  inner: Arc<RwLock<FileInner>>,
}

#[derive(Debug)]
enum DirectoryEntry {
  File(File),
  Directory(Directory),
  Symlink(Symlink),
}

impl DirectoryEntry {
  fn name(&self) -> &str {
    match self {
      DirectoryEntry::File(f) => &f.name,
      DirectoryEntry::Directory(d) => &d.name,
      DirectoryEntry::Symlink(s) => &s.name,
    }
  }

  fn modified_time(&self) -> SystemTime {
    match self {
      DirectoryEntry::File(f) => f.inner.read().modified_time,
      DirectoryEntry::Directory(d) => d.inner.read().modified_time,
      DirectoryEntry::Symlink(s) => s.inner.read().modified_time,
    }
  }
}

#[derive(Debug)]
struct SymlinkInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
}

#[derive(Debug)]
struct Symlink {
  name: String,
  target: PathBuf,
  inner: RwLock<SymlinkInner>,
}

#[derive(Debug)]
struct DirectoryInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
}

#[derive(Debug)]
struct Directory {
  name: String,
  inner: RwLock<DirectoryInner>,
  entries: Vec<DirectoryEntry>,
}

enum LookupEntry<'a> {
  NotFound(PathBuf),
  Found(PathBuf, &'a DirectoryEntry),
}

enum LookupNoFollowEntry<'a> {
  NotFound(PathBuf),
  Symlink {
    current_path: PathBuf,
    target_path: PathBuf,
    entry: &'a Symlink,
  },
  Found(PathBuf, &'a DirectoryEntry),
}

#[derive(Debug)]
struct InMemorySysInner {
  // Linux/Mac will always have one dir here, but Windows
  // may have multiple per drive.
  system_root: Vec<DirectoryEntry>,
  cwd: PathBuf,
  thread_sleep_enabled: bool,
  random_seed: Option<u64>,
  envs: HashMap<OsString, OsString>,
  time: Option<SystemTime>,
}

impl InMemorySysInner {
  fn to_absolute_path(&self, p: &Path) -> PathBuf {
    if p.is_absolute() {
      normalize_path(p)
    } else {
      normalize_path(&self.cwd.join(p))
    }
  }

  fn time_now(&self) -> SystemTime {
    self.time.unwrap_or_else(SystemTime::now)
  }

  fn lookup_entry<'a>(
    &'a self,
    path: &Path,
  ) -> Result<(PathBuf, &'a DirectoryEntry)> {
    match self.lookup_entry_detail(path)? {
      LookupEntry::Found(path, entry) => Ok((path, entry)),
      LookupEntry::NotFound(_) => Err(Error::new(
        ErrorKind::NotFound,
        format!("Path not found: '{}'", path.display()),
      )),
    }
  }

  fn lookup_entry_detail<'a>(&'a self, path: &Path) -> Result<LookupEntry<'a>> {
    let mut seen_entries = HashSet::new();
    let mut path = Cow::Borrowed(path);

    loop {
      match self.lookup_entry_detail_no_follow(&path)? {
        LookupNoFollowEntry::NotFound(path) => {
          return Ok(LookupEntry::NotFound(path));
        }
        LookupNoFollowEntry::Found(path, entry) => {
          return Ok(LookupEntry::Found(path, entry));
        }
        LookupNoFollowEntry::Symlink {
          current_path,
          target_path,
          ..
        } => {
          if seen_entries.is_empty() {
            // add the original path at this point in order to avoid allocating when we
            // don't have symlinks
            seen_entries.insert(current_path.clone());
          }
          if !seen_entries.insert(target_path.clone()) {
            return Err(Error::new(
              ErrorKind::Other,
              format!("Symlink loop detected resolving '{}'", path.display()),
            ));
          }
          path = Cow::Owned(target_path);
        }
      }
    }
  }

  fn lookup_entry_detail_no_follow<'a>(
    &'a self,
    path: &Path,
  ) -> Result<LookupNoFollowEntry<'a>> {
    let mut final_path = Vec::new();
    let mut comps = path.components().peekable();
    if comps.peek().is_none() {
      return Err(Error::new(ErrorKind::NotFound, "Empty path"));
    }

    let mut entries = &self.system_root;
    while let Some(comp) = comps.next() {
      final_path.push(comp);
      let comp = match comp {
        Component::RootDir => Cow::Borrowed(""),
        Component::Prefix(component) => {
          let component = component.as_os_str().to_string_lossy();
          if let Some(comp) = comps.next() {
            final_path.push(comp);
          }
          component
        }
        component => component.as_os_str().to_string_lossy(),
      };
      let pos = match entries.binary_search_by(|e| e.name().cmp(&comp)) {
        Ok(p) => p,
        Err(_) => {
          return Ok(LookupNoFollowEntry::NotFound(
            final_path.into_iter().chain(comps).collect(),
          ));
        }
      };

      match &entries[pos] {
        DirectoryEntry::Directory(dir) => {
          if comps.peek().is_none() {
            return Ok(LookupNoFollowEntry::Found(
              final_path.into_iter().collect(),
              &entries[pos],
            ));
          } else {
            entries = &dir.entries;
          }
        }
        DirectoryEntry::File(_) => {
          if comps.peek().is_none() {
            return Ok(LookupNoFollowEntry::Found(
              final_path.into_iter().collect(),
              &entries[pos],
            ));
          } else {
            return Err(Error::new(
              ErrorKind::Other,
              "Path leads into a file or symlink",
            ));
          }
        }
        DirectoryEntry::Symlink(symlink) => {
          let current_path = final_path.into_iter().collect::<PathBuf>();
          let target_path = normalize_path(&current_path.join(&symlink.target));
          return Ok(LookupNoFollowEntry::Symlink {
            current_path,
            target_path,
            entry: symlink,
          });
        }
      }
    }

    Ok(LookupNoFollowEntry::NotFound(
      final_path.into_iter().collect(),
    ))
  }

  fn find_directory_mut<'a>(
    &'a mut self,
    path: &Path,
    create_dirs: bool,
  ) -> Result<&'a mut Directory> {
    // ran into a lot of issues with the borrow checker... recommendation was to
    // resolve symlinks first then resolve the path
    let path = match self.lookup_entry_detail(path)? {
      LookupEntry::Found(path, _) => path,
      LookupEntry::NotFound(path) => path,
    };

    let time = self.time_now();
    let mut comps = path.components().peekable();
    if comps.peek().is_none() {
      return Err(Error::new(ErrorKind::NotFound, "Empty path"));
    }

    let mut entries = &mut self.system_root;
    while let Some(comp) = comps.next() {
      let comp = match comp {
        Component::RootDir => Cow::Borrowed(""),
        Component::Prefix(component) => {
          let component = component.as_os_str().to_string_lossy();
          comps.next();
          component
        }
        component => component.as_os_str().to_string_lossy(),
      };
      let pos = match entries.binary_search_by(|e| e.name().cmp(&comp)) {
        Ok(p) => p,
        Err(insert_pos) => {
          if create_dirs {
            let new_dir = Directory {
              name: comp.into_owned(),
              inner: RwLock::new(DirectoryInner {
                created_time: time,
                modified_time: time,
              }),
              entries: vec![],
            };
            entries.insert(insert_pos, DirectoryEntry::Directory(new_dir));
            insert_pos
          } else {
            return Err(Error::new(ErrorKind::NotFound, "Path not found"));
          }
        }
      };

      match &mut entries[pos] {
        DirectoryEntry::Directory(dir) => {
          if comps.peek().is_none() {
            return Ok(dir);
          } else {
            entries = &mut dir.entries;
          }
        }
        DirectoryEntry::File(_) | DirectoryEntry::Symlink { .. } => {
          return Err(Error::new(
            ErrorKind::Other,
            "Path leads into a file or symlink",
          ));
        }
      }
    }

    Err(Error::new(ErrorKind::NotFound, "Path not found"))
  }
}

/// An in-memory system implementation useful for testing.
///
/// This is extremely untested and sloppily implemented. Use with extreme caution
/// and only for testing. You will encounter bugs. Please submit fixes. I implemented
/// this lazily and quickly.
#[derive(Debug, Clone)]
pub struct InMemorySys(Arc<RwLock<InMemorySysInner>>);

impl Default for InMemorySys {
  fn default() -> Self {
    Self(Arc::new(RwLock::new(InMemorySysInner {
      envs: Default::default(),
      system_root: vec![],
      cwd: PathBuf::from("/"),
      thread_sleep_enabled: true,
      random_seed: None,
      time: None,
    })))
  }
}

impl InMemorySys {
  pub fn set_seed(&self, seed: Option<u64>) {
    self.0.write().random_seed = seed;
  }

  pub fn set_time(&self, time: Option<SystemTime>) {
    self.0.write().time = time;
  }

  /// Makes thread sleeping a no-op.
  pub fn disable_thread_sleep(&self) {
    self.0.write().thread_sleep_enabled = false;
  }

  pub fn fs_insert(&self, path: impl AsRef<Path>, data: impl AsRef<[u8]>) {
    self
      .fs_create_dir_all(path.as_ref().parent().unwrap())
      .unwrap();
    self.fs_write(path, data).unwrap();
  }

  /// Helper method for inserting json into the in-memory file system.
  #[cfg(feature = "serde_json")]
  pub fn fs_insert_json(
    &self,
    path: impl AsRef<Path>,
    json: serde_json::Value,
  ) {
    self
      .fs_create_dir_all(path.as_ref().parent().unwrap())
      .unwrap();
    self
      .fs_write(path, serde_json::to_string(&json).unwrap())
      .unwrap();
  }
}

impl EnvCurrentDir for InMemorySys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    Ok(self.0.read().cwd.clone())
  }
}

impl BaseEnvSetCurrentDir for InMemorySys {
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()> {
    let path = self.fs_canonicalize(path)?; // cause an error if not exists
    self.0.write().cwd = path;
    Ok(())
  }
}

impl BaseEnvVar for InMemorySys {
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString> {
    self.0.read().envs.get(key).cloned()
  }
}

impl BaseEnvSetVar for InMemorySys {
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr) {
    self
      .0
      .write()
      .envs
      .insert(key.to_os_string(), value.to_os_string());
  }
}

impl EnvCacheDir for InMemorySys {
  fn env_cache_dir(&self) -> Option<PathBuf> {
    self.env_home_dir().map(|h| h.join(".cache"))
  }
}

impl EnvHomeDir for InMemorySys {
  fn env_home_dir(&self) -> Option<PathBuf> {
    self.env_var("HOME").ok().map(PathBuf::from)
  }
}

impl EnvTempDir for InMemorySys {
  fn env_temp_dir(&self) -> std::io::Result<PathBuf> {
    let inner = self.0.read();
    if let Some(first_dir) = inner.system_root.get(0) {
      let name = first_dir.name();
      let name = if name.is_empty() { "/" } else { name };
      Ok(PathBuf::from(name).join("tmp"))
    } else {
      Err(std::io::Error::new(ErrorKind::Other, "Create a root for the InMemorySys file system before getting the temp dir."))
    }
  }
}

// File System

impl BaseFsCanonicalize for InMemorySys {
  fn base_fs_canonicalize(&self, path: &Path) -> Result<PathBuf> {
    let inner = self.0.read();
    let path = inner.to_absolute_path(path);
    let (path, _) = inner.lookup_entry(&path)?;
    Ok(path)
  }
}

impl BaseFsCreateDirAll for InMemorySys {
  fn base_fs_create_dir_all(&self, path: &Path) -> Result<()> {
    let mut inner = self.0.write();
    let abs = inner.to_absolute_path(path);
    inner.find_directory_mut(&abs, true)?;
    Ok(())
  }
}

impl BaseFsHardLink for InMemorySys {
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> Result<()> {
    let inner = self.0.read();
    let src = inner.to_absolute_path(src.as_ref());
    let dst = inner.to_absolute_path(dst.as_ref());
    let (_, entry) = inner.lookup_entry(&src)?;
    match entry {
      DirectoryEntry::File(file) => {
        let data = {
          let inner = file.inner.read();
          inner.data.clone()
        };
        drop(inner);
        self.fs_write(&dst, data)?;
      }
      DirectoryEntry::Directory(_) | DirectoryEntry::Symlink(_) => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot hard link directories or symlinks",
        ));
      }
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct InMemoryMetadata {
  file_type: FileType,
  modified: SystemTime,
}

impl FsMetadataValue for InMemoryMetadata {
  fn file_type(&self) -> FileType {
    self.file_type
  }

  fn modified(&self) -> Result<SystemTime> {
    Ok(self.modified)
  }
}

impl BaseFsMetadata for InMemorySys {
  type Metadata = InMemoryMetadata;

  fn base_fs_metadata(&self, path: &Path) -> std::io::Result<InMemoryMetadata> {
    let inner = self.0.read();
    let (_, entry) = inner.lookup_entry(path)?;
    Ok(InMemoryMetadata {
      file_type: match entry {
        DirectoryEntry::File(_) => FileType::File,
        DirectoryEntry::Directory(_) => FileType::Dir,
        DirectoryEntry::Symlink(_) => FileType::Symlink,
      },
      modified: entry.modified_time(),
    })
  }

  fn base_fs_symlink_metadata(
    &self,
    path: &Path,
  ) -> std::io::Result<InMemoryMetadata> {
    let inner = self.0.read();
    let detail = inner.lookup_entry_detail_no_follow(path)?;
    match detail {
      LookupNoFollowEntry::NotFound(path) => Err(Error::new(
        ErrorKind::NotFound,
        format!("Path not found: '{}'", path.display()),
      )),
      LookupNoFollowEntry::Symlink { entry, .. } => Ok(InMemoryMetadata {
        file_type: FileType::Symlink,
        modified: entry.inner.read().modified_time,
      }),
      LookupNoFollowEntry::Found(_, entry) => Ok(InMemoryMetadata {
        file_type: match entry {
          DirectoryEntry::File(_) => FileType::File,
          DirectoryEntry::Directory(_) => FileType::Dir,
          DirectoryEntry::Symlink(_) => FileType::Symlink,
        },
        modified: entry.modified_time(),
      }),
    }
  }
}

impl BaseFsOpen for InMemorySys {
  type File = InMemoryFile;

  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<InMemoryFile> {
    let mut inner = self.0.write();
    let time_now = inner.time_now();
    let path = inner.to_absolute_path(path);

    // Edge case: If `parent()` is None, path might be root or invalid
    // The minimal fix is to check for that scenario
    let parent_path = match path.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot open root or invalid path",
        ));
      }
    };

    let parent = inner.find_directory_mut(parent_path, false)?;
    let file_name = match path.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No file name found"));
      }
    };

    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(pos) => match &mut parent.entries[pos] {
        DirectoryEntry::File(f) => {
          if options.create_new {
            return Err(Error::new(
              ErrorKind::AlreadyExists,
              "File already exists (create_new=true)",
            ));
          }
          if options.truncate {
            let mut fi = f.inner.write();
            fi.data.clear();
            fi.modified_time = time_now;
          }
          Ok(InMemoryFile {
            sys: self.clone(),
            inner: f.inner.clone(),
            pos: if options.append {
              f.inner.read().data.len()
            } else {
              0
            },
          })
        }
        _ => Err(Error::new(ErrorKind::Other, "Path is not a file")),
      },
      Err(insert_pos) => {
        if !options.create {
          return Err(Error::new(ErrorKind::NotFound, "File not found"));
        }
        let new_file = File {
          name: file_name.into_owned(),
          inner: Arc::new(RwLock::new(FileInner {
            created_time: time_now,
            modified_time: time_now,
            data: vec![],
            mode: options.mode.unwrap_or(0o666),
          })),
        };
        let result = InMemoryFile {
          sys: self.clone(),
          inner: new_file.inner.clone(),
          pos: if options.append {
            new_file.inner.read().data.len()
          } else {
            0
          },
        };
        parent
          .entries
          .insert(insert_pos, DirectoryEntry::File(new_file));
        Ok(result)
      }
    }
  }
}

impl BaseFsRead for InMemorySys {
  fn base_fs_read(&self, path: &Path) -> std::io::Result<Cow<'static, [u8]>> {
    let arc_file = self.fs_open(path, &OpenOptions::read())?;
    let inner = arc_file.inner.read();
    Ok(Cow::Owned(inner.data.clone()))
  }
}

impl BaseFsReadDir for InMemorySys {
  type ReadDirEntry = InMemoryDirEntry;

  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    let inner = self.0.read();
    let abs_path = inner.to_absolute_path(path);

    let (_, entry) = inner.lookup_entry(&abs_path)?;
    match entry {
      DirectoryEntry::Directory(dir) => Ok(Box::new(
        dir
          .entries
          .iter()
          .map(|entry| Ok(InMemoryDirEntry::new(path, entry)))
          .collect::<Vec<_>>()
          .into_iter(),
      )),
      _ => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Path is not a directory",
      )),
    }
  }
}

#[derive(Debug)]
pub struct InMemoryDirEntry {
  name: String,
  path: PathBuf,
  file_type: FileType,
  modified: SystemTime,
}

impl InMemoryDirEntry {
  fn new(initial_path: &Path, entry: &DirectoryEntry) -> Self {
    Self {
      name: entry.name().to_string(),
      path: initial_path.join(entry.name()),
      file_type: match entry {
        DirectoryEntry::File(_) => FileType::File,
        DirectoryEntry::Directory(_) => FileType::Dir,
        DirectoryEntry::Symlink(_) => FileType::Symlink,
      },
      modified: entry.modified_time(),
    }
  }
}

impl FsDirEntry for InMemoryDirEntry {
  type Metadata = InMemoryMetadata;

  fn file_name(&self) -> std::borrow::Cow<std::ffi::OsStr> {
    std::borrow::Cow::Owned(self.name.clone().into())
  }

  fn file_type(&self) -> std::io::Result<FileType> {
    Ok(self.file_type)
  }

  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    Ok(InMemoryMetadata {
      file_type: self.file_type,
      modified: self.modified,
    })
  }

  fn path(&self) -> std::borrow::Cow<std::path::Path> {
    std::borrow::Cow::Borrowed(self.path.as_ref())
  }
}

impl BaseFsRemoveFile for InMemorySys {
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let path = inner.to_absolute_path(path);
    let parent_path = match path.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot remove root or invalid path",
        ));
      }
    };
    let parent = inner.find_directory_mut(parent_path, false)?;
    let file_name = match path.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No file name found"));
      }
    };

    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(pos) => match &parent.entries[pos] {
        DirectoryEntry::File(_) => {
          parent.entries.remove(pos);
          Ok(())
        }
        _ => Err(Error::new(ErrorKind::Other, "Not a file")),
      },
      Err(_) => Err(Error::new(ErrorKind::NotFound, "File not found")),
    }
  }
}

impl BaseFsRename for InMemorySys {
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let from = inner.to_absolute_path(from.as_ref());
    let to = inner.to_absolute_path(to.as_ref());

    let from_parent_path = match from.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename root or invalid path",
        ));
      }
    };
    let from_file_name = match from.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No source file name found"));
      }
    };

    let from_parent = inner.find_directory_mut(from_parent_path, false)?;
    let from_idx = match from_parent
      .entries
      .binary_search_by(|e| e.name().cmp(&from_file_name))
    {
      Ok(pos) => pos,
      Err(_) => {
        return Err(Error::new(ErrorKind::NotFound, "Source file not found"));
      }
    };
    let file_entry = from_parent.entries.remove(from_idx);

    let to_parent_path = match to.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        // If `to` has no valid parent, restore the original file entry:
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename to root or invalid path",
        ));
      }
    };
    let to_file_name = match to.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        // restore
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "No destination file name found",
        ));
      }
    };

    let to_parent = inner.find_directory_mut(to_parent_path, true)?;
    match file_entry {
      DirectoryEntry::File(mut f) => {
        match to_parent
          .entries
          .binary_search_by(|e| e.name().cmp(&to_file_name))
        {
          Ok(pos) => match &to_parent.entries[pos] {
            DirectoryEntry::Directory(_) => {
              let from_parent =
                inner.find_directory_mut(from_parent_path, false)?;
              from_parent
                .entries
                .insert(from_idx, DirectoryEntry::File(f));
              return Err(Error::new(
                ErrorKind::Other,
                "Cannot rename to a directory",
              ));
            }
            _ => {
              f.name = to_file_name.to_string();
              to_parent.entries[pos] = DirectoryEntry::File(f);
            }
          },
          Err(insert_pos) => {
            f.name = to_file_name.to_string();
            to_parent
              .entries
              .insert(insert_pos, DirectoryEntry::File(f));
          }
        }
      }
      _ => {
        let from_parent = inner.find_directory_mut(from_parent_path, false)?;
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename directories or symlinks here",
        ));
      }
    }
    Ok(())
  }
}

impl BaseFsSymlinkDir for InMemorySys {
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    self.base_fs_symlink_file(original, link)
  }
}

impl BaseFsSymlinkFile for InMemorySys {
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let time = inner.time_now();
    let link = inner.to_absolute_path(link.as_ref());
    let parent = inner.find_directory_mut(link.parent().unwrap(), false)?;
    let file_name = link.file_name().unwrap().to_string_lossy();
    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(overwrite_pos) => {
        match &parent.entries[overwrite_pos] {
          DirectoryEntry::Directory(directory) => {
            return Err(Error::new(
              ErrorKind::AlreadyExists,
              format!("Directory already exists: '{}'", directory.name),
            ));
          }
          DirectoryEntry::File(_) | DirectoryEntry::Symlink(_) => {
            // do nothing
          }
        }

        parent.entries[overwrite_pos] = DirectoryEntry::Symlink(Symlink {
          name: file_name.into_owned(),
          target: original.to_path_buf(),
          inner: RwLock::new(SymlinkInner {
            created_time: time,
            modified_time: time,
          }),
        });
        Ok(())
      }
      Err(insert_index) => {
        parent.entries.insert(
          insert_index,
          DirectoryEntry::Symlink(Symlink {
            name: file_name.into_owned(),
            target: original.to_path_buf(),
            inner: RwLock::new(SymlinkInner {
              created_time: time,
              modified_time: time,
            }),
          }),
        );
        Ok(())
      }
    }
  }
}

impl BaseFsWrite for InMemorySys {
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    let opts = OpenOptions {
      write: true,
      create: true,
      truncate: true,
      append: false,
      read: false,
      create_new: false,
      mode: None,
    };
    let time_now = self.sys_time_now();
    let file = self.fs_open(path, &opts)?;
    let mut inner = file.inner.write();
    inner.data.clear();
    inner.data.extend_from_slice(data.as_ref());
    inner.modified_time = time_now;
    Ok(())
  }
}

// File System File

impl FsFileSetPermissions for InMemoryFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    let mut inner = self.inner.write();
    inner.mode = mode;
    Ok(())
  }
}

impl std::io::Seek for InMemoryFile {
  fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64> {
    match pos {
      std::io::SeekFrom::Start(n) => {
        self.pos = n as usize;
      }
      std::io::SeekFrom::End(n) => {
        let inner = self.inner.read();
        if -n > inner.data.len() as i64 {
          return Err(Error::new(
            ErrorKind::InvalidInput,
            "Seeking before start of file",
          ));
        }
        self.pos = (inner.data.len() as i64 + n) as usize;
      }
      std::io::SeekFrom::Current(n) => {
        self.pos = self.pos.wrapping_add(n as usize);
      }
    }
    Ok(self.pos as u64)
  }
}

impl std::io::Write for InMemoryFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let time = self.sys.sys_time_now();
    let mut inner = self.inner.write();
    if self.pos > inner.data.len() {
      inner.data.resize(self.pos, 0);
    }
    inner.data.splice(self.pos.., buf.as_ref().iter().cloned());
    inner.modified_time = time;
    self.pos += buf.as_ref().len();
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl std::io::Read for InMemoryFile {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let inner = self.inner.read();
    if self.pos > inner.data.len() {
      return Ok(0);
    }
    let data = &inner.data[self.pos..];
    let len = std::cmp::min(data.len(), buf.len());
    buf[..len].copy_from_slice(&data[..len]);
    self.pos += len;
    Ok(len)
  }
}

// System

impl SystemTimeNow for InMemorySys {
  fn sys_time_now(&self) -> SystemTime {
    self.0.read().time_now()
  }
}

impl SystemRandom for InMemorySys {
  fn sys_random(&self, buf: &mut [u8]) -> std::io::Result<()> {
    fn random_with_seed(seed: u64, buf: &mut [u8]) {
      // not the best, but good enough for now
      let mut state = seed;
      for byte in buf.iter_mut() {
        // simple linear congruential generator
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        *byte = (state >> 24) as u8; // use the top 8 bits
      }
    }

    match self.0.read().random_seed {
      Some(seed) => {
        random_with_seed(seed, buf);
        Ok(())
      }
      None => {
        #[cfg(feature = "getrandom")]
        {
          getrandom::getrandom(buf)
            .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
        }
        #[cfg(not(feature = "getrandom"))]
        {
          random_with_seed(0, buf);
          Ok(())
        }
      }
    }
  }
}

impl ThreadSleep for InMemorySys {
  fn thread_sleep(&self, dur: std::time::Duration) {
    if self.0.read().thread_sleep_enabled {
      std::thread::sleep(dur);
    }
  }
}

/// Normalize all intermediate components of the path (ie. remove "./" and "../" components).
/// Similar to `fs::canonicalize()` but doesn't resolve symlinks.
///
/// Taken from Cargo
/// <https://github.com/rust-lang/cargo/blob/af307a38c20a753ec60f0ad18be5abed3db3c9ac/src/cargo/util/paths.rs#L60-L85>
#[inline]
fn normalize_path(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret =
    if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
      components.next();
      PathBuf::from(c.as_os_str())
    } else {
      PathBuf::new()
    };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        ret.push(c);
      }
    }
  }
  ret
}

// most of these tests were lazily created with ChatGPT
#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Seek;
  use std::io::Write;
  use std::path::Path;
  use std::time::Duration;
  use std::time::SystemTime;

  #[test]
  fn test_env_vars() {
    let sys = InMemorySys::default();
    sys.env_set_var("VALUE", "other");
    assert_eq!(sys.env_var_os("VALUE"), Some("other".into()));
  }

  #[test]
  fn test_create_dir_all() {
    let sys = InMemorySys::default();
    let dir_path = Path::new("/rootDir/subDir");
    assert!(!sys.fs_exists(dir_path).unwrap());
    sys.fs_create_dir_all(dir_path).unwrap();
    assert!(sys.fs_exists(dir_path).unwrap());
    assert!(sys.fs_is_dir(dir_path).unwrap());
  }

  #[test]
  fn test_write_read_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/test.txt";
    sys.fs_write(file_path, b"Hello World!").unwrap();
    assert!(sys.fs_exists(file_path).unwrap());
    assert!(sys.fs_is_file(file_path).unwrap());

    let contents = sys.fs_read_to_string(file_path).unwrap();
    assert_eq!(&*contents, "Hello World!");
  }

  #[test]
  fn test_truncate_existing_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/data.bin";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let opts = OpenOptions {
      write: true,
      truncate: true,
      ..Default::default()
    };
    let file = sys.fs_open(file_path, &opts).unwrap();
    // file is truncated at open, so should be empty
    let guard = file.inner.read();
    assert!(guard.data.is_empty());
  }

  #[test]
  fn test_rename_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let old_path = "/rootDir/old.txt";
    let new_path = "/rootDir/new.txt";
    sys.fs_write(old_path, b"some data").unwrap();

    sys.fs_rename(old_path, new_path).unwrap();
    assert!(!sys.fs_exists(old_path).unwrap());
    assert!(sys.fs_exists(new_path).unwrap());

    let data = sys.fs_read_to_string(new_path).unwrap();
    assert_eq!(&*data, "some data");
  }

  #[test]
  fn test_remove_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/remove_me.txt";
    sys.fs_write(file_path, b"Bye!").unwrap();
    assert!(sys.fs_exists(file_path).unwrap());

    sys.fs_remove_file(file_path).unwrap();
    assert!(!sys.fs_exists(file_path).unwrap());
  }

  #[test]
  fn test_modified_time() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/hello.txt";
    sys.fs_write(file_path, b"Hi!").unwrap();

    // First check if we can get a valid modified time
    let modified = sys.fs_metadata(file_path).unwrap().modified;

    // Since we can't easily freeze or manipulate real time,
    // we'll just assert it's no earlier than the current system time minus some buffer.
    let now = SystemTime::now();
    let duration = now.duration_since(modified);
    assert!(duration.is_ok());
  }

  #[test]
  fn test_exists_no_err() {
    let sys = InMemorySys::default();
    assert!(!sys.fs_exists_no_err("/does/not/exist"));
    sys.fs_create_dir_all("/exists").unwrap();
    assert!(sys.fs_exists_no_err("/exists"));
  }

  #[test]
  fn test_is_file_no_err() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"contents").unwrap();
    assert!(!sys.fs_is_file_no_err("/no/file"));
    assert!(!sys.fs_is_file_no_err("/dir"));
    assert!(sys.fs_is_file_no_err("/dir/file.txt"));
  }

  #[test]
  fn test_is_dir_no_err() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"contents").unwrap();
    assert!(!sys.fs_is_dir_no_err("/no/dir"));
    assert!(sys.fs_is_dir_no_err("/dir"));
    assert!(!sys.fs_is_dir_no_err("/dir/file.txt"));
  }

  #[test]
  fn test_file_permissions() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();

    let file_path = "/dir/perm_test.txt";
    sys.fs_write(file_path, b"Testing perms").unwrap();
    let mut file = sys.fs_open(file_path, &OpenOptions::read()).unwrap();
    file.fs_file_set_permissions(0o755).unwrap();

    let guard = file.inner.read();
    assert_eq!(guard.mode, 0o755);
  }

  #[test]
  fn test_file_append() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();

    let file_path = "/dir/append_test.txt";
    let mut opts = OpenOptions {
      write: true,
      create: true,
      ..Default::default()
    };
    // Not truncate
    sys.fs_open(file_path, &opts).unwrap(); // creates empty file
                                            // Now open again with append
    opts.append = true;
    let mut file = sys.fs_open(file_path, &opts).unwrap();
    // Should start at position 0 in the code, but let's test manually
    _ = file.write(b"Appending ").unwrap();
    _ = file.write(b"more data").unwrap();

    let contents = sys.fs_read_to_string(file_path).unwrap();
    assert_eq!(&*contents, "Appending more data");
  }

  #[test]
  fn test_create_dir_that_already_exists() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/already/exists").unwrap();
    let result = sys.fs_create_dir_all("/already/exists");
    assert!(
      result.is_ok(),
      "Creating a directory that already exists should succeed"
    );
  }

  #[test]
  fn test_remove_non_empty_directory_fails() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"data").unwrap();
    let result = sys.fs_remove_file("/dir");
    assert!(
            result.is_err(),
            "Removing a non-empty directory (treated as a directory, not a file) should fail"
        );
  }

  #[test]
  fn test_fs_canonicalize_relative_path() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test/relative").unwrap();
    {
      let mut inner = sys.0.write();
      inner.cwd = PathBuf::from("/test");
    }
    let abs = sys.fs_canonicalize("relative").unwrap();
    assert_eq!(abs, PathBuf::from("/test/relative"));
  }

  #[test]
  fn test_fs_canonicalize_absolute_path() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/absolute").unwrap();
    let abs = sys.fs_canonicalize("/absolute").unwrap();
    assert_eq!(abs, PathBuf::from("/absolute"));
  }

  #[test]
  fn test_sys_random_no_seed() {
    let sys = InMemorySys::default();
    let mut buf1 = [0u8; 8];
    let mut buf2 = [0u8; 8];
    sys.sys_random(&mut buf1).unwrap();
    sys.sys_random(&mut buf2).unwrap();
    // There's no guarantee on exact values without a seed, but it should succeed
    assert_ne!(buf1, [0u8; 8]);
    assert_ne!(buf2, [0u8; 8]);
  }

  #[test]
  fn test_thread_sleep_no_op() {
    let sys = InMemorySys::default();
    sys.disable_thread_sleep();
    let start = SystemTime::now();
    sys.thread_sleep(Duration::from_secs(1));
    // Should be effectively no-op, so the elapsed time should be tiny
    let elapsed = start.elapsed().unwrap();
    assert!(
      elapsed < Duration::from_millis(100),
      "Sleep should be disabled"
    );
  }

  #[test]
  fn test_rename_file_to_existing_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file1.txt", b"111").unwrap();
    sys.fs_write("/dir/file2.txt", b"222").unwrap();
    let result = sys.fs_rename("/dir/file1.txt", "/dir/file2.txt");
    assert!(result.is_ok() || result.is_err());
    let file1_exists = sys.fs_exists_no_err("/dir/file1.txt");
    let file2_exists = sys.fs_exists_no_err("/dir/file2.txt");
    assert!(!file1_exists && file2_exists);
  }

  #[test]
  fn test_fs_write_into_non_existent_subdir_fails() {
    let sys = InMemorySys::default();
    let result = sys.fs_write("/no-such-subdir/myfile.txt", b"content");
    assert!(
      result.is_err(),
      "Should fail because /no-such-subdir does not exist"
    );
  }

  #[test]
  fn test_fs_read_dir_with_files() {
    let sys = InMemorySys::default();
    let root_dir = "/test";

    // Setup directories and files
    sys.fs_create_dir_all(root_dir).unwrap();
    sys
      .fs_write(format!("{}/file1.txt", root_dir), b"Content 1")
      .unwrap();
    sys
      .fs_write(format!("{}/file2.txt", root_dir), b"Content 2")
      .unwrap();

    // Read directory
    let entries: Vec<_> = sys
      .fs_read_dir(root_dir)
      .unwrap()
      .map(|res| res.unwrap().file_name().to_string_lossy().to_string())
      .collect();

    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&"file1.txt".to_string()));
    assert!(entries.contains(&"file2.txt".to_string()));
  }

  #[test]
  fn test_fs_read_dir_with_subdirectories() {
    let sys = InMemorySys::default();
    let root_dir = "/test";

    // Setup directories and files
    sys
      .fs_create_dir_all(format!("{}/subdir", root_dir))
      .unwrap();
    sys
      .fs_write(format!("{}/subdir/file.txt", root_dir), b"Content")
      .unwrap();

    // Read root directory
    let entries: Vec<_> = sys
      .fs_read_dir(root_dir)
      .unwrap()
      .map(|res| res.unwrap().file_name().to_string_lossy().to_string())
      .collect();

    assert_eq!(entries.len(), 1);
    assert!(entries.contains(&"subdir".to_string()));
  }

  #[test]
  fn test_fs_read_dir_not_a_directory() {
    let sys = InMemorySys::default();
    let file_path = "/file.txt";

    // Create a file
    sys.fs_create_dir_all("/").unwrap();
    sys.fs_write(file_path, b"Content").unwrap();

    // Attempt to read as directory
    let result = sys.fs_read_dir(file_path);
    assert!(result.is_err());
    match result {
      Err(err) => {
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
      }
      _ => panic!("Expected an error"),
    }
  }

  #[test]
  fn test_fs_read_dir_empty_directory() {
    let sys = InMemorySys::default();
    let empty_dir = "/empty";
    sys.fs_create_dir_all(empty_dir).unwrap();

    let entries: Vec<_> = sys
      .fs_read_dir(empty_dir)
      .unwrap()
      .map(|res| res.unwrap().file_name().to_string_lossy().to_string())
      .collect();

    assert!(entries.is_empty());
  }

  #[test]
  fn test_hard_link_sync() {
    let sys = InMemorySys::default();
    let empty_dir = "/empty";
    sys.fs_create_dir_all(empty_dir).unwrap();

    sys.fs_write("/empty/file.txt", b"Content").unwrap();
    sys
      .fs_hard_link("/empty/file.txt", "/empty/file2.txt")
      .unwrap();
    assert_eq!(
      sys.fs_read("/empty/file2.txt").unwrap().as_ref(),
      b"Content"
    );
  }

  #[test]
  fn test_seek_start() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test").unwrap();
    let file_path = "/test/seek.txt";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let mut file = sys.fs_open(file_path, &OpenOptions::write()).unwrap();

    // Seek to the start of the file
    let new_pos = file.seek(std::io::SeekFrom::Start(0)).unwrap();
    assert_eq!(new_pos, 0);
    assert_eq!(file.pos, 0);
  }

  #[test]
  fn test_seek_end() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test").unwrap();
    let file_path = "/test/seek.txt";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let mut file = sys.fs_open(file_path, &OpenOptions::read()).unwrap();

    // Seek to the end of the file
    let new_pos = file.seek(std::io::SeekFrom::End(0)).unwrap();
    assert_eq!(new_pos, 6);
    assert_eq!(file.pos, 6);

    // Seek 2 bytes before the end
    let new_pos = file.seek(std::io::SeekFrom::End(-2)).unwrap();
    assert_eq!(new_pos, 4);
    assert_eq!(file.pos, 4);
  }

  #[test]
  fn test_seek_current() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test").unwrap();
    let file_path = "/test/seek.txt";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let mut file = sys.fs_open(file_path, &OpenOptions::write()).unwrap();

    // Seek 2 bytes forward from the start
    let new_pos = file.seek(std::io::SeekFrom::Current(2)).unwrap();
    assert_eq!(new_pos, 2);
    assert_eq!(file.pos, 2);

    // Seek 1 byte backward from the current position
    let new_pos = file.seek(std::io::SeekFrom::Current(-1)).unwrap();
    assert_eq!(new_pos, 1);
    assert_eq!(file.pos, 1);
  }

  #[test]
  fn test_seek_before_start_fails() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test").unwrap();
    let file_path = "/test/seek.txt";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let mut file = sys.fs_open(file_path, &OpenOptions::write()).unwrap();

    // Attempt to seek before the start of the file
    let result = file.seek(std::io::SeekFrom::End(-1000));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
  }

  #[test]
  fn test_seek_write_position() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test").unwrap();
    let file_path = "/test/seek_write.txt";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let mut file = sys
      .fs_open(
        file_path,
        &OpenOptions {
          truncate: false,
          write: false,
          ..Default::default()
        },
      )
      .unwrap();

    // Seek to position 3 and write data
    file.seek(std::io::SeekFrom::Start(3)).unwrap();
    file.write_all(b"XYZ").unwrap();
    // Seek then write past the end
    file.seek(std::io::SeekFrom::End(2)).unwrap();
    file.write_all(b"a").unwrap();

    let contents = sys.fs_read_to_string(file_path).unwrap();
    assert_eq!(&*contents, "abcXYZ\0\0a");
  }

  #[test]
  fn test_temp_dir() {
    let sys = InMemorySys::default();
    assert!(sys.env_temp_dir().is_err());
    sys.fs_create_dir_all("/test").unwrap();
    assert_eq!(sys.env_temp_dir().unwrap(), PathBuf::from("/tmp"));
  }
}
