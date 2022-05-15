# display-info

Cross-platform get display info for MacOS、Windows、Linux. Like [electron Display Object](https://www.electronjs.org/docs/latest/api/structures/display)

## example

```rust
use display_info::DisplayInfo;

fn main() {
  let display_infos = DisplayInfo::all();
  for display_info in display_infos {
    println!(
      "DisplayInfo:{} x: {} y: {} width: {} height: {} scale: {} rotation: {}\n",
      display_info.id,
      display_info.x,
      display_info.y,
      display_info.width,
      display_info.height,
      display_info.scale,
      display_info.rotation
    );
  }
  let display_info = DisplayInfo::from_point(100, 100).unwrap();
  println!("display_info {:?}", display_info);
  println!(
    "DisplayInfo:{} x: {} y: {} width: {} height: {} scale: {} rotation: {}\n",
    display_info.id,
    display_info.x,
    display_info.y,
    display_info.width,
    display_info.height,
    display_info.scale,
    display_info.rotation
  );
}
```
