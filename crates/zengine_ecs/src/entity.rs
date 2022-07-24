use std::{
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(usize);
impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug)]
pub(crate) struct EntityGenerator {
    current: AtomicUsize,
}
impl EntityGenerator {
    pub fn generate(&self) -> Entity {
        let current = self.current.fetch_add(1, Ordering::Relaxed);
        Entity(current)
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
        assert_eq!(*generator.current.get_mut(), 4);
    }
}
