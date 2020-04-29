pub struct Node {
  pub name: String,

  pub is_active: bool,
  pub is_visible: bool,

  children: Vec<Node>,
}

impl Node {
  pub fn update(&mut self) {
    if self.is_active {
      self.children.iter_mut().for_each(|n| n.update());
    }
  }

  pub fn render(&self) {
    if self.is_active {
      self.children.iter().for_each(|n| n.render());
    }
  }
}
