use crate::life::Garden;
use glam::{Mat4, Vec2, Vec3};
use js_sys::Float32Array;
use std::{
    cell::RefCell,
    f32::consts::{PI, TAU},
    rc::Rc,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, HtmlInputElement, WebGl2RenderingContext as Gl, WebGlBuffer,
    WebGlProgram, WebGlUniformLocation,
};

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}
fn element(id: &str) -> web_sys::Element {
    document().get_element_by_id(id).unwrap()
}
fn text(id: &str, value: &str) {
    element(id).set_text_content(Some(value));
}
fn pressed(id: &str, value: bool) {
    let _ = element(id).set_attribute("aria-pressed", if value { "true" } else { "false" });
}

// 一個頂點：位置、法向量、顏色、不受光影影響的比例。
struct Mesh {
    triangles: Vec<f32>,
    lines: Vec<f32>,
    detailed: bool,
}
fn vertex(out: &mut Vec<f32>, p: Vec3, n: Vec3, color: [f32; 4], glow: f32) {
    out.extend_from_slice(&[
        p.x, p.y, p.z, n.x, n.y, n.z, color[0], color[1], color[2], color[3], glow,
    ]);
}
impl Mesh {
    fn new() -> Self {
        Self {
            triangles: vec![],
            lines: vec![],
            detailed: true,
        }
    }
    fn line(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        for p in [a, b] {
            vertex(&mut self.lines, p, Vec3::Z, color, 1.);
        }
    }
    fn ellipsoid(&mut self, model: Mat4, radius: Vec3, color: [f32; 4], glow: f32) {
        let nx = if self.detailed { 16 } else { 8 };
        let ny = if self.detailed { 10 } else { 5 };
        for y in 0..ny {
            for x in 0..nx {
                let unit = |xx: usize, yy: usize| {
                    let a = xx as f32 / nx as f32 * TAU;
                    let b = yy as f32 / ny as f32 * PI;
                    Vec3::new(b.sin() * a.cos(), b.cos(), b.sin() * a.sin())
                };
                let q = [
                    unit(x, y),
                    unit(x + 1, y),
                    unit(x + 1, y + 1),
                    unit(x, y + 1),
                ];
                for i in [0, 1, 2, 0, 2, 3] {
                    vertex(
                        &mut self.triangles,
                        model.transform_point3(q[i] * radius),
                        model.transform_vector3(q[i] / radius).normalize_or_zero(),
                        color,
                        glow,
                    );
                }
            }
        }
    }
    // 平滑參數曲面：有限差分求法線，轉動後仍能看到膜的弧度。
    fn surface(
        &mut self,
        model: Mat4,
        nu: usize,
        nv: usize,
        color: [f32; 4],
        point: impl Fn(f32, f32) -> Vec3,
    ) {
        let mut grid = Vec::with_capacity((nu + 1) * (nv + 1));
        for y in 0..=nv {
            for x in 0..=nu {
                let u = x as f32 / nu as f32;
                let v = y as f32 / nv as f32;
                let p = point(u, v);
                let du = point(u + 0.0005, v) - point(u - 0.0005, v);
                let dv = point(u, (v + 0.0005).min(1.)) - point(u, (v - 0.0005).max(0.));
                grid.push((
                    model.transform_point3(p),
                    model.transform_vector3(du.cross(dv)).normalize_or_zero(),
                ));
            }
        }
        for y in 0..nv {
            for x in 0..nu {
                for (xx, yy) in [
                    (x, y),
                    (x + 1, y),
                    (x + 1, y + 1),
                    (x, y),
                    (x + 1, y + 1),
                    (x, y + 1),
                ] {
                    let (p, n) = grid[yy * (nu + 1) + xx];
                    vertex(&mut self.triangles, p, n, color, 0.);
                }
            }
        }
    }

    fn tube(&mut self, points: &[Vec3], radius: f32, color: [f32; 4]) {
        for i in 1..points.len() {
            let direction = (points[i] - points[i - 1]).normalize_or_zero();
            let axis = if direction.x.abs() < 0.9 {
                Vec3::X
            } else {
                Vec3::Z
            };
            let right = direction.cross(axis).normalize_or_zero();
            let up = direction.cross(right).normalize_or_zero();
            for side in 0..5 {
                for (end, s) in [
                    (0, side),
                    (1, side),
                    (1, side + 1),
                    (0, side),
                    (1, side + 1),
                    (0, side + 1),
                ] {
                    let a = s as f32 / 5. * TAU;
                    let n = right * a.cos() + up * a.sin();
                    let taper = (1. - (i - 1 + end) as f32 / points.len() as f32)
                        .sqrt()
                        .max(0.15);
                    vertex(
                        &mut self.triangles,
                        points[i - 1 + end] + n * radius * taper,
                        n,
                        color,
                        0.,
                    );
                }
            }
        }
    }
    fn specimen(&mut self, g: &Garden, c: &crate::life::Creature) {
        use crate::soft_body::{bell, pose, strand_angle};
        let r = c.size * c.young;
        let model = pose(c.position, g.time, c.phase);
        let detailed = g.single;
        let fade = |part: u8| {
            if g.focus == 0 || g.focus == part {
                1.
            } else {
                0.13
            }
        };
        // 雙層膠膜與五道向內捲曲的褶瓣；不是硬玻璃外殼。
        for layer in 0..2 {
            let scale = if layer == 0 { 1. } else { 0.91 };
            self.surface(
                model,
                if detailed { 64 } else { 20 },
                if detailed { 24 } else { 10 },
                [
                    0.60,
                    0.72,
                    0.76,
                    (if layer == 0 { 0.25 } else { 0.13 }) * fade(1),
                ],
                |u, v| {
                    bell(v, u * TAU, g.time, c.bloom, c.phase) * r * scale
                        + Vec3::Y * r * layer as f32 * 0.035
                },
            );
        }
        for petal in 0..5 {
            let a = petal as f32 / 5. * TAU;
            self.surface(
                model,
                if detailed { 12 } else { 4 },
                if detailed { 20 } else { 8 },
                [0.70, 0.53, 0.67, 0.30 * fade(1)],
                |u, v| {
                    let angle = a + (u - 0.5) * (0.18 + v * 0.60);
                    let mut p = bell(0.35 + v * 0.64, angle, g.time, c.bloom, c.phase) * r * 0.9;
                    p.y += ((u - 0.5) * PI * 2.).cos() * r * 0.12 * v + r * 0.06;
                    p
                },
            );
            // 細溝延續到褶邊；微小液珠沿著它滑向囊體。
            let path: Vec<_> = (0..=28)
                .map(|i| {
                    model.transform_point3(
                        bell(i as f32 / 28., a, g.time, c.bloom, c.phase) * r * 1.005,
                    )
                })
                .collect();
            if detailed {
                self.tube(&path, r * 0.005, [0.32, 0.48, 0.55, 0.62 * fade(1)]);
            }
            if detailed && c.water > 0.2 {
                let u = 1. - (g.time * 0.07 + petal as f32 * 0.19).fract();
                let p = model.transform_point3(bell(u, a, g.time, c.bloom, c.phase) * r * 1.02);
                self.ellipsoid(
                    Mat4::from_translation(p),
                    Vec3::splat(r * 0.023),
                    [0.76, 0.89, 0.90, 0.75 * fade(1)],
                    0.1,
                );
            }
        }
        // 三個囊室內各有一簇顆粒；水量變化能從囊室大小讀到。
        for j in 0..3 {
            let angle = j as f32 / 3. * TAU + 0.4;
            let center = Vec3::new(angle.sin() * r * 0.32, -r * 0.30, angle.cos() * r * 0.26);
            let inner = model * Mat4::from_translation(center) * Mat4::from_rotation_z(angle * 0.3);
            self.ellipsoid(
                inner,
                Vec3::new(0.24, 0.30, 0.22) * r * (0.75 + c.water * 0.3),
                [0.58, 0.51, 0.66, 0.40 * fade(2)],
                0.,
            );
            for k in 0..if detailed { 5 } else { 2 } {
                let a = k as f32 * 2.4 + j as f32;
                let p = center
                    + Vec3::new(
                        a.cos() * 0.09,
                        (a + g.time * 0.4).sin() * 0.1,
                        a.sin() * 0.09,
                    ) * r;
                self.ellipsoid(
                    model * Mat4::from_translation(p),
                    Vec3::splat(r * (0.033 + c.energy * 0.018)),
                    [0.36, 0.29, 0.43, 0.82 * fade(2)],
                    0.15,
                );
            }
        }
        for (j, strand) in c.filaments.iter().enumerate() {
            if !detailed {
                if j % 3 == 0 {
                    for pair in strand.points.windows(2) {
                        self.line(pair[0], pair[1], [0.4, 0.4, 0.5, 0.5 * fade(3)]);
                    }
                }
                continue;
            }
            self.tube(
                &strand.points,
                r * 0.008,
                [0.36, 0.49, 0.55, 0.70 * fade(3)],
            );
            if detailed || j % 3 == 0 {
                for node in [8, 12, 16] {
                    let p = strand.points[node];
                    let a = strand_angle(j) + node as f32;
                    let branch: Vec<_> = (0..7)
                        .map(|k| {
                            let s = k as f32 / 6.;
                            p + Vec3::new(
                                a.sin() * s * 0.20,
                                s * 0.15 + (s * PI).sin() * 0.05,
                                a.cos() * s * 0.2,
                            ) * r
                        })
                        .collect();
                    self.tube(&branch, r * 0.0045, [0.50, 0.40, 0.56, 0.55 * fade(3)]);
                }
            }
            if detailed && c.water > 0.30 && j % 3 == 0 {
                let p = strand.points[12];
                self.ellipsoid(
                    Mat4::from_translation(p),
                    Vec3::new(0.021, 0.033, 0.021) * r,
                    [0.70, 0.83, 0.85, 0.65 * fade(3)],
                    0.,
                );
            }
        }
    }
    fn garden(&mut self, g: &Garden) {
        self.detailed = g.single;
        let mut order: Vec<_> = g.creatures.iter().collect();
        order.sort_by(|a, b| g.project(a.position).1.total_cmp(&g.project(b.position).1));
        for c in order {
            self.specimen(g, c);
        }
        for (i, a) in g.creatures.iter().enumerate() {
            for b in g.creatures.iter().skip(i + 1) {
                let d = a.position.distance(b.position);
                if d < 155. && a.water > 0.28 && b.water > 0.28 {
                    self.line(
                        a.position,
                        b.position,
                        [0.32, 0.48, 0.50, (1. - d / 155.) * 0.25],
                    );
                }
            }
        }
        // 沿用波動的生命週期與輕推反應，視覺改成逸散的珠光雨泡。
        for w in &g.waves {
            let progress = (w.radius / 350.).clamp(0., 1.);
            let fade = (progress * 8.).min(1.) * (1. - progress).powf(1.4);
            let seed = w.position.x * 0.013 + w.position.y * 0.021;
            for i in 0..7 {
                let a = i as f32 * 2.39996 + seed;
                let spread = progress.sqrt() * (28. + i as f32 * 6.);
                let p = w.position
                    + Vec3::new(
                        a.cos() * spread + (progress * 3. + a).sin() * 8. * progress,
                        a.sin() * spread * 0.55 - progress * 78.,
                        (a + 0.7).cos() * spread * 0.6,
                    );
                let size = (6. + ((i * 7) % 13) as f32) * (0.45 + progress.sqrt() * 0.65);
                let model = Mat4::from_translation(p) * Mat4::from_rotation_z(a + progress * 0.5);
                self.ellipsoid(
                    model,
                    Vec3::new(size, size * 1.12, size * 0.9),
                    [0.68, 0.71, 0.83, fade * 0.56],
                    0.,
                );
            }
        }
        for m in &g.mist {
            let direction = m.velocity.normalize_or_zero();
            // 淡尾、亮端：讓細雨能看出下落方向，而不是靜止的碎點。
            let opacity = ((1. - m.life) * 9.).min(1.) * m.life.min(0.7);
            let middle = m.position - direction * m.length * 0.45;
            self.line(
                m.position - direction * m.length,
                middle,
                [0.49, 0.50, 0.65, opacity * 0.38],
            );
            self.line(middle, m.position, [0.37, 0.46, 0.61, opacity * 0.80]);
        }
        for i in 0..70 {
            let a = i as f32;
            let p = Vec3::new(
                (a * 137.3 + g.time * 2.) % g.width,
                (a * 81.7) % g.height,
                (a * 59.7) % 600. - 300.,
            );
            self.line(p, p + Vec3::X * 1.4, [0.40, 0.49, 0.50, 0.26]);
        }
    }
}

struct Renderer {
    gl: Gl,
    program: WebGlProgram,
    buffer: WebGlBuffer,
    mvp: WebGlUniformLocation,
    view: WebGlUniformLocation,
    canvas: HtmlCanvasElement,
}
impl Renderer {
    fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl: Gl = canvas
            .get_context("webgl2")?
            .ok_or_else(|| {
                JsValue::from_str("瀏覽器未提供 WebGL2；請開啟圖形加速或使用新版瀏覽器")
            })?
            .dyn_into()?;
        let vs = Self::shader(
            &gl,
            Gl::VERTEX_SHADER,
            r#"#version 300 es
precision highp float;
in vec3 a_position; in vec3 a_normal; in vec4 a_color; in float a_glow;
uniform mat4 u_mvp; uniform mat4 u_view;
out vec3 v_normal;out vec4 v_color;out float v_glow;
void main(){gl_Position=u_mvp*vec4(a_position,1.0);v_normal=mat3(u_view)*a_normal;v_color=a_color;v_glow=a_glow;}"#,
        )?;
        let fs = Self::shader(
            &gl,
            Gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision highp float;
in vec3 v_normal;in vec4 v_color;in float v_glow;out vec4 out_color;
// Pearl-film approximation: directional interference colours, not a physical refraction solver.
void main(){
    vec3 n=normalize(v_normal+vec3(0.00001));
    float facing=abs(n.z);
    float edge=pow(1.0-facing,2.2);
    float light=0.5+0.5*dot(n,normalize(vec3(-0.5,-0.6,1.0)));
    vec3 pearl=0.64+0.25*cos(vec3(0.1,2.0,4.3)+facing*7.0+n.x*2.5);
    float spec=pow(abs(dot(n,normalize(vec3(-0.6,-0.7,1.0)))),40.0);
    vec3 c=mix(v_color.rgb*(0.72+0.28*light),pearl,0.45+edge*0.25);
    c=mix(c,vec3(1.0),spec*0.92);
    float alpha=v_color.a*(0.40+edge*1.9+spec*0.8);
    out_color=vec4(mix(c,v_color.rgb,v_glow),mix(alpha,v_color.a,v_glow));
}"#,
        )?;
        let program = gl.create_program().ok_or("無法建立繪圖程式")?;
        gl.attach_shader(&program, &vs);
        gl.attach_shader(&program, &fs);
        gl.link_program(&program);
        if !gl
            .get_program_parameter(&program, Gl::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            return Err(JsValue::from_str(
                &gl.get_program_info_log(&program).unwrap_or_default(),
            ));
        }
        let buffer = gl.create_buffer().ok_or("無法建立繪圖緩衝")?;
        let mvp = gl
            .get_uniform_location(&program, "u_mvp")
            .ok_or("找不到投影參數")?;
        let view = gl
            .get_uniform_location(&program, "u_view")
            .ok_or("找不到視角參數")?;
        gl.use_program(Some(&program));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&buffer));
        for (name, size, offset) in [
            ("a_position", 3, 0),
            ("a_normal", 3, 12),
            ("a_color", 4, 24),
            ("a_glow", 1, 40),
        ] {
            let a = gl.get_attrib_location(&program, name) as u32;
            gl.enable_vertex_attrib_array(a);
            gl.vertex_attrib_pointer_with_i32(a, size, Gl::FLOAT, false, 44, offset);
        }
        gl.enable(Gl::BLEND);
        // 分別累積色彩與 alpha，讓薄膜正確合成到 CSS 的霧灰背景。
        gl.blend_func_separate(
            Gl::SRC_ALPHA,
            Gl::ONE_MINUS_SRC_ALPHA,
            Gl::ONE,
            Gl::ONE_MINUS_SRC_ALPHA,
        );
        gl.enable(Gl::DEPTH_TEST);
        gl.depth_func(Gl::LEQUAL);
        Ok(Self {
            gl,
            program,
            buffer,
            mvp,
            view,
            canvas,
        })
    }
    fn shader(gl: &Gl, kind: u32, source: &str) -> Result<web_sys::WebGlShader, JsValue> {
        let s = gl.create_shader(kind).ok_or("無法建立著色器")?;
        gl.shader_source(&s, source);
        gl.compile_shader(&s);
        if !gl
            .get_shader_parameter(&s, Gl::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            return Err(JsValue::from_str(
                &gl.get_shader_info_log(&s).unwrap_or_default(),
            ));
        }
        Ok(s)
    }
    fn draw(&self, g: &Garden) {
        let gl = &self.gl;
        let dpr = web_sys::window().unwrap().device_pixel_ratio().min(1.5);
        let (w, h) = (
            (g.width as f64 * dpr) as u32,
            (g.height as f64 * dpr) as u32,
        );
        if self.canvas.width() != w || self.canvas.height() != h {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }
        gl.viewport(0, 0, w as i32, h as i32);
        gl.clear_color(0., 0., 0., 0.);
        gl.depth_mask(true);
        gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
        gl.use_program(Some(&self.program));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.buffer));
        let mvp = Mat4::orthographic_rh_gl(
            -g.width / 2.,
            g.width / 2.,
            g.height / 2.,
            -g.height / 2.,
            -1400.,
            1400.,
        ) * Mat4::from_scale(Vec3::splat(g.zoom))
            * Mat4::from_rotation_x(g.pitch)
            * Mat4::from_rotation_y(g.yaw)
            * Mat4::from_translation(Vec3::new(-g.width / 2., -g.height / 2., 0.));
        gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp), false, &mvp.to_cols_array());
        let view = Mat4::from_rotation_x(g.pitch) * Mat4::from_rotation_y(g.yaw);
        gl.uniform_matrix4fv_with_f32_array(Some(&self.view), false, &view.to_cols_array());
        let mut mesh = Mesh::new();
        mesh.garden(g);
        // 所有半透明三角形按視角由後往前畫，內部囊室才不會被外膜錯誤遮掉。
        let mut triangles: Vec<_> = mesh.triangles.chunks_exact(33).collect();
        triangles.sort_by_cached_key(|v| {
            let p = Vec3::new(
                (v[0] + v[11] + v[22]) / 3.,
                (v[1] + v[12] + v[23]) / 3.,
                (v[2] + v[13] + v[24]) / 3.,
            );
            (g.project(p).1 * 100.) as i32
        });
        let sorted: Vec<f32> = triangles.into_iter().flatten().copied().collect();
        gl.depth_mask(false);
        self.upload(&sorted, Gl::TRIANGLES);
        self.upload(&mesh.lines, Gl::LINES);
    }
    fn upload(&self, vertices: &[f32], mode: u32) {
        let data = Float32Array::from(vertices);
        self.gl
            .buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &data, Gl::DYNAMIC_DRAW);
        self.gl.draw_arrays(mode, 0, (vertices.len() / 11) as i32);
    }
}
struct App {
    garden: Garden,
    renderer: Renderer,
    last: f64,
    drag: Option<Vec2>,
    frames: u32,
}
impl App {
    fn ui(&self) {
        let g = &self.garden;
        text(
            "status",
            if g.paused {
                "時間停在這裡"
            } else if g.rain > 0. {
                "雨正在落下"
            } else if g.water() < 0.2 {
                "收攏 · 休眠"
            } else if g.inside && !g.rotate {
                "游標帶來細雨"
            } else {
                "緩慢漂移"
            },
        );
        text("water", &format!("{:02.0}", g.water() * 100.));
        text("count", &format!("{:02}", g.creatures.len()));
        element("canvas")
            .set_attribute("data-raindrops", &g.mist.len().to_string())
            .ok();
        element("canvas")
            .set_attribute("data-rain-blooms", &g.waves.len().to_string())
            .ok();
        text("pause", if g.paused { "繼續" } else { "暫停" });
        pressed("pause", g.paused);
        pressed("wind", g.vortex);
        pressed("water-mode", !g.rotate);
        pressed("orbit-mode", g.rotate);
        pressed("single", g.single);
        pressed("colony", !g.single);
        pressed("inspect", g.inspect);
        element("inspect")
            .set_attribute("aria-expanded", if g.inspect { "true" } else { "false" })
            .ok();
        if g.inspect {
            element("anatomy").remove_attribute("hidden").ok();
        } else {
            element("anatomy").set_attribute("hidden", "").ok();
        }
        for (i, id) in ["whole", "membrane", "chambers", "filaments"]
            .iter()
            .enumerate()
        {
            pressed(id, g.focus == i as u8);
        }
        let (name, description) = match g.focus {
            1 => (
                "膠質外膜",
                "水霧漸濃，五道褶瓣慢慢展開。液珠沿著細溝聚攏；外膜的珠光隨觀察角度改變。",
            ),
            2 => (
                "內部囊室",
                "三個半透明囊室隨含水量脹縮，深色顆粒留在裡面。外膜淡去後，可以轉到背面觀察。",
            ),
            3 => (
                "分岔細絲",
                "身體先移動，細絲才跟上。末端分出更細的枝節；水霧聚成小珠，掛在絲上。",
            ),
            _ => (
                "一個移動的棲地",
                "膜、囊室與細絲一起呼吸。把水霧留在牠身旁，觀察展開、蓄水與下沉。",
            ),
        };
        text("part-title", name);
        text("part-description", description);
        element("canvas")
            .set_attribute("data-focus", &g.focus.to_string())
            .ok();
        element("exhibit")
            .set_attribute("data-inspect", if g.inspect { "true" } else { "false" })
            .ok();
        element("canvas")
            .set_attribute("data-input", if g.rotate { "orbit" } else { "mist" })
            .ok();
        element("canvas")
            .set_attribute("data-engine", "rust-webassembly")
            .ok();
        element("canvas")
            .set_attribute("data-water", &format!("{:0.4}", g.water()))
            .ok();
        element("canvas")
            .set_attribute("data-population", &g.creatures.len().to_string())
            .ok();
        element("canvas")
            .set_attribute("data-time", &format!("{:0.2}", g.time))
            .ok();
        element("canvas")
            .set_attribute(
                "data-view",
                &format!("{:0.3},{:0.3},{:0.2}", g.yaw, g.pitch, g.zoom),
            )
            .ok();
        element("zoom")
            .dyn_ref::<HtmlInputElement>()
            .unwrap()
            .set_value(&(g.zoom * 100.).round().to_string());
    }
    fn frame(&mut self, now: f64) {
        // 30 fps 足以呈現呼吸與漂移，也減少展示時耗電。
        if self.last > 0. && now - self.last < 32. {
            return;
        }
        let dt = if self.last == 0. {
            0.016
        } else {
            ((now - self.last) / 1000.) as f32
        };
        self.last = now;
        let r = self.renderer.canvas.get_bounding_client_rect();
        let (w, h) = (r.width() as f32, r.height() as f32);
        if w > 0. && h > 0. && (w != self.garden.width || h != self.garden.height) {
            self.garden.resize(w, h);
        }
        self.garden.update(dt);
        self.renderer.draw(&self.garden);
        self.frames += 1;
        if self.frames % 5 == 0 {
            self.ui();
        }
        // 將 Codrops effect 1 的縮放揭露節奏改由 Rust 驅動，不引入 GSAP／Splitting。
        let title = element("world-title").dyn_into::<HtmlElement>().unwrap();
        let rect = title.get_bounding_client_rect();
        let progress =
            ((self.garden.height - rect.top() as f32) / (self.garden.height * 0.75)).clamp(0., 1.);
        let reduced = web_sys::window()
            .unwrap()
            .match_media("(prefers-reduced-motion: reduce)")
            .ok()
            .flatten()
            .map(|m| m.matches())
            .unwrap_or(false);
        let scale = if reduced { 1. } else { 0.75 + progress * 0.25 };
        title
            .style()
            .set_property("transform", &format!("scale({scale})"))
            .ok();
    }
}
fn on_click(app: &Rc<RefCell<App>>, id: &str, action: fn(&mut Garden)) -> Result<(), JsValue> {
    let app = app.clone();
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        let mut a = app.borrow_mut();
        action(&mut a.garden);
        a.ui();
    });
    element(id).add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
    c.forget();
    Ok(())
}
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = element("canvas").dyn_into::<HtmlCanvasElement>()?;
    let rect = canvas.get_bounding_client_rect();
    let renderer = Renderer::new(canvas.clone())?;
    let app = Rc::new(RefCell::new(App {
        garden: Garden::new(rect.width() as f32, rect.height() as f32),
        renderer,
        last: 0.,
        drag: None,
        frames: 0,
    }));
    on_click(&app, "rain", |g| g.rain = 8.)?;
    on_click(&app, "wind", |g| g.vortex = !g.vortex)?;
    on_click(&app, "pause", |g| g.paused = !g.paused)?;
    on_click(&app, "reset", |g| g.reset())?;
    on_click(&app, "water-mode", |g| g.rotate = false)?;
    on_click(&app, "orbit-mode", |g| g.rotate = true)?;
    on_click(&app, "single", |g| {
        g.single = true;
        g.reset();
    })?;
    on_click(&app, "colony", |g| {
        g.single = false;
        g.reset();
    })?;
    on_click(&app, "front", |g| {
        g.yaw = 0.;
        g.pitch = 0.;
        g.zoom = 1.;
    })?;
    on_click(&app, "inspect", |g| {
        g.inspect = !g.inspect;
        if !g.inspect {
            g.focus = 0;
        }
    })?;
    on_click(&app, "whole", |g| g.focus = 0)?;
    on_click(&app, "membrane", |g| g.focus = 1)?;
    on_click(&app, "chambers", |g| g.focus = 2)?;
    on_click(&app, "filaments", |g| g.focus = 3)?;
    let a = app.clone();
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        let input = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        a.borrow_mut().garden.zoom = input.value().parse::<f32>().unwrap_or(100.) / 100.;
    });
    element("zoom").add_event_listener_with_callback("input", c.as_ref().unchecked_ref())?;
    c.forget();
    let a = app.clone();
    let c = Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
        let mut a = a.borrow_mut();
        let r = a.renderer.canvas.get_bounding_client_rect();
        let p = Vec2::new(
            (e.client_x() as f64 - r.left()) as f32,
            (e.client_y() as f64 - r.top()) as f32,
        );
        a.garden.pointer = p;
        a.garden.inside = true;
        if a.garden.rotate {
            a.drag = Some(p);
            a.renderer.canvas.set_pointer_capture(e.pointer_id()).ok();
        } else {
            a.garden.ripple();
        }
    });
    canvas.add_event_listener_with_callback("pointerdown", c.as_ref().unchecked_ref())?;
    c.forget();
    let a = app.clone();
    let c = Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
        let mut a = a.borrow_mut();
        let r = a.renderer.canvas.get_bounding_client_rect();
        let p = Vec2::new(
            (e.client_x() as f64 - r.left()) as f32,
            (e.client_y() as f64 - r.top()) as f32,
        );
        if a.garden.rotate {
            if let Some(old) = a.drag {
                a.garden.yaw += (p.x - old.x) * 0.008;
                a.garden.pitch = (a.garden.pitch + (p.y - old.y) * 0.008).clamp(-1.1, 1.1);
                a.drag = Some(p);
            }
        }
        a.garden.pointer = p;
        a.garden.inside = true;
    });
    canvas.add_event_listener_with_callback("pointermove", c.as_ref().unchecked_ref())?;
    c.forget();
    for name in ["pointerup", "pointercancel", "pointerleave"] {
        let a = app.clone();
        let c =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
                let mut a = a.borrow_mut();
                a.drag = None;
                if name != "pointerup" || e.pointer_type() != "mouse" {
                    a.garden.inside = false;
                }
            });
        canvas.add_event_listener_with_callback(name, c.as_ref().unchecked_ref())?;
        c.forget();
    }
    // 滾輪在旋轉模式才攔截，閱讀展場文字時仍可正常捲動。
    let a = app.clone();
    let c = Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |e: web_sys::WheelEvent| {
        let mut a = a.borrow_mut();
        if a.garden.rotate {
            e.prevent_default();
            a.garden.zoom = (a.garden.zoom * (-e.delta_y() as f32 * 0.001).exp()).clamp(0.65, 1.8);
        }
    });
    canvas.add_event_listener_with_callback("wheel", c.as_ref().unchecked_ref())?;
    c.forget();
    for event in ["keydown", "keyup"] {
        let a = app.clone();
        let c =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                let mut a = a.borrow_mut();
                if e.code() == "ShiftLeft" || e.code() == "ShiftRight" {
                    a.garden.shift = event == "keydown";
                }
                if event != "keydown" || e.repeat() {
                    return;
                }
                if e.code() == "Escape" {
                    element("guide").set_attribute("hidden", "").ok();
                    element("help").set_attribute("aria-expanded", "false").ok();
                    a.garden.inspect = false;
                    a.garden.focus = 0;
                    a.ui();
                    return;
                }
                if e.target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .is_some_and(|t| ["INPUT", "BUTTON", "A"].contains(&t.tag_name().as_str()))
                {
                    return;
                }
                match e.code().as_str() {
                    "Space" => {
                        e.prevent_default();
                        a.garden.paused = !a.garden.paused;
                    }
                    "KeyR" => a.garden.reset(),
                    "Escape" => {
                        element("guide").set_attribute("hidden", "").ok();
                        element("help").set_attribute("aria-expanded", "false").ok();
                    }
                    _ => {}
                }
            });
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(event, c.as_ref().unchecked_ref())?;
        c.forget();
    }
    let a = app.clone();
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        let mut a = a.borrow_mut();
        a.garden.shift = false;
        a.drag = None;
        a.garden.inside = false;
    });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("blur", c.as_ref().unchecked_ref())?;
    c.forget();
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        let panel = element("guide");
        let open = panel.has_attribute("hidden");
        if open {
            panel.remove_attribute("hidden").ok();
        } else {
            panel.set_attribute("hidden", "").ok();
        }
        element("help")
            .set_attribute("aria-expanded", if open { "true" } else { "false" })
            .ok();
    });
    element("help").add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
    c.forget();
    app.borrow().ui();
    text("load-state", "");
    element("loading").set_attribute("hidden", "")?;
    let callback = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let next = callback.clone();
    *callback.borrow_mut() = Some(Closure::new(move |now| {
        app.borrow_mut().frame(now);
        web_sys::window()
            .unwrap()
            .request_animation_frame(next.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}
