use crate::world::node::Node;

pub struct Scene {
  pub name: String,
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
