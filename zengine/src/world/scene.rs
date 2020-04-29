use serde::Deserialize;

use crate::graphics::color::Color;
use crate::world::node::Node;

#[derive(Deserialize)]
pub struct Scene {
  pub name: String,

  #[serde(default)]
  pub background_color: Color,

  root: Node,
}

impl Scene {
  pub fn update(&mut self) {
    self.root.update();
  }

  pub fn render(&self) {
    self.root.render();
  }
}
