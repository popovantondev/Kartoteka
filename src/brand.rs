//! Shared address-book mark for the header, window and executable resources.
pub fn mark_paths() -> Vec<Vec<[f32; 2]>> {
    let mut paths = vec![vec![
        [-5.0, -8.0],
        [6.0, -8.0],
        [7.0, -7.0],
        [7.0, 7.0],
        [6.0, 8.0],
        [-5.0, 8.0],
        [-6.0, 7.0],
        [-6.0, -7.0],
        [-5.0, -8.0],
    ]];
    for y in [-5.0, 0.0, 5.0] {
        paths.push(vec![[-9.0, y], [-4.0, y]]);
    }
    paths.push(
        (0..=32)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 32.0;
                [0.5 + a.cos() * 2.4, -3.0 + a.sin() * 2.4]
            })
            .collect(),
    );
    paths.push(
        (0..=24)
            .map(|i| {
                let t = i as f32 / 24.0;
                let u = 1.0 - t;
                [
                    u * u * u * (-3.5)
                        + 3.0 * u * u * t * (-3.5)
                        + 3.0 * u * t * t * 4.5
                        + t * t * t * 4.5,
                    u * u * u * 5.0
                        + 3.0 * u * u * t * 0.0
                        + 3.0 * u * t * t * 0.0
                        + t * t * t * 5.0,
                ]
            })
            .collect(),
    );
    paths
}
pub fn rgba(size: u32) -> Vec<u8> {
    let paths = mark_paths();
    let mut result = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let mut sums = [0.0_f32; 4];
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = ((x as f32 + (sx as f32 + 0.5) / 4.0) / size as f32 - 0.5) * 24.0;
                    let py = ((y as f32 + (sy as f32 + 0.5) / 4.0) / size as f32 - 0.5) * 24.0;
                    let outside =
                        ((px.abs() - 6.0).max(0.0).hypot((py.abs() - 6.0).max(0.0))) > 6.0;
                    if outside {
                        continue;
                    }
                    let white = paths.iter().any(|path| {
                        path.windows(2).any(|edge| {
                            let a = edge[0];
                            let b = edge[1];
                            let vx = b[0] - a[0];
                            let vy = b[1] - a[1];
                            let t = (((px - a[0]) * vx + (py - a[1]) * vy) / (vx * vx + vy * vy))
                                .clamp(0.0, 1.0);
                            (px - a[0] - t * vx).hypot(py - a[1] - t * vy) <= 0.85
                        })
                    });
                    let rgb = if white {
                        [255.0, 255.0, 255.0]
                    } else {
                        [46.0, 107.0, 255.0]
                    };
                    for c in 0..3 {
                        sums[c] += rgb[c];
                    }
                    sums[3] += 255.0;
                }
            }
            let covered = sums[3] / 255.0;
            for sum in sums.iter().take(3) {
                result.push(if covered > 0.0 {
                    (sum / covered).round() as u8
                } else {
                    0
                });
            }
            result.push((sums[3] / 16.0).round() as u8);
        }
    }
    result
}
