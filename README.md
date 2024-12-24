# `sys_traits`

WARNING: Extremely experimental and mostly untested... trying to get the high
level design right first.

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
