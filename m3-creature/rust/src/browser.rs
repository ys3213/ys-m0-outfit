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
        }
    }
    fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        for p in [a, b, c] {
            vertex(&mut self.triangles, p, n, color, 0.);
        }
    }
    fn line(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        for p in [a, b] {
            vertex(&mut self.lines, p, Vec3::Z, color, 1.);
        }
    }
    fn ellipsoid(&mut self, model: Mat4, radius: Vec3, color: [f32; 4], glow: f32) {
        let nx = 16;
        let ny = 10;
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
    fn crystal(&mut self, model: Mat4, length: f32, wide: f32, thick: f32) {
        let v = [
            Vec3::ZERO,
            Vec3::new(length * 0.57, -wide, 0.),
            Vec3::new(length, 0., 0.),
            Vec3::new(length * 0.57, wide, 0.),
            Vec3::new(length * 0.5, 0., thick),
            Vec3::new(length * 0.5, 0., -thick),
        ]
        .map(|p| model.transform_point3(p));
        for (i, [a, b, c]) in [
            [0, 1, 4],
            [1, 2, 4],
            [2, 3, 4],
            [3, 0, 4],
            [1, 0, 5],
            [2, 1, 5],
            [3, 2, 5],
            [0, 3, 5],
        ]
        .into_iter()
        .enumerate()
        {
            self.tri(
                v[a],
                v[b],
                v[c],
                if i % 2 == 0 {
                    [0.27, 0.49, 0.58, 0.57]
                } else {
                    [0.43, 0.62, 0.67, 0.4]
                },
            );
        }
        self.line(v[0], v[4], [0.18, 0.34, 0.39, 0.5]);
        self.line(v[4], v[2], [0.18, 0.34, 0.39, 0.5]);
        for [a, b] in [[0, 1], [1, 2], [2, 3], [3, 0]] {
            self.line(v[a], v[b], [0.27, 0.43, 0.46, 0.35]);
        }
    }
    fn garden(&mut self, g: &Garden) {
        let mut order: Vec<_> = g.creatures.iter().collect();
        order.sort_by(|a, b| g.project(a.position).1.total_cmp(&g.project(b.position).1));
        for c in order {
            let t = g.time;
            let r =
                c.size * (0.7 + c.bloom * 0.6) * (1. + (t * 1.6 + c.phase).sin() * 0.065) * c.young;
            let model = Mat4::from_translation(c.position)
                * Mat4::from_rotation_x(0.35 + (t * 0.35 + c.phase).sin() * 0.25)
                * Mat4::from_rotation_y(c.phase + (t * 0.22 + c.phase).sin() * 0.5)
                * Mat4::from_rotation_z((t * 0.3 + c.phase).sin() * 0.24);
            for i in 0..5 {
                let petal = model
                    * Mat4::from_rotation_z(i as f32 * TAU / 5.)
                    * Mat4::from_rotation_y(
                        0.45 - c.bloom * 0.7 + (t + c.phase + i as f32).sin() * 0.06,
                    );
                self.crystal(
                    petal,
                    r * (1.43 + c.bloom * 0.8),
                    r * 0.44 * (0.38 + c.bloom * 0.8),
                    r * 0.22,
                );
            }
            let core = (r * 0.15).max(4.) * (0.7 + c.energy * 0.5);
            self.ellipsoid(
                model,
                Vec3::splat(core),
                [0.18, 0.43, 0.48, 0.96],
                0.35 + c.energy * 0.5,
            );
            self.ellipsoid(
                model,
                Vec3::new(r * 0.84, r, r * 0.72),
                [0.40, 0.63, 0.65, 0.30],
                0.08,
            );
            // 五條膜脈與兩條感絲。水分低時，感絲也會降低擺幅。
            for j in 0..5 {
                let mut prev = None;
                for k in 0..=16 {
                    let a = k as f32 / 16. * PI;
                    let b = j as f32 / 5. * TAU;
                    let p = model.transform_point3(Vec3::new(
                        r * 0.845 * a.sin() * b.cos(),
                        r * a.cos(),
                        r * 0.725 * a.sin() * b.sin(),
                    ));
                    if let Some(old) = prev {
                        self.line(old, p, [0.25, 0.44, 0.47, 0.32]);
                    }
                    prev = Some(p);
                }
            }
            for j in 0..2 {
                let mut prev = None;
                for k in 0..=18 {
                    let a = k as f32 / 18.;
                    let p = model.transform_point3(Vec3::new(
                        (j as f32 - 0.5) * r * 0.45
                            + (a * 4. + t + c.phase).sin() * r * 0.15 * a * (0.3 + c.bloom),
                        r * (0.6 + a * 1.2),
                        (a * 3. + t + c.phase).cos() * r * 0.35 * a,
                    ));
                    if let Some(old) = prev {
                        self.line(old, p, [0.22, 0.40, 0.45, 0.55]);
                    }
                    prev = Some(p);
                }
            }
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
        for w in &g.waves {
            for axis in 0..3 {
                for k in 0..64 {
                    let point = |n: usize| {
                        let a = n as f32 / 64. * TAU;
                        let (x, y) = (a.cos() * w.radius, a.sin() * w.radius);
                        w.position
                            + match axis {
                                0 => Vec3::new(x, y, 0.),
                                1 => Vec3::new(x, 0., y),
                                _ => Vec3::new(0., x, y),
                            }
                    };
                    self.line(
                        point(k),
                        point(k + 1),
                        [0.34, 0.45, 0.51, (1. - w.radius / 350.) * 0.5],
                    );
                }
            }
        }
        for (i, m) in g.mist.iter().enumerate() {
            let d = 1.4 + (i % 3) as f32 * 0.5;
            self.line(
                m.position - Vec3::Y * d,
                m.position + Vec3::Y * d,
                [0.26, 0.52, 0.58, m.life * 0.6],
            );
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
uniform mat4 u_mvp;
out vec3 v_normal;out vec4 v_color;out float v_glow;
void main(){gl_Position=u_mvp*vec4(a_position,1.0);v_normal=a_normal;v_color=a_color;v_glow=a_glow;}"#,
        )?;
        let fs = Self::shader(
            &gl,
            Gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision highp float;
in vec3 v_normal;in vec4 v_color;in float v_glow;out vec4 out_color;
void main(){vec3 n=normalize(v_normal);float a=abs(dot(n,normalize(vec3(-0.4,-0.3,1.0))));float b=pow(abs(dot(n,normalize(vec3(0.5,-0.5,1.0)))),24.0);vec3 c=v_color.rgb*(0.64+0.36*a)+vec3(0.16)*b;out_color=vec4(mix(c,v_color.rgb,v_glow),v_color.a);}"#,
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
        // 保持畫布不透明，避免瀏覽器合成時把薄膜洗成白色。
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
        gl.clear_color(0.957, 0.949, 0.925, 1.);
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
        let mut mesh = Mesh::new();
        mesh.garden(g);
        self.upload(&mesh.triangles, Gl::TRIANGLES);
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
                "感受水霧"
            } else {
                "緩慢漂移"
            },
        );
        text("water", &format!("{:02.0}", g.water() * 100.));
        text("count", &format!("{:02}", g.creatures.len()));
        text("pause", if g.paused { "繼續" } else { "暫停" });
        pressed("pause", g.paused);
        pressed("wind", g.vortex);
        pressed("water-mode", !g.rotate);
        pressed("orbit-mode", g.rotate);
        pressed("single", g.single);
        pressed("colony", !g.single);
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
        let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let mut a = a.borrow_mut();
            a.drag = None;
            if name == "pointerleave" {
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
