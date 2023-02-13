
extern crate nalgebra as na;

pub fn view4(
    position: &na::Point4<f32>,
    x: &na::Vector4<f32>,
    y: &na::Vector4<f32>,
    z: &na::Vector4<f32>,
    w: &na::Vector4<f32>,
) -> na::Matrix5<f32> {
    let m_trans = translate4(&na::Matrix5::identity(), &(na::Point4::origin() - position));

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

pub fn perspective4(n: f32, f: f32) -> na::Matrix5<f32> {
    let m = na::Matrix5::new(
        n, 0., 0., 0., 0.,
        0., n, 0., 0., 0.,
        0., 0., n, 0., 0.,
        0., 0., 0., n+f, -n*f,
        0., 0., 0., 1., 0.,
    );

    return m
}

pub fn ortho4(
    x_near: f32, x_far: f32,
    y_near: f32, y_far: f32,
    z_near: f32, z_far: f32,
    w_near: f32, w_far: f32,
) -> na::Matrix5<f32> {
    let m_trans = translate4(
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

pub fn ortho4_short(w_near: f32, w_far: f32, half_width: f32) -> na::Matrix5<f32> {
    ortho4(
        -half_width, half_width,
        -half_width, half_width,
        -half_width, half_width,
        w_near, w_far,
    )
}

pub fn translate4(m: &na::Matrix5<f32>, v: &na::Vector4<f32>) -> na::Matrix5<f32> {
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

pub fn scale4(m: &na::Matrix5<f32>, v: &na::Vector4<f32>) -> na::Matrix5<f32> {
    let mut res = na::Matrix5::zeros();

    res.set_column(0, &(m.column(0) * v[0]));
    res.set_column(1, &(m.column(1) * v[1]));
    res.set_column(2, &(m.column(2) * v[2]));
    res.set_column(3, &(m.column(3) * v[3]));
    res.set_column(4, &(m.column(4) * v[4]));
    
    res
}

// four-dimensional cross product
pub fn cross4(
    x: &na::Vector4<f32>,
    y: &na::Vector4<f32>,
    z: &na::Vector4<f32>
) -> na::Vector4<f32> {
    
    let a = (y[0] * z[1]) - (y[1] * z[0]);
    let b = (y[0] * z[2]) - (y[2] * z[0]);
    let c = (y[0] * z[3]) - (y[3] * z[0]);
    let d = (y[1] * z[2]) - (y[2] * z[1]);
    let e = (y[1] * z[3]) - (y[3] * z[1]);
    let f = (y[2] * z[3]) - (y[3] * z[2]);

    let result = na::Vector4::new(
          (x[1] * f) - (x[2] * e) + (x[3] * d),
        - (x[0] * f) + (x[2] * c) - (x[3] * b),
          (x[0] * e) - (x[1] * c) + (x[3] * a),
        - (x[0] * d) + (x[1] * b) - (x[2] * a),
    );
    
    result
}

pub fn ratate4_xy(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(0,0)] = angle.cos();
    res[(0,1)] = angle.sin();
    res[(1,0)] = -angle.sin();
    res[(1,1)] = angle.cos();

    res
}

pub fn ratate4_yz(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(1,1)] = angle.cos();
    res[(1,2)] = angle.sin();
    res[(2,1)] = -angle.sin();
    res[(2,2)] = angle.cos();

    res
}

pub fn ratate4_zx(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(0,0)] = angle.cos();
    res[(0,2)] = -angle.sin();
    res[(2,0)] = angle.sin();
    res[(2,2)] = angle.cos();

    res
}

pub fn ratate4_xw(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(0,0)] = angle.cos();
    res[(0,3)] = angle.sin();
    res[(3,0)] = -angle.sin();
    res[(3,3)] = angle.cos();

    res
}

pub fn ratate4_yw(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(1,1)] = angle.cos();
    res[(1,3)] = -angle.sin();
    res[(3,1)] = angle.sin();
    res[(3,3)] = angle.cos();

    res
}

pub fn ratate4_zw(angle: f32) -> na::Matrix5<f32> {
    let mut res: na::Matrix5<f32> = na::Matrix5::identity();

    res[(2,2)] = angle.cos();
    res[(2,3)] = -angle.sin();
    res[(3,2)] = angle.sin();
    res[(3,3)] = angle.cos();

    res
}