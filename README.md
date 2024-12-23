# `sys_traits`

Trait per function for system related functionality.

Write functions that specify only the system functionality they need.

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

Comes with a `sys_traits::imp::RealSys` implementation that implements all the traits.
