use zengine_macro::Component;

#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
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
    pub fn new(position: glam::Vec3, rotation: glam::Vec3, scale: f32) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    pub fn get_transformation_matrix(&self) -> glam::Mat4 {
        let translation = glam::Mat4::from_translation(self.position);

        let rotation_x = glam::Mat4::from_rotation_x(self.rotation.x.to_radians());
        let rotation_y = glam::Mat4::from_rotation_y(self.rotation.y.to_radians());
        let rotation_z = glam::Mat4::from_rotation_z(self.rotation.z.to_radians());

        let scale = glam::Mat4::from_scale(glam::Vec3::from_array([self.scale; 3]));

        translation * (rotation_x * rotation_y * rotation_z) * scale
    }

    // pub fn get_transformation_matrix_inverse(
    //     &self,
    //     translation_inverse: bool,
    //     rotation_inverse: bool,
    //     scale_inverse: bool,
    // ) -> Matrix4x4 {
    //     let translation = Matrix4x4::translation(if translation_inverse {
    //         self.position.inverse()
    //     } else {
    //         self.position
    //     });
    //     let rotation = Matrix4x4::rotation(if rotation_inverse {
    //         self.rotation.inverse()
    //     } else {
    //         self.rotation
    //     });
    //     let scale = Matrix4x4::scale(if scale_inverse {
    //         Vector3::new(self.scale, self.scale, self.scale).inverse()
    //     } else {
    //         Vector3::new(self.scale, self.scale, self.scale)
    //     });

    //     translation * rotation * scale
    // }
}
