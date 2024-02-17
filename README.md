# display-info

Cross-platform get display info for MacOS、Windows、Linux. Like [electron Display Object](https://www.electronjs.org/docs/latest/api/structures/display)

## Example

```rust
use display_info::DisplayInfo;
use std::time::Instant;

fn main() {
  let start = Instant::now();

  let display_infos = DisplayInfo::all().unwrap();
  for display_info in display_infos {
    println!("display_info {display_info:?}");
  }
  let display_info = DisplayInfo::from_point(100, 100).unwrap();
  println!("display_info {display_info:?}");
  println!("运行耗时: {:?}", start.elapsed());
}
```

## DisplayInfo struct

-   `id` u32 - Unique identifier associated with the display.
-   `name` String - The name of the display
-   `raw_handle` CGDisplay/HMONITOR/Output - Native display raw handle
-   `x` i32 - The display x coordinate.
-   `y` i32 - The display y coordinate.
-   `width` u32 - The display pixel width.
-   `height` u32 - The display pixel height.
-   `rotation` f32 - Can be 0, 90, 180, 270, represents screen rotation in clock-wise degrees.
-   `scale_factor` f32 - Output device's pixel scale factor.
-   `frequency` f32 - The display refresh rate.
-   `is_primary` bool - Whether the screen is the main screen

## Linux requirements

On Linux, you need to install `libxcb`、`libxrandr`

Debian/Ubuntu:

```sh
apt-get install libxcb1 libxrandr2
```

Alpine:

```sh
apk add libxcb libxrandr
```

ArchLinux:

```sh
pacman -S libxcb libxrandr
```
