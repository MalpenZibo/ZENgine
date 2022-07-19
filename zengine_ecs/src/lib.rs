mod archetype;
pub mod component;
mod entity;
mod iterators;
pub mod query;
pub mod system;
pub mod system_parameter;
pub mod world;

#[derive(Debug)]
pub enum ECSError {
    EntityNotValid,
    EntityDontHaveComponent,
}
