mod archetype;
mod component;
mod entity;
pub mod world;

pub enum ECSError {
    EntityNotValid,
    EntityDontHaveComponent,
}
