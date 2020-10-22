#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::suspicious_op_assign_impl)]

use crate::math::vector3::Vector3;
use auto_ops::*;

pub struct Matrix4x4 {
    pub data: [f32; 16],
}

impl_op_ex!(*|a: &Matrix4x4, b: &Matrix4x4| -> Matrix4x4 {
    let mut m = Matrix4x4::identity();

    let b00 = b.data[0];
    let b01 = b.data[1];
    let b02 = b.data[2];
    let b03 = b.data[3];
    let b10 = b.data[4];
    let b11 = b.data[5];
    let b12 = b.data[6];
    let b13 = b.data[7];
    let b20 = b.data[8];
    let b21 = b.data[9];
    let b22 = b.data[10];
    let b23 = b.data[11];
    let b30 = b.data[12];
    let b31 = b.data[13];
    let b32 = b.data[14];
    let b33 = b.data[15];

    let a00 = a.data[0];
    let a01 = a.data[1];
    let a02 = a.data[2];
    let a03 = a.data[3];
    let a10 = a.data[4];
    let a11 = a.data[5];
    let a12 = a.data[6];
    let a13 = a.data[7];
    let a20 = a.data[8];
    let a21 = a.data[9];
    let a22 = a.data[10];
    let a23 = a.data[11];
    let a30 = a.data[12];
    let a31 = a.data[13];
    let a32 = a.data[14];
    let a33 = a.data[15];

    m.data[0] = b00 * a00 + b01 * a10 + b02 * a20 + b03 * a30;
    m.data[1] = b00 * a01 + b01 * a11 + b02 * a21 + b03 * a31;
    m.data[2] = b00 * a02 + b01 * a12 + b02 * a22 + b03 * a32;
    m.data[3] = b00 * a03 + b01 * a13 + b02 * a23 + b03 * a33;
    m.data[4] = b10 * a00 + b11 * a10 + b12 * a20 + b13 * a30;
    m.data[5] = b10 * a01 + b11 * a11 + b12 * a21 + b13 * a31;
    m.data[6] = b10 * a02 + b11 * a12 + b12 * a22 + b13 * a32;
    m.data[7] = b10 * a03 + b11 * a13 + b12 * a23 + b13 * a33;
    m.data[8] = b20 * a00 + b21 * a10 + b22 * a20 + b23 * a30;
    m.data[9] = b20 * a01 + b21 * a11 + b22 * a21 + b23 * a31;
    m.data[10] = b20 * a02 + b21 * a12 + b22 * a22 + b23 * a32;
    m.data[11] = b20 * a03 + b21 * a13 + b22 * a23 + b23 * a33;
    m.data[12] = b30 * a00 + b31 * a10 + b32 * a20 + b33 * a30;
    m.data[13] = b30 * a01 + b31 * a11 + b32 * a21 + b33 * a31;
    m.data[14] = b30 * a02 + b31 * a12 + b32 * a22 + b33 * a32;
    m.data[15] = b30 * a03 + b31 * a13 + b32 * a23 + b33 * a33;

    m
});

impl_op_ex!(*= |a: &mut Matrix4x4, b: &Matrix4x4| {
    let b00 = b.data[0];
    let b01 = b.data[1];
    let b02 = b.data[2];
    let b03 = b.data[3];
    let b10 = b.data[4];
    let b11 = b.data[5];
    let b12 = b.data[6];
    let b13 = b.data[7];
    let b20 = b.data[8];
    let b21 = b.data[9];
    let b22 = b.data[10];
    let b23 = b.data[11];
    let b30 = b.data[12];
    let b31 = b.data[13];
    let b32 = b.data[14];
    let b33 = b.data[15];

    let a00 = a.data[0];
    let a01 = a.data[1];
    let a02 = a.data[2];
    let a03 = a.data[3];
    let a10 = a.data[4];
    let a11 = a.data[5];
    let a12 = a.data[6];
    let a13 = a.data[7];
    let a20 = a.data[8];
    let a21 = a.data[9];
    let a22 = a.data[10];
    let a23 = a.data[11];
    let a30 = a.data[12];
    let a31 = a.data[13];
    let a32 = a.data[14];
    let a33 = a.data[15];

    a.data[0] = b00 * a00 + b01 * a10 + b02 * a20 + b03 * a30;
    a.data[1] = b00 * a01 + b01 * a11 + b02 * a21 + b03 * a31;
    a.data[2] = b00 * a02 + b01 * a12 + b02 * a22 + b03 * a32;
    a.data[3] = b00 * a03 + b01 * a13 + b02 * a23 + b03 * a33;
    a.data[4] = b10 * a00 + b11 * a10 + b12 * a20 + b13 * a30;
    a.data[5] = b10 * a01 + b11 * a11 + b12 * a21 + b13 * a31;
    a.data[6] = b10 * a02 + b11 * a12 + b12 * a22 + b13 * a32;
    a.data[7] = b10 * a03 + b11 * a13 + b12 * a23 + b13 * a33;
    a.data[8] = b20 * a00 + b21 * a10 + b22 * a20 + b23 * a30;
    a.data[9] = b20 * a01 + b21 * a11 + b22 * a21 + b23 * a31;
    a.data[10] = b20 * a02 + b21 * a12 + b22 * a22 + b23 * a32;
    a.data[11] = b20 * a03 + b21 * a13 + b22 * a23 + b23 * a33;
    a.data[12] = b30 * a00 + b31 * a10 + b32 * a20 + b33 * a30;
    a.data[13] = b30 * a01 + b31 * a11 + b32 * a21 + b33 * a31;
    a.data[14] = b30 * a02 + b31 * a12 + b32 * a22 + b33 * a32;
    a.data[15] = b30 * a03 + b31 * a13 + b32 * a23 + b33 * a33;
});

impl Matrix4x4 {
    pub fn identity() -> Matrix4x4 {
        Matrix4x4 {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn orthographics(
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        let r_plus_l = right + left;
        let r_minus_l = right - left;
        let t_plus_b = top + bottom;
        let t_minus_b = top - bottom;
        let f_plus_n = far_clip + near_clip;
        let f_minus_n = far_clip - near_clip;

        m.data[0] = 2.0 / r_minus_l;
        m.data[5] = 2.0 / t_minus_b;
        m.data[10] = -2.0 / f_minus_n;
        m.data[12] = -(r_plus_l / r_minus_l);
        m.data[13] = -(t_plus_b / t_minus_b);
        m.data[14] = -(f_plus_n / f_minus_n);

        m
    }

    pub fn translation(position: Vector3) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        m.data[12] = position.x;
        m.data[13] = position.y;
        m.data[14] = position.z;

        m
    }

    pub fn rotation_x(angle: f32) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        let c = angle.cos();
        let s = angle.sin();

        m.data[5] = c;
        m.data[6] = s;
        m.data[9] = -s;
        m.data[10] = c;

        m
    }

    pub fn rotation_y(angle: f32) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        let c = angle.cos();
        let s = angle.sin();

        m.data[0] = c;
        m.data[2] = -s;
        m.data[8] = s;
        m.data[10] = c;

        m
    }

    pub fn rotation_z(angle: f32) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        let c = angle.cos();
        let s = angle.sin();

        m.data[0] = c;
        m.data[1] = s;
        m.data[4] = -s;
        m.data[5] = c;

        m
    }

    pub fn rotation(angle: Vector3) -> Matrix4x4 {
        let rx = Matrix4x4::rotation_x(angle.x);
        let ry = Matrix4x4::rotation_y(angle.y);
        let rz = Matrix4x4::rotation_z(angle.z);

        rz * ry * rx
    }

    pub fn scale(scale: Vector3) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();

        m.data[0] = scale.x;
        m.data[5] = scale.y;
        m.data[10] = scale.z;

        m
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Self::identity()
    }
}
