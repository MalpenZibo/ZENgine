use crate::World;


pub trait System: Send + Sync {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World);

    fn apply(&mut self, world: &mut World);
}

#[cfg(test)]
mod tests {
    use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

    use super::System;

    #[test]
    fn test_system() {
        let mut systems: Vec<Box<dyn System>> = vec![];

        let world = Default::default();

        systems.par_iter_mut().for_each(|s| {
            s.run(&world);
        });
    }
}
