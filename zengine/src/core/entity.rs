#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct EntityId(pub u32);

#[derive(Debug)]
pub struct Entity {
  pub id: EntityId,
}
