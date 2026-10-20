use glam::{Vec2, Vec3};
use std::f32::consts::TAU;
pub const LIMIT: usize = 28;

#[derive(Clone)]
pub struct Creature {
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: f32,
    pub water: f32,
    pub energy: f32,
    pub phase: f32,
    pub age: f32,
    pub bloom: f32,
    pub young: f32,
}
impl Creature {
    fn new(position: Vec3, size: f32, phase: f32) -> Self {
        Self {
            position,
            size,
            phase,
            velocity: Vec3::ZERO,
            water: 0.45,
            energy: 0.45,
            age: 0.,
            bloom: 0.45,
            young: 1.,
        }
    }
    /// Reynolds 的 arrival：理想速度減掉目前速度，再限制轉向力。
    pub fn arrive(&self, target: Vec3) -> Vec3 {
        let offset = target - self.position;
        let speed = 26. * (offset.length() / 110.).min(1.);
        (offset.normalize_or_zero() * speed - self.velocity).clamp_length_max(25.)
    }
}
#[derive(Clone, Copy)]
pub struct Wave {
    pub position: Vec3,
    pub radius: f32,
}
pub struct Mote {
    pub position: Vec3,
    pub life: f32,
}
pub struct Garden {
    pub creatures: Vec<Creature>,
    pub waves: Vec<Wave>,
    pub mist: Vec<Mote>,
    pub time: f32,
    pub rain: f32,
    pub paused: bool,
    pub vortex: bool,
    pub single: bool,
    pub width: f32,
    pub height: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub pointer: Vec2,
    pub inside: bool,
    pub rotate: bool,
    pub shift: bool,
    seed: u32,
}
impl Garden {
    pub fn new(width: f32, height: f32) -> Self {
        let mut g = Self {
            creatures: vec![],
            waves: vec![],
            mist: vec![],
            time: 0.,
            rain: 0.,
            paused: false,
            vortex: false,
            single: true,
            width,
            height,
            yaw: -0.3,
            pitch: -0.15,
            zoom: 1.,
            pointer: Vec2::ZERO,
            inside: false,
            rotate: false,
            shift: false,
            seed: 42,
        };
        g.reset();
        g
    }
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 8) as f32 / 16777216.
    }
    pub fn reset(&mut self) {
        self.creatures.clear();
        self.waves.clear();
        self.mist.clear();
        self.seed = 42;
        self.time = 0.;
        self.rain = 0.;
        self.paused = false;
        self.vortex = false;
        self.rotate = false;
        self.shift = false;
        self.yaw = -0.3;
        self.pitch = -0.15;
        self.zoom = 1.;
        for _ in 0..if self.single { 1 } else { 16 } {
            let position = if self.single {
                Vec3::new(self.width * 0.62, self.height * 0.48, 0.)
            } else {
                Vec3::new(
                    self.width * (0.22 + self.random() * 0.60),
                    self.height * (0.3 + self.random() * 0.35),
                    (self.random() - 0.5) * 260.,
                )
            };
            let size = if self.single {
                (self.width * 0.075).clamp(38., 100.)
            } else {
                24. + self.random() * 10.
            };
            let phase = if self.single {
                0.3
            } else {
                self.random() * TAU
            };
            self.creatures.push(Creature::new(position, size, phase));
        }
    }
    /// 正交投影。前後位置不同，物體仍有真實厚度與遮擋。
    pub fn project(&self, p: Vec3) -> (Vec2, f32) {
        let d = p - Vec3::new(self.width / 2., self.height / 2., 0.);
        let x = self.yaw.cos() * d.x + self.yaw.sin() * d.z;
        let z = -self.yaw.sin() * d.x + self.yaw.cos() * d.z;
        (
            Vec2::new(
                self.width / 2. + x * self.zoom,
                self.height / 2. + (self.pitch.cos() * d.y - self.pitch.sin() * z) * self.zoom,
            ),
            self.pitch.sin() * d.y + self.pitch.cos() * z,
        )
    }
    pub fn pointer_at_depth(&self, depth: f32) -> Vec3 {
        let p = (self.pointer - Vec2::new(self.width / 2., self.height / 2.)) / self.zoom;
        let y = self.pitch.cos() * p.y + self.pitch.sin() * depth;
        let z = -self.pitch.sin() * p.y + self.pitch.cos() * depth;
        Vec3::new(
            self.width / 2. + self.yaw.cos() * p.x - self.yaw.sin() * z,
            self.height / 2. + y,
            self.yaw.sin() * p.x + self.yaw.cos() * z,
        )
    }
    pub fn ripple(&mut self) {
        if !self.rotate && !self.paused {
            self.waves.push(Wave {
                position: self.pointer_at_depth(0.),
                radius: 1.,
            });
        }
    }
    pub fn resize(&mut self, w: f32, h: f32) {
        for c in &mut self.creatures {
            c.position.x *= w / self.width;
            c.position.y *= h / self.height;
            if self.single {
                c.size = (w * 0.075).clamp(38., 100.);
            }
        }
        self.width = w;
        self.height = h;
    }
    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        let dt = dt.clamp(0., 0.05);
        self.time += dt;
        self.rain = (self.rain - dt).max(0.);
        for w in &mut self.waves {
            w.radius += dt * 150.;
        }
        self.waves.retain(|w| w.radius < 350.);
        if self.inside && !self.rotate {
            let mut p = self.pointer_at_depth(0.);
            p += Vec3::new(
                (self.random() - 0.5) * 50.,
                (self.random() - 0.5) * 50.,
                (self.random() - 0.5) * 120.,
            );
            self.mist.push(Mote {
                position: p,
                life: 1.,
            });
        }
        if self.rain > 0. {
            for _ in 0..3 {
                let p = Vec3::new(
                    self.random() * self.width,
                    self.random() * self.height,
                    (self.random() - 0.5) * 360.,
                );
                self.mist.push(Mote {
                    position: p,
                    life: 1.,
                });
            }
        }
        for m in &mut self.mist {
            m.position += Vec3::new(5., 12., 0.) * dt;
            m.life -= dt * 0.6;
        }
        self.mist.retain(|m| m.life > 0.);
        if self.mist.len() > 350 {
            self.mist.drain(..self.mist.len() - 350);
        }
        let mut forces = vec![Vec3::ZERO; self.creatures.len()];
        for (i, c) in self.creatures.iter().enumerate() {
            let (screen, depth) = self.project(c.position);
            let watering =
                self.inside && !self.rotate && screen.distance(self.pointer) < 145. * self.zoom;
            let target = self.pointer_at_depth(depth);
            let mut f = Vec3::new(
                (self.time * 0.25 + c.phase).sin() * 7.,
                (self.time * 0.19 + c.phase).cos() * 4.,
                (self.time * 0.22 + c.phase * 2.).sin() * 6. - c.position.z * 0.025,
            );
            if watering && c.water < 0.78 {
                f += c.arrive(target);
            }
            let wind = if self.inside && !self.rotate {
                target
            } else {
                Vec3::new(self.width / 2., self.height / 2., 0.)
            };
            if (self.vortex || (self.shift && self.inside && !self.rotate))
                && c.position.distance(wind) < 300.
            {
                let d = c.position - wind;
                f += Vec3::new(-d.y * 0.32, d.x * 0.32, 0.);
            }
            for w in &self.waves {
                let delta = c.position - w.position;
                if (delta.length() - w.radius).abs() < 24. {
                    f += delta.normalize_or_zero() * 85.;
                }
            }
            for (j, b) in self.creatures.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = b.position - c.position;
                let length = d.length().max(1.);
                if length < 60. {
                    f -= d / length * 16.;
                } else if length < 145. && c.water > 0.3 {
                    f += d * 0.014;
                }
            }
            f.y += (c.water - 0.73).max(0.) * 100.;
            if c.position.y > self.height * 0.66 {
                f.y -= (c.position.y - self.height * 0.66) * 0.7;
            }
            if c.position.y < self.height * 0.25 {
                f.y += (self.height * 0.25 - c.position.y) * 0.5;
            }
            if c.position.x < 65. {
                f.x += (65. - c.position.x) * 0.4;
            }
            if c.position.x > self.width - 65. {
                f.x -= (c.position.x - self.width + 65.) * 0.4;
            }
            forces[i] = f;
        }
        let count = self.creatures.len();
        let mut children = vec![];
        for i in 0..self.creatures.len() {
            let (screen, _) = self.project(self.creatures[i].position);
            let wet = self.rain > 0.
                || (self.inside
                    && !self.rotate
                    && screen.distance(self.pointer) < 145. * self.zoom);
            let c = &mut self.creatures[i];
            c.age += dt;
            c.young = (c.young + dt * 0.025).min(1.);
            c.water = (c.water + dt * if wet { 0.15 } else { -0.016 }).clamp(0., 1.);
            c.energy = (c.energy + dt * if wet { 0.08 } else { -0.008 }).clamp(0., 1.);
            c.bloom += (c.water - c.bloom) * dt * 2.;
            c.velocity = (c.velocity + forces[i] * dt) * (-dt * 1.5).exp();
            let activity = if c.water < 0.2 { 0.2 } else { 1. };
            c.position += c.velocity * dt * activity;
            c.position.x = c.position.x.clamp(35., (self.width - 35.).max(35.));
            c.position.y = c.position.y.clamp(
                self.height * 0.24,
                (self.height - 140.).max(self.height * 0.24),
            );
            c.position.z = c.position.z.clamp(-190., 190.);
            if !self.single
                && c.age > 18.
                && c.energy > 0.82
                && c.water > 0.6
                && count + children.len() < LIMIT
            {
                let mut child =
                    Creature::new(c.position + Vec3::new(25., 0., 10.), c.size, c.phase + 1.);
                child.young = 0.55;
                child.water = 0.16;
                child.bloom = 0.16;
                child.position.z = child.position.z.clamp(-190., 190.);
                child.position.x = child.position.x.clamp(35., self.width - 35.);
                children.push(child);
                c.energy -= 0.45;
                c.water -= 0.16;
                c.age = 0.;
            }
        }
        self.creatures.extend(children);
    }
    pub fn water(&self) -> f32 {
        self.creatures.iter().map(|c| c.water).sum::<f32>() / self.creatures.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotated_pointer_follows_visible_creature() {
        let mut g = Garden::new(1200., 800.);
        let p = Vec3::new(710., 310., 120.);
        for yaw in [-2., -0.3, 0., 1.57, 2.8] {
            for pitch in [-1., 0., 0.8] {
                for zoom in [0.65, 1., 1.8] {
                    g.yaw = yaw;
                    g.pitch = pitch;
                    g.zoom = zoom;
                    let (s, d) = g.project(p);
                    g.pointer = s;
                    assert!(g.pointer_at_depth(d).distance(p) < 0.001);
                }
            }
        }
    }
    #[test]
    fn life_reacts_and_cannot_grow_forever() {
        let mut g = Garden::new(1200., 800.);
        g.single = false;
        g.reset();
        for _ in 0..16000 {
            g.rain = 8.;
            g.update(0.02);
        }
        assert_eq!(g.creatures.len(), LIMIT);
        assert!(g.water() > 0.6);
        for _ in 0..5000 {
            g.update(0.02);
        }
        assert!(g.water() < 0.2);
        for c in &g.creatures {
            assert!(c.position.is_finite());
            assert!(c.position.z.abs() <= 190.);
        }
    }
    #[test]
    fn pause_and_rotation_do_not_accidentally_water() {
        let mut g = Garden::new(1200., 800.);
        g.inside = true;
        g.rotate = true;
        g.pointer = g.project(g.creatures[0].position).0;
        let w = g.water();
        g.update(0.05);
        assert!(g.water() < w);
        g.paused = true;
        let p = g.creatures[0].position;
        let t = g.time;
        g.update(0.05);
        assert_eq!(g.time, t);
        assert_eq!(g.creatures[0].position, p);
    }
    #[test]
    fn arrival_slows_down_near_water() {
        let c = Creature::new(Vec3::ZERO, 30., 0.);
        assert!(c.arrive(Vec3::X * 10.).length() < c.arrive(Vec3::X * 100.).length());
    }
}
