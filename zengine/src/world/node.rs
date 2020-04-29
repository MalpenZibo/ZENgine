use serde::Deserialize;

fn boolean_default_true() -> bool {
  true
}

#[derive(Deserialize)]
pub struct Node {
  pub name: String,

  #[serde(default = "boolean_default_true")]
  pub is_active: bool,

  #[serde(default = "boolean_default_true")]
  pub is_visible: bool,

  #[serde(default)]
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
