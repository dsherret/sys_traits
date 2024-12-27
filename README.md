# `sys_traits`

Trait per function for system related functionality.

Write functions that specify only the system functions they need.

```rs
use sys_traits::FsWriteFile;
use sys_traits::SystemRandom;

pub fn write_random_data<TSys: FsWriteFile + SystemRandom>(
  sys: &TSys,
  file_path: &Path,
) -> std::io::Result<()> {
  let mut buf = [0u8; 16];
  sys.sys_random(&mut buf)?;
  sys.fs_write_file(file_path, buf)
}
```

Now a caller only needs to provide a type that implements those two functions.

## Implementations

Comes with two implementations that implement all the traits.

- `sys_traits::impl::RealSys` - A real implementation of the current system.
  - Automatically works with Wasm in Deno
  - Will implement Node.js support once I need it
    (https://github.com/dsherret/sys_traits/issues/4)
- `sys_traits::impl::InMemorySys` - An in-memory system useful for testing.

## Creating an implementation

To create an implementation you must implement the traits; however, some traits
require implementing `<TraitName>Impl` traits instead. For example, instead of
implementing `FsWrite`, you must implement `BaseFsWrite`:

```rs
pub struct MyCustomFileSystem;

impl sys_traits::BaseFsWrite for MyCustomFileSystem {
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    // ...
  }
}
```

The `sys_traits::FsWrite` trait gets automatically implemented for this as its
definition is:

```rs
pub trait FsWrite: BaseFsWrite {
  #[inline]
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    self.base_fs_write(path.as_ref(), data.as_ref())
  }
}

impl<T: BaseFsWrite> FsWrite for T {}
```

There's two reasons for this:

1. You can't box traits with `impl ...`.
2. By design it limits code generation of multiple kinds of `impl AsRef<Path>`
   and `impl AsRef<[u8]>` to only being a single statement.
