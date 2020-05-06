#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct EntityId(pub u32);

#[derive(Debug, Eq, PartialEq)]
pub struct Entity {
  pub id: EntityId,
}
