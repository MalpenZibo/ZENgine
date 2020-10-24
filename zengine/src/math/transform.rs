use crate::core::Component;
use crate::math::matrix4x4::Matrix4x4;
use crate::math::vector3::Vector3;

#[derive(Component, Debug)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::one(),
        }
    }
}

impl Transform {
    pub fn new(position: Vector3, rotation: Vector3, scale: Vector3) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    pub fn get_transformation_matrix(&self) -> Matrix4x4 {
        let translation = Matrix4x4::translation(self.position);
        let rotation = Matrix4x4::rotation(self.rotation);
        let scale = Matrix4x4::scale(self.scale);

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
            self.scale.inverse()
        } else {
            self.scale
        });

        translation * rotation * scale
    }
}
