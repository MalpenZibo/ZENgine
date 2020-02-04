pub struct Matrix4x4 {
    pub data: [f32; 16]
}

impl Matrix4x4 {
    pub fn identity() -> Matrix4x4 {
        Matrix4x4 {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ]
        }
    }

    pub fn orthographics(
        left: f32, right: f32,
        top: f32, bottom: f32,
        near_clip: f32, far_clip: f32
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


}