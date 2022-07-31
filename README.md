# display-info

Cross-platform get display info for MacOS、Windows、Linux. Like [electron Display Object](https://www.electronjs.org/docs/latest/api/structures/display)

## example

```rust
use display_info::DisplayInfo;
use std::time::Instant;

fn main() {
  let start = Instant::now();

  let display_infos = DisplayInfo::all().unwrap();
  for display_info in display_infos {
    println!("display_info {:?}", display_info);
  }
  let display_info = DisplayInfo::from_point(100, 100).unwrap();
  println!("display_info {:?}", display_info);
  println!("运行耗时: {:?}", start.elapsed());
}
```

## Linux requirements

On Linux, you need to install `libxcb`、`libxrandr`

Debian/Ubuntu:

```sh
apt-get install libxcb1 libxrandr2
```

Alpine:

```sh
apk add libxcb1 libxrandr2
```
