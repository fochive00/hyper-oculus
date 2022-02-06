
extern crate nalgebra as na;

pub fn view(
    position: &na::Point4<f32>,
    x: &na::Vector4<f32>,
    y: &na::Vector4<f32>,
    z: &na::Vector4<f32>,
    w: &na::Vector4<f32>,
) -> na::Matrix5<f32> {
    let m_trans = translate(&na::Matrix5::identity(), &(na::Point4::origin() - position));

    let x = x.normalize();
    let y = y.normalize();
    let z = z.normalize();
    let w = w.normalize();

    let m_rot = na::Matrix5::new(
        x[0], x[1], x[2], x[3], 0.0,
        y[0], y[1], y[2], y[3], 0.0,
        z[0], z[1], z[2], z[3], 0.0,
        -w[0], -w[1], -w[2], -w[3], 0.0,
        0.0, 0.0, 0.0, 0.0, 1.0,
    );

    m_rot * m_trans
}

pub fn ortho(
    x_near: f32, x_far: f32,
    y_near: f32, y_far: f32,
    z_near: f32, z_far: f32,
    w_near: f32, w_far: f32,
) -> na::Matrix5<f32> {
    let m_trans = translate(
        &na::Matrix5::identity(), 
        &na::Vector4::new(
            -(x_near + x_far) / 2.0,
            -(y_near + y_far) / 2.0,
            -(z_near + z_far) / 2.0,
            -(w_near + w_far) / 2.0,
        )
    );

    let m_scale = na::Matrix5::from_diagonal(
        &na::Vector5::new(
            2.0 / (x_far - x_near),
            2.0 / (y_far - y_near),
            2.0 / (z_far - z_near),
            2.0 / (w_near - w_far),
            1.0,
        )
    );

    m_scale * m_trans
}

pub fn ortho_short(w_near: f32, w_far: f32, half_width: f32) -> na::Matrix5<f32> {
    ortho(
        -half_width, half_width,
        -half_width, half_width,
        -half_width, half_width,
        w_near, w_far,
    )
}

pub fn translate(m: &na::Matrix5<f32>, v: &na::Vector4<f32>) -> na::Matrix5<f32> {
    let mut res = m.clone();

    res.set_column(
        4,
        &(
        m.column(0) * v[0] + 
        m.column(1) * v[1] +
        m.column(2) * v[2] +
        m.column(3) * v[3] +
        m.column(4)
    ));

    res
}

pub fn scale(m: &na::Matrix5<f32>, v: &na::Vector4<f32>) -> na::Matrix5<f32> {
    let mut res = na::Matrix5::zeros();

    res.set_column(0, &(m.column(0) * v[0]));
    res.set_column(1, &(m.column(1) * v[1]));
    res.set_column(2, &(m.column(2) * v[2]));
    res.set_column(3, &(m.column(3) * v[3]));
    res.set_column(4, &(m.column(4) * v[4]));
    
    res
}

pub fn cross(
    x: &na::Vector4<f32>,
    y: &na::Vector4<f32>,
    z: &na::Vector4<f32>
) -> na::Vector4<f32> {
    
    // let row_a = na::RowVector4:(a);
    // let row_b = na::RowVector4:(b);
    // let row_c = na::RowVector4:(c);
    // let na::M
    let mat: na::Matrix3x4<f32> = na::Matrix3x4::from_rows(&[x.transpose(), y.transpose(), z.transpose()]);
    
    let w = na::Vector4::new(
        mat.remove_column(0).determinant(),
        mat.remove_column(1).determinant(),
        mat.remove_column(2).determinant(),
        mat.remove_column(3).determinant()
    );
    
    // println!("{:?}", w);
    w
}