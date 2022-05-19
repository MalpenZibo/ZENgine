use crate::{component::ComponentColumn, entity::Entity};
use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

pub type ArchetypeId = u64;
pub type ArchetypeSpecs = Vec<TypeId>;

pub(crate) fn calculate_archetype_id(types: &[TypeId]) -> ArchetypeId {
    let mut s = rustc_hash::FxHasher::default();
    types.hash(&mut s);
    s.finish()
}

#[derive(Debug)]
pub struct Archetype {
    pub(crate) archetype_specs: ArchetypeSpecs,
    pub(crate) entities: Vec<Entity>,
    pub(crate) components: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    pub fn root() -> Self {
        Archetype {
            archetype_specs: vec![],
            entities: Vec::default(),
            components: Vec::default(),
        }
    }

    pub fn new_from_component(
        archetype_specs: ArchetypeSpecs,
        from_components: Vec<Box<dyn ComponentColumn>>,
    ) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(from_components.len()),
        };

        for c in from_components.into_iter() {
            archetype.components.push(c);
        }

        archetype
    }

    /// Removes the component from an entity and pushes it to the other archetype
    /// The type does not need to be known to call this function.
    /// But the types of component_index and other_index need to match.
    pub fn migrate_component(
        &mut self,
        component_index: usize,
        entity_row: usize,
        other_archetype: &mut Archetype,
        other_index: usize,
    ) {
        self.components[component_index]
            .migrate(entity_row, &mut *other_archetype.components[other_index]);
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::{
        component::{component_vec_to_mut, Component, ComponentBundle},
        entity::EntityGenerator,
    };

    use super::Archetype;

    #[derive(Debug, PartialEq)]
    struct Component1 {}
    impl Component for Component1 {}

    #[derive(Debug)]
    struct Component2 {}
    impl Component for Component2 {}

    #[test]
    fn new_from_component() {
        let archetype = Archetype::new_from_component(
            Component1::get_types(),
            Component1::get_component_columns(),
        );

        assert_eq!(archetype.components.len(), 1);
        assert_eq!(archetype.archetype_specs, vec!(TypeId::of::<Component1>()))
    }

    #[test]
    fn migrate_component() {
        let mut generator = EntityGenerator::default();
        let entity = generator.generate();

        let component1 = Component1 {};

        let mut archetype1 = Archetype::new_from_component(
            Component1::get_types(),
            Component1::get_component_columns(),
        );

        let mut archetype2 = Archetype::new_from_component(
            <(Component1, Component2)>::get_types(),
            <(Component1, Component2)>::get_component_columns(),
        );

        archetype1.entities.push(entity);
        Component1::inser_into(&mut archetype1, component1, vec![0]);

        archetype1.migrate_component(0, 0, &mut archetype2, 0);
        archetype1.entities.remove(0);
        archetype2.entities.push(entity);

        assert_eq!(archetype1.entities.len(), 0);
        assert_eq!(archetype2.entities.len(), 1);

        let column = component_vec_to_mut::<Component1>(&mut *archetype2.components[0]);
        let component: &Component1 = column.get(0).unwrap();
        assert_eq!(component, &Component1 {})
    }
}
