#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
  pub id: u32,
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub scale: f32,
  pub rotation: f32,
}
