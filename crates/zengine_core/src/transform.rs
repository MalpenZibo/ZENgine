use zengine_macro::Component;

/// A [Component](zengine_ecs::Component) which describe the position of an entity.
///
/// To place or move an entity, you should set its [`Transform`]
#[derive(Component, Debug, Clone)]
pub struct Transform {
    /// Position of the entity. In 2d, the last value of the Vec3 is used for z-ordering
    pub position: glam::Vec3,
    /// Rotation of the entity
    pub rotation: glam::Vec3,
    /// Scale of the entity
    pub scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Vec3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl Transform {
    /// Creates a new trasform from a position, a rotation and a scale
    pub fn new(position: glam::Vec3, rotation: glam::Vec3, scale: f32) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    /// Gets the 3d trasnformation matrix from this transforms translation, rotation, and scale.
    pub fn get_transformation_matrix(&self) -> glam::Mat4 {
        let translation = glam::Mat4::from_translation(self.position);

        let rotation_x = glam::Mat4::from_rotation_x(self.rotation.x.to_radians());
        let rotation_y = glam::Mat4::from_rotation_y(self.rotation.y.to_radians());
        let rotation_z = glam::Mat4::from_rotation_z(self.rotation.z.to_radians());

        let scale = glam::Mat4::from_scale(glam::Vec3::from_array([self.scale; 3]));

        translation * (rotation_x * rotation_y * rotation_z) * scale
    }
}
