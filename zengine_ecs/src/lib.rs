mod archetype;
mod component;
mod entity;
pub mod system;
pub mod world;

#[derive(Debug)]
pub enum ECSError {
    EntityNotValid,
    EntityDontHaveComponent,
}
