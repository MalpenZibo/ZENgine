use std::ops::Deref;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(usize);
impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct EntityGenerator {
    current: usize,
}
impl EntityGenerator {
    pub fn generate(&mut self) -> Entity {
        let entity = Entity(self.current);
        self.current += 1;

        entity
    }

    pub fn valid_entity(&self, entity: Entity) -> bool {
        entity.0 < self.current
    }
}
