use crate::matrix4x4::Matrix4x4;
use crate::vector3::Vector3;
use zengine_macro::Component;

#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: 1.0,
        }
    }
}

impl Transform {
    pub fn new(position: Vector3, rotation: Vector3, scale: f32) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    pub fn get_transformation_matrix(&self) -> Matrix4x4 {
        let translation = Matrix4x4::translation(self.position);
        let rotation = Matrix4x4::rotation(self.rotation);
        let scale = Matrix4x4::scale(Vector3::new(self.scale, self.scale, self.scale));

        translation * rotation * scale
    }

    pub fn get_transformation_matrix_inverse(
        &self,
        translation_inverse: bool,
        rotation_inverse: bool,
        scale_inverse: bool,
    ) -> Matrix4x4 {
        let translation = Matrix4x4::translation(if translation_inverse {
            self.position.inverse()
        } else {
            self.position
        });
        let rotation = Matrix4x4::rotation(if rotation_inverse {
            self.rotation.inverse()
        } else {
            self.rotation
        });
        let scale = Matrix4x4::scale(if scale_inverse {
            Vector3::new(self.scale, self.scale, self.scale).inverse()
        } else {
            Vector3::new(self.scale, self.scale, self.scale)
        });

        translation * rotation * scale
    }
}