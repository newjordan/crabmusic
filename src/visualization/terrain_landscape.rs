// Green Grid Landscape - Procedural moving landscape with audio reactivity
// Renders a retro wireframe terrain and flies forward over it

use super::{lerp, BrailleGrid, GridBuffer, Visualizer, Color};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;

use std::time::Instant;


pub struct TerrainLandscapeVisualizer {
    // Visual tuning
    grid_cols: usize,
    grid_rows: usize,
    spacing_x: f32,   // world units between columns
    spacing_z: f32,   // world units between rows (depth)

    // Road & Sun ambience
    road_half_width: f32,
    start_time: Instant,
    sun_cycle_seconds: f32,

    near_plane: f32,
    far_plane: f32,

    // Motion
    scroll_z: f32,    // world units progressed forward
    base_speed: f32,  // world units per frame

    // Height shaping
    base_height: f32,
    phase: f32,
    beat_pulse: f32,

    // Smoothed audio bands
    bass: f32,
    mid: f32,
    treble: f32,
    amplitude: f32,

    // Precomputed x positions (world units, centered) and marching row buffer
    xs_base: Vec<f32>,
    height_rows: Vec<Vec<f32>>, // rows x cols heights (world y relative)
    profile_z: f32,              // noise-space z cursor for generating new rows

    // Theme
    _color_scheme: ColorScheme,
}

impl TerrainLandscapeVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        // Base grid settings
        let grid_cols = 22;
        let grid_rows = 34;
        let spacing_x = 1.0;
        let spacing_z = 1.2;

        // Precompute centered base x positions (world units)
        let mut xs_base: Vec<f32> = Vec::with_capacity(grid_cols);
        for i in 0..grid_cols {
            let x = (i as f32 - (grid_cols as f32 - 1.0) * 0.5) * spacing_x;
            xs_base.push(x);
        }

        let mut s = Self {
            grid_cols,
            grid_rows,
            spacing_x,
            spacing_z,

            // Road & Sun ambience
            road_half_width: 6.0,
            start_time: Instant::now(),
            sun_cycle_seconds: 300.0,

            near_plane: 2.2,
            far_plane: 70.0,

            scroll_z: 0.0,
            base_speed: 0.018, // much slower travel pace

            base_height: 9.0,
            phase: 0.0,
            beat_pulse: 0.0,

            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            amplitude: 0.0,

            xs_base,
            height_rows: Vec::new(),
            profile_z: 0.0,

            _color_scheme: color_scheme,
        };
        s.seed_rows();
        s
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self._color_scheme = color_scheme;
    }

    #[inline]
    fn height_fn(&self, x: f32, z: f32) -> f32 {
        // Simple old-school mountainous terrain using fBm noise
        // Smooth and random, no road/valley shaping
        let freq = 0.08;
        let n = fbm((x) * freq, (z + 23.0) * freq);
        // Ridged look but softened for continuity
        let ridged = n.abs().powf(1.25); // 0..1
        let h = self.base_height * (0.2 + 0.8 * ridged); // keep some floor, avoid deep pits
        h
    }

    fn seed_rows(&mut self) {
        self.height_rows.clear();
        let mut z = self.profile_z;
        for _ in 0..self.grid_rows {
            self.height_rows.push(self.gen_row_at(z));
            z += self.spacing_z;
        }
        self.profile_z = z;
        // Initial smoothing passes for very soft topology
        self.smooth_height_rows_pass();
        self.smooth_height_rows_pass();
    }

    fn gen_row_at(&self, z_seed: f32) -> Vec<f32> {
        let mut row: Vec<f32> = Vec::with_capacity(self.grid_cols);
        for &x in &self.xs_base {
            let dx = (x.abs() - self.road_half_width).max(0.0);
            let shoulder = 3.5; // blend width from road to walls
            let w = smoothstep01((dx / shoulder).clamp(0.0, 1.0));

            // Valley profile that rises with distance from road
            let valley_gain = self.base_height * 0.8;
            let valley = valley_gain * (dx / (self.road_half_width + shoulder)).powf(1.15);

            // Noise adds shape outside the road, but never dips below road level
            let side_shift = if x >= 0.0 { 17.0 } else { -17.0 };
            let n = fbm((x + side_shift) * 0.085, (z_seed + 23.0) * 0.085);
            let n_pos = 0.5 * (n + 1.0); // 0..1
            let noise_amp = self.base_height * 0.6 * w; // softened noise for smoother topology
            let mountains = (valley + noise_amp * n_pos).max(0.0);

            // Road is the lowest point
            let h = lerp(0.0, mountains, w);
            row.push(h);
        }
        row
    }
    fn smooth_row_cols(row: &mut [f32]) {
        if row.len() < 3 { return; }
        let mut tmp = row.to_vec();
        for i in 0..row.len() {
            let l = if i>0 { row[i-1] } else { row[i] };
            let c = row[i];
            let r = if i+1<row.len() { row[i+1] } else { row[i] };
            tmp[i] = (l + 2.0*c + r) * 0.25; // simple [1,2,1]/4 kernel
        }
        row.copy_from_slice(&tmp);
    }

    fn smooth_height_rows_pass(&mut self) {
        // Smooth columns per row
        for r in &mut self.height_rows {
            Self::smooth_row_cols(r);
        }
        // Smooth across rows for each column
        if self.height_rows.len() < 3 { return; }
        let rows = self.height_rows.len();
        let cols = self.grid_cols;
        let src = self.height_rows.clone();
        for j in 0..rows {
            for i in 0..cols {
                let a = if j>0 { src[j-1][i] } else { src[j][i] };
                let b = src[j][i];
                let c = if j+1<rows { src[j+1][i] } else { src[j][i] };
                self.height_rows[j][i] = (a + 2.0*b + c) * 0.25;
            }
        }
    }

    fn smooth_tail_rows(&mut self, tail_rows: usize) {
        let rows = self.height_rows.len();
        if rows == 0 { return; }
        let start = rows.saturating_sub(tail_rows);
        // Smooth columns per row in tail
        for j in start..rows {
            Self::smooth_row_cols(&mut self.height_rows[j]);
        }
        // Smooth across rows within the tail region for each column
        if rows - start >= 3 {
            let cols = self.grid_cols;
            let src = self.height_rows.clone();
            for j in start..rows {
                for i in 0..cols {
                    let a = if j>start { src[j-1][i] } else { src[j][i] };
                    let b = src[j][i];
                    let c = if j+1<rows { src[j+1][i] } else { src[j][i] };
                    self.height_rows[j][i] = (a + 2.0*b + c) * 0.25;
                }
            }
        }
    }



    #[inline]
    fn project(&self, focal_len: f32, horizon_y: f32, x: f32, y: f32, z: f32, cx: f32, dot_w: usize, dot_h: usize) -> Option<(usize, usize)> {
        if z <= self.near_plane || z > self.far_plane { return None; }
        let f = focal_len;
        let sx = cx + f * x / z;
        let sy = horizon_y - f * y / z;
        if sx.is_finite() && sy.is_finite() {
            let ix = sx.round() as i32;
            let iy = sy.round() as i32;
            if ix >= 0 && iy >= 0 && (ix as usize) < dot_w && (iy as usize) < dot_h {
                return Some((ix as usize, iy as usize));
            }
        }
        None
    }

    #[inline]
    fn green_for_depth(&self, z: f32) -> Color {
        // Near is brighter, far is dimmer (no audio shimmer)
        let zn = ((self.far_plane - z) / (self.far_plane - self.near_plane)).clamp(0.0, 1.0);
        let g = (90.0 + 165.0 * (0.4 + 0.6 * zn)).clamp(0.0, 255.0) as u8;
        Color::new(0, g, 0)
    }
}

impl Visualizer for TerrainLandscapeVisualizer {
    fn update(&mut self, _params: &AudioParameters) {
        // Disconnected from audio: decay all audio values toward zero
        let k = 0.2;
        self.bass = lerp(self.bass, 0.0, k);
        self.mid = lerp(self.mid, 0.0, k);
        self.treble = lerp(self.treble, 0.0, k);
        self.amplitude = lerp(self.amplitude, 0.0, k);

        // No beat pulses
        self.beat_pulse = lerp(self.beat_pulse, 0.0, 0.22);

        // Smooth, slowed forward motion (continuous; no row shifting)
        self.scroll_z += self.base_speed;

        // Gentle evolution baseline
        self.phase += 0.006; // slower evolution
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let width = grid.width();
        let height = grid.height();
        let mut braille = BrailleGrid::new(width, height);
        let dot_w = braille.dot_width();
        let dot_h = braille.dot_height();
        let cx = (dot_w as f32) * 0.5;

        // Camera/Projection tuning derived from current resolution
        let fov_scale = 0.55; // 0.45..0.70 looks good
        let horizon_ratio = 0.40; // fraction of height from top
        let mut focal = (dot_w as f32) * fov_scale;
        if focal < 80.0 { focal = 80.0; }

        let horizon_y = (dot_h as f32) * horizon_ratio;

        // Row depth loop: j indexes rows into the screen
        // Use scroll_z as a sub-cell offset so the grid advances smoothly
        let cols = self.grid_cols;
        let rows = self.grid_rows;

        // Dynamic horizontal scale so mid-depth endpoints reach screen edges
        let base_x_max = self.xs_base.last().copied().unwrap_or(1.0).abs().max(0.001);
        let j_ref = ((rows as f32) * 0.33) as usize;
        let z_ref = self.near_plane + (j_ref as f32) * self.spacing_z;
        let mut scale_s = (((dot_w as f32 - 1.0) - cx) * z_ref) / (focal * base_x_max);
        if !scale_s.is_finite() || scale_s <= 0.0 { scale_s = 1.0; }

        // Push X sampling slightly off-screen so edge segments keep deformation
        let edge_pad = 0.12; // 12% offscreen
        scale_s *= 1.0 + edge_pad;

        // Cache projected points for connecting lines
        let mut proj: Vec<Vec<Option<(usize, usize, f32)>>> = vec![vec![None; cols]; rows];

        // Compute z range so we cover near->far with uniform spacing
        // Camera moves forward by increasing scroll_z; sample world at z_world = scroll_z + z_cam
        for j in 0..rows {
            // camera-space depth
            let z_cam = self.near_plane + (j as f32) * self.spacing_z;
            if z_cam > self.far_plane { continue; }

            let z_world = self.scroll_z + z_cam;

            for i in 0..cols {
                let x_noise = self.xs_base[i];
                let x_proj = self.xs_base[i] * scale_s;
                let yh = self.height_fn(x_noise, z_world);
                let yw = -16.0 + yh;

                // Project with dynamic focal & horizon
                let p = self.project(focal, horizon_y, x_proj, yw, z_cam, cx, dot_w, dot_h);
                if let Some((px, py)) = p {
                    proj[j][i] = Some((px, py, z_cam));
                }
            }
        }

        // Draw grid lines: lateral (i to i+1) first, then depth (j to j+1)
        // This preserves strong vertical-in-depth lines visually
        for j in 0..rows {
            for i in 0..cols {
                if let Some((x0, y0, z0)) = proj[j][i] {
                    // Connect sideways (lateral)
                    if i + 1 < cols {
                        if let Some((x1, y1, z1)) = proj[j][i + 1] {
                            let z_avg = 0.5 * (z0 + z1);
                            let color = self.green_for_depth(z_avg);
                            braille.draw_line_with_color(x0, y0, x1, y1, color);
                        }
                    }
                    // Connect forward (depth)
                    if j + 1 < rows {
                        if let Some((x1, y1, z1)) = proj[j + 1][i] {
                            let z_avg = 0.5 * (z0 + z1);
                            let color = self.green_for_depth(z_avg);
                            braille.draw_line_with_color(x0, y0, x1, y1, color);
                        }
                    }
                }
            }
        }

        // HUD overlay: sci‑fi probe scanning UI
        let elapsed = self.start_time.elapsed().as_secs_f32();
        // Keep HUD fully green
        let hud_cyan = Color::new(0, 220, 0);
        let hud_dim  = Color::new(0, 150, 0);
        let cy = ((dot_h as f32) * 0.52).clamp(0.0, (dot_h - 1) as f32) as usize;
        // Minimal center reticle: three disconnected green lines forming a triangle
        let tri_s = ((dot_h as f32) * 0.035).max(2.0) as i32;
        let ax = cx as i32;             let ay = cy as i32 - tri_s;
        let bx = cx as i32 - tri_s;     let by = cy as i32 + tri_s;
        let cxv = cx as i32 + tri_s;    let cyv = cy as i32 + tri_s;
        // Shorten each side toward its center to leave small gaps at the corners
        let gap = (tri_s as f32 * 0.38).max(1.0);
        // AB shortened
        let (dxab, dyab) = ((bx - ax) as f32, (by - ay) as f32);
        let lenab = (dxab*dxab + dyab*dyab).sqrt().max(1.0);
        let ab_ax = (ax as f32 + dxab * (gap/lenab)).round() as i32;
        let ab_ay = (ay as f32 + dyab * (gap/lenab)).round() as i32;
        let ab_bx = (bx as f32 - dxab * (gap/lenab)).round() as i32;
        let ab_by = (by as f32 - dyab * (gap/lenab)).round() as i32;
        // BC shortened
        let (dxbc, dybc) = ((cxv - bx) as f32, (cyv - by) as f32);
        let lenbc = (dxbc*dxbc + dybc*dybc).sqrt().max(1.0);
        let bc_bx = (bx as f32 + dxbc * (gap/lenbc)).round() as i32;
        let bc_by = (by as f32 + dybc * (gap/lenbc)).round() as i32;
        let bc_cx = (cxv as f32 - dxbc * (gap/lenbc)).round() as i32;
        let bc_cy = (cyv as f32 - dybc * (gap/lenbc)).round() as i32;
        // CA shortened
        let (dxca, dyca) = ((ax - cxv) as f32, (ay - cyv) as f32);
        let lenca = (dxca*dxca + dyca*dyca).sqrt().max(1.0);
        let ca_cx = (cxv as f32 + dxca * (gap/lenca)).round() as i32;
        let ca_cy = (cyv as f32 + dyca * (gap/lenca)).round() as i32;
        let ca_ax = (ax as f32 - dxca * (gap/lenca)).round() as i32;
        let ca_ay = (ay as f32 - dyca * (gap/lenca)).round() as i32;
        // Clamp to canvas
        let clamp_x = |v: i32| v.clamp(0, (dot_w - 1) as i32) as usize;
        let clamp_y = |v: i32| v.clamp(0, (dot_h - 1) as i32) as usize;
        // Draw three independent sides (all green)
        braille.draw_line_with_color(clamp_x(ab_ax), clamp_y(ab_ay), clamp_x(ab_bx), clamp_y(ab_by), hud_cyan);
        braille.draw_line_with_color(clamp_x(bc_bx), clamp_y(bc_by), clamp_x(bc_cx), clamp_y(bc_cy), hud_cyan);
        braille.draw_line_with_color(clamp_x(ca_cx), clamp_y(ca_cy), clamp_x(ca_ax), clamp_y(ca_ay), hud_cyan);


        // Blit braille grid back to character grid
        for cell_y in 0..height {
            for cell_x in 0..width {
                let ch = braille.get_char(cell_x, cell_y);
                if ch != ' ' {
                    let color = braille.get_color(cell_x, cell_y);
                    let cell = grid.get_cell_mut(cell_x, cell_y);
                    cell.character = ch;
                    cell.foreground_color = color;
                }
            }
        }

        // Telemetry text overlays (designation + periodic elements scan)
        let mut write_text = |x: usize, y: usize, s: &str, color: Color| {
            if y < height {
                let mut xi = x;
                for ch in s.chars() {
                    if xi >= width { break; }
                    let cell = grid.get_cell_mut(xi, y);
                    cell.character = ch;
                    cell.foreground_color = Some(color);
                    xi += 1;
                }
            }
        };
        let tick = (elapsed * 0.22).floor() as u32; // slow transitions
        let h = hash_u32(tick.wrapping_mul(0x9E3779B9));
        let letter = |k: u32| -> char { (b'A' + ((h.wrapping_add(k) % 26) as u8)) as char };
        let desig = format!("DESIG {}{}-{:02X}", letter(0), letter(1), ((h >> 12) & 0xFF));
        let sector = format!("SECTOR {:02}-{}", ((h >> 20) & 0x3F), letter(8));
        write_text(2, 1, &desig, hud_dim);
        write_text(2, 2, &sector, hud_dim);
        let elems = ["H","He","C","N","O","Ne","Na","Mg","Si","P","S","Cl","K","Ca","Ti","V","Cr","Mn","Fe","Co","Ni","Cu","Zn","Ag","Sn","I","Xe","Cs","Ba","W","Pt","Au","Hg","Pb","U"];
        let pick = |off: u32| -> &str { let idx = (hash_u32(h.wrapping_add(off)) % (elems.len() as u32)) as usize; elems[idx] };
        let per_short = format!("ELEM: {} {} {}", pick(1), pick(7), pick(13));
        let snr = (hash_u32(h ^ 0x55AA) % 300) as f32 / 10.0;
        let conf = (hash_u32(h ^ 0xCC33) % 100) as f32 / 100.0;

        // Saturn system scan: target and trajectory readouts
        let moons = [
            "TITAN","ENCELADUS","RHEA","IAPETUS","DIONE","TETHYS","MIMAS","HYPERION","PHOEBE","JANUS"
        ];
        let moon_idx = (hash_u32(h ^ 0xA5A5_5A5A) % (moons.len() as u32)) as usize;
        let target = moons[moon_idx];
        let phase_str = if ((h >> 7) & 1) == 0 { "INBOUND" } else { "OUTBOUND" };
        let apo_km = 8.0e5 + ((h & 0x3FFF) as f32) * 35.0;   // ~0.8M .. ~1.3M km
        let peri_km = 5.0e5 + (((h >> 10) & 0x3FFF) as f32) * 28.0; // ~0.5M .. ~0.9M km
        let inc_deg = 3.0 + (((h >> 18) & 0x7FF) as f32) * 0.02;     // ~3..25 deg
        let eta_s = 60 + ((h >> 4) % 600) as usize; // 1..11 minutes
        let eta_m = eta_s / 60; let eta_r = eta_s % 60;

        // compute local right column anchor (duplicate of below, to place extended lines)
        let col_w2 = 18usize;
        let right_x2 = if width > col_w2 + 2 { width - col_w2 - 2 } else { width.saturating_sub(2) };
        let right_y02 = ((height as f32) * 0.25) as usize;
        write_text(right_x2, right_y02 + 4, &format!("TARGET {}", target), hud_cyan);
        write_text(right_x2, right_y02 + 5, &format!("PHASE {}", phase_str), hud_dim);
        write_text(right_x2, right_y02 + 6, &format!("APO {:>4.0}k", apo_km/1000.0), hud_dim);
        write_text(right_x2, right_y02 + 7, &format!("PERI {:>4.0}k", peri_km/1000.0), hud_dim);
        write_text(right_x2, right_y02 + 8, &format!("INC {:>4.1}°", inc_deg), hud_dim);
        write_text(right_x2, right_y02 + 9, &format!("ETA {:02}:{:02}", eta_m, eta_r), hud_dim);

        // Subtle references to unnamed/provisional small moons (dim)
        let years = [2004u32, 2006, 2007, 2009, 2019, 2020];
        let yr_pick = |seed: u32| -> u32 {
            let idx = (hash_u32(h ^ seed) % (years.len() as u32)) as usize;
            years[idx]
        };
        let num_pick = |seed: u32| -> u32 { 1 + (hash_u32(h ^ seed) % 27) };
        let u1 = format!("S/{} S {}", yr_pick(0x11AA), num_pick(0x22BB));
        let u2 = format!("S/{} S {}", yr_pick(0x33CC), num_pick(0x44DD));
        let u3 = format!("S/{} S {}", yr_pick(0x55EE), num_pick(0x66FF));
        let others = format!("OTHERS: {}  {}  {}", u1, u2, u3);
        write_text(right_x2, right_y02 + 10, &others, hud_dim);

        // Trajectory arc (bottom-right quadrant) + moving marker
        let arc_cx = (dot_w as f32 * 0.78) as i32;
        let arc_cy = (dot_h as f32 * 0.78) as i32;
        let rx = (dot_w as f32 * 0.14).max(8.0) as i32;
        let ry = (dot_h as f32 * 0.09).max(6.0) as i32;
        let a0 = -2.6f32; // start angle
        let a1 =  0.6f32; // end angle
        // subtle static micro-markers along the arc (dim), hinting at unnamed bodies
        let mk_t1 = ((hash_u32(h ^ 0x1357) % 40) as f32) / 55.0;
        let mk_t2 = ((hash_u32(h ^ 0x2468) % 40 + 10) as f32) / 55.0;
        for t in [mk_t1, mk_t2] {
            let a = a0 + (a1 - a0) * t;
            let x = (arc_cx as f32 + (rx as f32) * a.cos()).round() as i32;
            let y = (arc_cy as f32 + (ry as f32) * a.sin()).round() as i32;
            let xu = x.clamp(0, (dot_w - 1) as i32) as usize;
            let yu = y.clamp(0, (dot_h - 1) as i32) as usize;
            braille.draw_circle(xu, yu, 1, hud_dim);
        }

        let mut px = 0usize; let mut py = 0usize; let mut has_prev = false;
        for step in 0..56 {
            let t = step as f32 / 55.0;
            let a = a0 + (a1 - a0) * t;
            let x = (arc_cx as f32 + (rx as f32) * a.cos()).round() as i32;
            let y = (arc_cy as f32 + (ry as f32) * a.sin()).round() as i32;
            let xu = x.clamp(0, (dot_w - 1) as i32) as usize;
            let yu = y.clamp(0, (dot_h - 1) as i32) as usize;
            if has_prev { braille.draw_line_with_color(px, py, xu, yu, hud_dim); }
            px = xu; py = yu; has_prev = true;
        }
        // moving marker along the arc
        let prog = ((elapsed * 0.015) % 1.0) as f32; // very slow
        let am = a0 + (a1 - a0) * prog;
        let mx = (arc_cx as f32 + (rx as f32) * am.cos()).round().clamp(0.0, (dot_w - 1) as f32) as usize;
        let my = (arc_cy as f32 + (ry as f32) * am.sin()).round().clamp(0.0, (dot_h - 1) as f32) as usize;
        braille.draw_circle(mx, my, 1, hud_cyan);

        // Side columns (vertical state)
        let left_x = 2usize;
        let left_y0 = ((height as f32) * 0.25) as usize;
        write_text(left_x, left_y0.saturating_sub(1), "NAV", hud_dim);
        let z_sample = self.scroll_z + 12.0;
        let alt = self.height_fn(0.0, z_sample);
        write_text(left_x, left_y0 + 0, &format!("VEL {:>5.2}", self.base_speed), hud_dim);
        write_text(left_x, left_y0 + 1, &format!("ALT {:>5.2}", alt), hud_dim);
        write_text(left_x, left_y0 + 2, &format!("ZPOS {:>6.1}", self.scroll_z), hud_dim);

        let col_w = 18usize;
        let right_x = if width > col_w + 2 { width - col_w - 2 } else { width.saturating_sub(2) };
        let right_y0 = left_y0;
        write_text(right_x, right_y0.saturating_sub(1), "SCAN", hud_dim);
        write_text(right_x, right_y0 + 0, &per_short, hud_cyan);
        write_text(right_x, right_y0 + 1, &format!("SNR  {:>4.1} dB", snr), hud_dim);
        write_text(right_x, right_y0 + 2, &format!("CONF {:>4.2}", conf), hud_dim);
        write_text(right_x, right_y0 + 3, "MODE PROSPECT", hud_dim);

    }

    fn name(&self) -> &str {
        "Green Grid Landscape"
    }
}



// --- Simple 2D value noise and fBm helpers (no external deps) ---
#[inline]
fn smoothstep01(t: f32) -> f32 { let t = t.clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }

#[inline]
fn hash_u32(mut x: u32) -> u32 {
    // Mix bits (Thomas Wang-ish integer hash)
    x ^= x >> 16; x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15; x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16; x
}

#[inline]
fn rand01_i32(xi: i32, zi: i32) -> f32 {
    let mut h = 0x9E3779B9u32; // golden ratio fractional
    h ^= (xi as u32).wrapping_mul(0x94D049BB);
    h = h.rotate_left(13) ^ (zi as u32).wrapping_mul(0x5F356495);
    let v = hash_u32(h);
    (v as f32) / (u32::MAX as f32)
}

#[inline]
fn value_noise2(x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32; let z0 = z.floor() as i32;
    let x1 = x0 + 1;         let z1 = z0 + 1;
    let fx = x - x0 as f32;  let fz = z - z0 as f32;
    let ux = smoothstep01(fx); let uz = smoothstep01(fz);

    let v00 = rand01_i32(x0, z0);
    let v10 = rand01_i32(x1, z0);
    let v01 = rand01_i32(x0, z1);
    let v11 = rand01_i32(x1, z1);

    let a = lerp(v00, v10, ux);
    let b = lerp(v01, v11, ux);
    let r01 = lerp(a, b, uz);
    r01 * 2.0 - 1.0 // to [-1,1]
}

#[inline]
fn fbm(x: f32, z: f32) -> f32 {
    let mut sum = 0.0; let mut amp = 1.0; let mut freq = 1.0; let mut norm = 0.0;
    for _ in 0..4 { // 4 octaves
        sum += value_noise2(x * freq, z * freq) * amp;
        norm += amp; amp *= 0.5; freq *= 2.0;
    }
    sum / norm
}
