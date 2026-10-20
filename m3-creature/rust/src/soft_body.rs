//! 柔膜輪廓改寫自 Holtsetio / Aurelia 的 getBellPosition（MIT）。
//! 原作來源與授權見 vendor/aurelia；囊室、五瓣與分岔是本作品的改作。
use glam::{Mat4, Vec3};
use std::f32::consts::{PI, TAU};

pub fn pose(position: Vec3, time: f32, phase: f32) -> Mat4 {
    Mat4::from_translation(position)
        * Mat4::from_rotation_z(-0.22 + (time * 0.27 + phase).sin() * 0.13)
        * Mat4::from_rotation_x(-0.22 + (time * 0.19 + phase).sin() * 0.16)
        * Mat4::from_rotation_y(phase * 0.2 + time.sin() * 0.035)
}

/// 從頂端 u=0 到褶邊 u=1。邊緣比中心晚半拍，不是整顆等比縮放。
pub fn bell(u: f32, a: f32, time: f32, bloom: f32, phase: f32) -> Vec3 {
    let beat = time * (0.6 + bloom * 0.65) + phase - u * u * 0.95 + PI * 0.5;
    let open = 0.72 + bloom * 0.28;
    let ripple = 1. + (a * 16. + PI * 0.5).sin() * 0.035 * u.powi(4);
    let lobes = 1. + (a * 5. + 0.3).cos() * 0.13 * u.powi(3);
    let angle = (beat.sin() * 0.085 + 0.50) * u * PI;
    let radius = angle.sin() * (beat.sin() * 0.14 + 1.3) * open * ripple * lobes;
    Vec3::new(
        a.sin() * radius,
        -angle.cos() * 0.98 + 0.09 * (a * 5. + beat).sin() * u.powi(3),
        a.cos() * radius * 0.82,
    )
}

pub const STRANDS: usize = 15;
pub const NODES: usize = 19;

#[derive(Clone)]
pub struct Filament {
    pub points: [Vec3; NODES],
    previous: [Vec3; NODES],
    initialized: bool,
}
impl Default for Filament {
    fn default() -> Self {
        Self {
            points: [Vec3::ZERO; NODES],
            previous: [Vec3::ZERO; NODES],
            initialized: false,
        }
    }
}
impl Filament {
    pub fn resize(&mut self, scale: Vec3) {
        for p in &mut self.points {
            *p *= scale;
        }
        for p in &mut self.previous {
            *p *= scale;
        }
    }
    /// 固定根部、保留上一刻位置，再限制節長。尾端因此會落後身體。
    pub fn step(&mut self, anchor: Vec3, length: f32, dt: f32, wind: Vec3) {
        if !self.initialized || self.points[0].distance(anchor) > length * NODES as f32 {
            for i in 0..NODES {
                self.points[i] = anchor + Vec3::Y * length * i as f32;
            }
            self.previous = self.points;
            self.initialized = true;
        }
        if dt <= 0. {
            return;
        }
        let steps = (dt / (1. / 60.)).ceil().max(1.) as usize;
        let h = dt / steps as f32;
        for _ in 0..steps {
            for i in 1..NODES {
                let now = self.points[i];
                let velocity = (now - self.previous[i]) * 0.96;
                self.points[i] += velocity + (wind + Vec3::Y * 22.) * h * h;
                self.previous[i] = now;
            }
            // 交替處理兩端，讓急轉彎也不會把細絲拉到無限長。
            for _ in 0..8 {
                self.points[0] = anchor;
                for i in 1..NODES {
                    let d = self.points[i] - self.points[i - 1];
                    let correction = d.normalize_or_zero() * (d.length() - length);
                    if i == 1 {
                        self.points[i] -= correction;
                    } else {
                        self.points[i - 1] += correction * 0.5;
                        self.points[i] -= correction * 0.5;
                    }
                }
            }
        }
    }
}

pub fn strand_angle(i: usize) -> f32 {
    i as f32 / STRANDS as f32 * TAU
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn membranes_remain_finite_through_dry_and_wet_cycles() {
        for bloom in [0., 0.45, 1.] {
            for t in [0., 3., 13., 90.] {
                for u in 0..=24 {
                    for a in 0..64 {
                        let p = bell(u as f32 / 24., a as f32 / 64. * TAU, t, bloom, 0.3);
                        assert!(p.is_finite() && p.length() < 2.);
                    }
                }
            }
        }
    }
    #[test]
    fn moving_anchor_keeps_strands_attached_and_bounded() {
        let mut f = Filament::default();
        for frame in 0..1200 {
            let t = frame as f32 / 30.;
            let anchor = Vec3::new(t.sin() * 45., t.cos() * 20., t.sin() * 30.);
            f.step(anchor, 7., 1. / 30., Vec3::X * 30.);
            assert_eq!(f.points[0], anchor);
            for pair in f.points.windows(2) {
                assert!(pair[1].is_finite());
                assert!(pair[0].distance(pair[1]) < 10.);
            }
        }
        let saved = f.points;
        f.step(saved[0], 7., 0., Vec3::X);
        assert_eq!(saved, f.points);
    }
}
