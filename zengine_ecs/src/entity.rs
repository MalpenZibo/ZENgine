use std::ops::Deref;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(usize);
impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug)]
pub struct EntityGenerator {
    current: usize,
}
impl EntityGenerator {
    pub fn generate(&mut self) -> Entity {
        let entity = Entity(self.current);
        self.current += 1;

        entity
    }
}

#[cfg(test)]
mod tests {
    use super::EntityGenerator;

    #[test]
    fn generate_an_entity() {
        let mut generator = EntityGenerator::default();

        let entities = vec![
            generator.generate(),
            generator.generate(),
            generator.generate(),
            generator.generate(),
        ];

        assert_eq!(
            entities.into_iter().map(|e| e.0).collect::<Vec<usize>>(),
            vec!(0, 1, 2, 3)
        );
        assert_eq!(generator.current, 4);
    }
}
