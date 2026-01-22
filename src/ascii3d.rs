//! 3D ASCII Shader Renderer
//!
//! Renders 3D geometry as ASCII art using luminosity-based character mapping.
//! Inspired by the ASCII shader technique that converts brightness values to
//! characters of varying visual density.
//!
//! # Character Density Scale (dark to light)
//! `@%#*+=-:. ` - from densest (darkest) to sparsest (brightest)

use std::f32::consts::PI;

// ============================================================================
// ASCII Character Palette
// ============================================================================

/// Character palette ordered by visual density (dark to light)
/// Each character has a different "fill" percentage when rendered
pub const ASCII_PALETTE: &[char] = &[
    '@', // ~90% fill - darkest
    '#', // ~80% fill
    '%', // ~70% fill
    '&', // ~65% fill
    '*', // ~50% fill
    '+', // ~40% fill
    '=', // ~35% fill
    '-', // ~25% fill
    ':', // ~15% fill
    '.', // ~10% fill
    ' ', // ~0% fill - brightest (background)
];

/// Extended palette with more gradation for smoother shading
pub const ASCII_PALETTE_EXTENDED: &[char] = &[
    '@', '$', '#', '%', '&', '?', '*', '+', ';', ':', ',', '.', ' ',
];

/// Map a luminosity value (0.0 = dark, 1.0 = bright) to an ASCII character
pub fn luminosity_to_char(luminosity: f32) -> char {
    let clamped = luminosity.clamp(0.0, 1.0);
    let index = (clamped * (ASCII_PALETTE.len() - 1) as f32).round() as usize;
    ASCII_PALETTE[index.min(ASCII_PALETTE.len() - 1)]
}

// ============================================================================
// 3D Math Primitives
// ============================================================================

/// 3D Vector
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const FORWARD: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 0.0001 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Vec3::ZERO
        }
    }

    pub fn scale(self, s: f32) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

/// 4x4 Transformation Matrix (column-major)
#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [c, 0.0, s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0],
            ],
        }
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        Self {
            m: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, sz, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Perspective projection matrix
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov / 2.0).tan();
        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) / (near - far), -1.0],
                [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
            ],
        }
    }

    /// Orthographic projection (simpler for ASCII art)
    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            m: [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / height, 0.0, 0.0],
                [0.0, 0.0, -2.0 / (far - near), 0.0],
                [0.0, 0.0, -(far + near) / (far - near), 1.0],
            ],
        }
    }

    pub fn multiply(self, other: Mat4) -> Mat4 {
        let mut result = Mat4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = 0.0;
                for k in 0..4 {
                    result.m[i][j] += self.m[k][j] * other.m[i][k];
                }
            }
        }
        result
    }

    pub fn transform_point(self, p: Vec3) -> Vec3 {
        let w = self.m[0][3] * p.x + self.m[1][3] * p.y + self.m[2][3] * p.z + self.m[3][3];
        Vec3 {
            x: (self.m[0][0] * p.x + self.m[1][0] * p.y + self.m[2][0] * p.z + self.m[3][0]) / w,
            y: (self.m[0][1] * p.x + self.m[1][1] * p.y + self.m[2][1] * p.z + self.m[3][1]) / w,
            z: (self.m[0][2] * p.x + self.m[1][2] * p.y + self.m[2][2] * p.z + self.m[3][2]) / w,
        }
    }

    pub fn transform_normal(self, n: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0][0] * n.x + self.m[1][0] * n.y + self.m[2][0] * n.z,
            y: self.m[0][1] * n.x + self.m[1][1] * n.y + self.m[2][1] * n.z,
            z: self.m[0][2] * n.x + self.m[1][2] * n.y + self.m[2][2] * n.z,
        }
        .normalize()
    }
}

// ============================================================================
// 3D Mesh Definition
// ============================================================================

/// A triangle face with vertex indices and a normal
#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
    pub normal: Vec3,
}

/// A simple 3D mesh
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl Mesh {
    /// Create a unit cube centered at origin
    pub fn cube() -> Self {
        let vertices = vec![
            Vec3::new(-0.5, -0.5, -0.5), // 0: back-bottom-left
            Vec3::new(0.5, -0.5, -0.5),  // 1: back-bottom-right
            Vec3::new(0.5, 0.5, -0.5),   // 2: back-top-right
            Vec3::new(-0.5, 0.5, -0.5),  // 3: back-top-left
            Vec3::new(-0.5, -0.5, 0.5),  // 4: front-bottom-left
            Vec3::new(0.5, -0.5, 0.5),   // 5: front-bottom-right
            Vec3::new(0.5, 0.5, 0.5),    // 6: front-top-right
            Vec3::new(-0.5, 0.5, 0.5),   // 7: front-top-left
        ];

        let faces = vec![
            // Front face
            Face { v0: 4, v1: 5, v2: 6, normal: Vec3::new(0.0, 0.0, 1.0) },
            Face { v0: 4, v1: 6, v2: 7, normal: Vec3::new(0.0, 0.0, 1.0) },
            // Back face
            Face { v0: 1, v1: 0, v2: 3, normal: Vec3::new(0.0, 0.0, -1.0) },
            Face { v0: 1, v1: 3, v2: 2, normal: Vec3::new(0.0, 0.0, -1.0) },
            // Top face
            Face { v0: 7, v1: 6, v2: 2, normal: Vec3::new(0.0, 1.0, 0.0) },
            Face { v0: 7, v1: 2, v2: 3, normal: Vec3::new(0.0, 1.0, 0.0) },
            // Bottom face
            Face { v0: 0, v1: 1, v2: 5, normal: Vec3::new(0.0, -1.0, 0.0) },
            Face { v0: 0, v1: 5, v2: 4, normal: Vec3::new(0.0, -1.0, 0.0) },
            // Right face
            Face { v0: 5, v1: 1, v2: 2, normal: Vec3::new(1.0, 0.0, 0.0) },
            Face { v0: 5, v1: 2, v2: 6, normal: Vec3::new(1.0, 0.0, 0.0) },
            // Left face
            Face { v0: 0, v1: 4, v2: 7, normal: Vec3::new(-1.0, 0.0, 0.0) },
            Face { v0: 0, v1: 7, v2: 3, normal: Vec3::new(-1.0, 0.0, 0.0) },
        ];

        Self { vertices, faces }
    }

    /// Create a sphere using UV subdivision
    pub fn sphere(segments: usize, rings: usize) -> Self {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        // Generate vertices
        for j in 0..=rings {
            let theta = PI * j as f32 / rings as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for i in 0..=segments {
                let phi = 2.0 * PI * i as f32 / segments as f32;
                let x = sin_theta * phi.cos();
                let y = cos_theta;
                let z = sin_theta * phi.sin();
                vertices.push(Vec3::new(x * 0.5, y * 0.5, z * 0.5));
            }
        }

        // Generate faces
        for j in 0..rings {
            for i in 0..segments {
                let v0 = j * (segments + 1) + i;
                let v1 = v0 + 1;
                let v2 = v0 + segments + 1;
                let v3 = v2 + 1;

                // Calculate face normal (average of vertex normals for sphere)
                let center = vertices[v0].add(vertices[v1]).add(vertices[v2]).scale(1.0 / 3.0);
                let normal = center.normalize().scale(2.0); // Point outward

                faces.push(Face { v0, v1: v2, v2: v1, normal });

                let center2 = vertices[v1].add(vertices[v2]).add(vertices[v3]).scale(1.0 / 3.0);
                let normal2 = center2.normalize().scale(2.0);

                faces.push(Face { v0: v1, v1: v2, v2: v3, normal: normal2 });
            }
        }

        Self { vertices, faces }
    }

    /// Create a cute character mesh (simplified humanoid mascot)
    pub fn character() -> Self {
        let mut all_vertices = Vec::new();
        let mut all_faces = Vec::new();

        // Helper to add a transformed sphere to the mesh
        let add_sphere = |verts: &mut Vec<Vec3>, faces: &mut Vec<Face>,
                          center: Vec3, scale: Vec3, segments: usize, rings: usize| {
            let offset = verts.len();
            let sphere = Mesh::sphere(segments, rings);

            for v in &sphere.vertices {
                verts.push(Vec3::new(
                    v.x * scale.x + center.x,
                    v.y * scale.y + center.y,
                    v.z * scale.z + center.z,
                ));
            }

            for f in &sphere.faces {
                faces.push(Face {
                    v0: f.v0 + offset,
                    v1: f.v1 + offset,
                    v2: f.v2 + offset,
                    normal: f.normal,
                });
            }
        };

        // Head (large sphere)
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.0, 0.6, 0.0),
                   Vec3::new(0.8, 0.9, 0.7),
                   12, 8);

        // Body (slightly smaller sphere)
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.0, -0.2, 0.0),
                   Vec3::new(0.6, 0.7, 0.5),
                   10, 6);

        // Left eye (small sphere, positioned forward)
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(-0.2, 0.7, 0.3),
                   Vec3::new(0.15, 0.15, 0.1),
                   6, 4);

        // Right eye
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.2, 0.7, 0.3),
                   Vec3::new(0.15, 0.15, 0.1),
                   6, 4);

        // Nose (tiny sphere)
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.0, 0.5, 0.4),
                   Vec3::new(0.08, 0.08, 0.08),
                   4, 3);

        // Left ear (small sphere on top-left of head)
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(-0.35, 1.1, 0.0),
                   Vec3::new(0.15, 0.25, 0.12),
                   6, 4);

        // Right ear
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.35, 1.1, 0.0),
                   Vec3::new(0.15, 0.25, 0.12),
                   6, 4);

        // Left arm
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(-0.5, -0.1, 0.0),
                   Vec3::new(0.15, 0.35, 0.15),
                   6, 4);

        // Right arm
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.5, -0.1, 0.0),
                   Vec3::new(0.15, 0.35, 0.15),
                   6, 4);

        // Left foot
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(-0.2, -0.7, 0.1),
                   Vec3::new(0.18, 0.15, 0.22),
                   6, 4);

        // Right foot
        add_sphere(&mut all_vertices, &mut all_faces,
                   Vec3::new(0.2, -0.7, 0.1),
                   Vec3::new(0.18, 0.15, 0.22),
                   6, 4);

        Self {
            vertices: all_vertices,
            faces: all_faces,
        }
    }
}

// ============================================================================
// ASCII Framebuffer & Renderer
// ============================================================================

/// A 2D framebuffer for ASCII rendering
pub struct AsciiFramebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<f32>,      // Luminosity buffer (0.0 = dark, 1.0 = bright)
    pub depth: Vec<f32>,       // Z-buffer for depth testing
}

impl AsciiFramebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![1.0; width * height], // Start with bright (background)
            depth: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(1.0);
        self.depth.fill(f32::INFINITY);
    }

    /// Set a pixel with depth testing
    pub fn set_pixel(&mut self, x: i32, y: i32, z: f32, luminosity: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        if z < self.depth[idx] {
            self.depth[idx] = z;
            self.pixels[idx] = luminosity;
        }
    }

    /// Convert framebuffer to ASCII string
    pub fn to_ascii(&self) -> String {
        let mut result = String::with_capacity(self.width * self.height + self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let c = luminosity_to_char(self.pixels[idx]);
                result.push(c);
            }
            result.push('\n');
        }
        result
    }
}

/// 3D ASCII Renderer
pub struct Ascii3DRenderer {
    pub framebuffer: AsciiFramebuffer,
    pub light_dir: Vec3,
    pub ambient: f32,
}

impl Ascii3DRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            framebuffer: AsciiFramebuffer::new(width, height),
            light_dir: Vec3::new(0.3, 0.5, 0.8).normalize(),
            ambient: 0.2,
        }
    }

    /// Render a mesh with the given transformation
    pub fn render(&mut self, mesh: &Mesh, transform: Mat4) {
        let width = self.framebuffer.width as f32;
        let height = self.framebuffer.height as f32;

        // Aspect ratio correction for terminal characters (typically 2:1 height:width)
        let aspect = (width / height) * 0.5;

        // Projection matrix (orthographic for cleaner ASCII look)
        let projection = Mat4::orthographic(2.5 * aspect, 2.5, 0.1, 100.0);
        let view = Mat4::translation(0.0, 0.0, -3.0);
        let mvp = projection.multiply(view).multiply(transform);

        for face in &mesh.faces {
            // Get transformed vertices
            let v0 = mvp.transform_point(mesh.vertices[face.v0]);
            let v1 = mvp.transform_point(mesh.vertices[face.v1]);
            let v2 = mvp.transform_point(mesh.vertices[face.v2]);

            // Transform normal for lighting
            let normal = transform.transform_normal(face.normal);

            // Simple diffuse lighting
            let ndotl = normal.dot(self.light_dir).max(0.0);
            let luminosity = 1.0 - (self.ambient + (1.0 - self.ambient) * ndotl);

            // Backface culling (optional, helps performance)
            let edge1 = v1.sub(v0);
            let edge2 = v2.sub(v0);
            let cross_z = edge1.x * edge2.y - edge1.y * edge2.x;
            if cross_z > 0.0 {
                continue; // Face is facing away
            }

            // Convert to screen coordinates
            let screen_v0 = self.to_screen(v0);
            let screen_v1 = self.to_screen(v1);
            let screen_v2 = self.to_screen(v2);

            // Rasterize the triangle
            self.rasterize_triangle(
                screen_v0, v0.z,
                screen_v1, v1.z,
                screen_v2, v2.z,
                luminosity,
            );
        }
    }

    fn to_screen(&self, v: Vec3) -> (i32, i32) {
        let x = ((v.x + 1.0) * 0.5 * self.framebuffer.width as f32) as i32;
        let y = ((1.0 - v.y) * 0.5 * self.framebuffer.height as f32) as i32;
        (x, y)
    }

    /// Rasterize a triangle using scanline algorithm
    fn rasterize_triangle(
        &mut self,
        v0: (i32, i32), z0: f32,
        v1: (i32, i32), z1: f32,
        v2: (i32, i32), z2: f32,
        luminosity: f32,
    ) {
        // Find bounding box
        let min_x = v0.0.min(v1.0).min(v2.0).max(0);
        let max_x = v0.0.max(v1.0).max(v2.0).min(self.framebuffer.width as i32 - 1);
        let min_y = v0.1.min(v1.1).min(v2.1).max(0);
        let max_y = v0.1.max(v1.1).max(v2.1).min(self.framebuffer.height as i32 - 1);

        // Edge function for point-in-triangle test
        let edge = |a: (i32, i32), b: (i32, i32), p: (i32, i32)| -> i32 {
            (p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0)
        };

        let area = edge(v0, v1, v2);
        if area == 0 {
            return; // Degenerate triangle
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = (x, y);
                let w0 = edge(v1, v2, p);
                let w1 = edge(v2, v0, p);
                let w2 = edge(v0, v1, p);

                // Check if point is inside triangle (same sign for all edges)
                if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                    // Interpolate depth
                    let area_f = area as f32;
                    let z = (w0 as f32 * z0 + w1 as f32 * z1 + w2 as f32 * z2) / area_f;

                    self.framebuffer.set_pixel(x, y, z, luminosity);
                }
            }
        }
    }
}

// ============================================================================
// Animated Character
// ============================================================================

/// Expression states for the character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterExpression {
    Neutral,
    Happy,
    Thinking,
    Surprised,
    Sad,
}

/// Animated 3D character for CLI display
pub struct AnimatedCharacter {
    mesh: Mesh,
    rotation_y: f32,
    rotation_x: f32,
    expression: CharacterExpression,
    bob_phase: f32,
}

impl Default for AnimatedCharacter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimatedCharacter {
    pub fn new() -> Self {
        Self {
            mesh: Mesh::character(),
            rotation_y: 0.0,
            rotation_x: 0.0,
            expression: CharacterExpression::Neutral,
            bob_phase: 0.0,
        }
    }

    pub fn set_expression(&mut self, expression: CharacterExpression) {
        self.expression = expression;
    }

    pub fn set_rotation(&mut self, x: f32, y: f32) {
        self.rotation_x = x;
        self.rotation_y = y;
    }

    /// Update animation (call each frame)
    pub fn tick(&mut self, delta_time: f32) {
        // Gentle bobbing animation
        self.bob_phase += delta_time * 2.0;
        if self.bob_phase > 2.0 * PI {
            self.bob_phase -= 2.0 * PI;
        }
    }

    /// Render the character to an ASCII string
    pub fn render(&self, width: usize, height: usize) -> String {
        let mut renderer = Ascii3DRenderer::new(width, height);
        renderer.framebuffer.clear();

        // Build transformation matrix
        let bob_offset = self.bob_phase.sin() * 0.05;
        let transform = Mat4::translation(0.0, bob_offset, 0.0)
            .multiply(Mat4::rotation_y(self.rotation_y))
            .multiply(Mat4::rotation_x(self.rotation_x));

        renderer.render(&self.mesh, transform);
        renderer.framebuffer.to_ascii()
    }

    /// Render with a specific rotation angle (convenience method)
    pub fn render_at_angle(&self, width: usize, height: usize, angle_y: f32) -> String {
        let mut renderer = Ascii3DRenderer::new(width, height);
        renderer.framebuffer.clear();

        let bob_offset = self.bob_phase.sin() * 0.05;
        let transform = Mat4::translation(0.0, bob_offset, 0.0)
            .multiply(Mat4::rotation_y(angle_y))
            .multiply(Mat4::rotation_x(self.rotation_x));

        renderer.render(&self.mesh, transform);
        renderer.framebuffer.to_ascii()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Render a spinning 3D character at the given frame
pub fn render_character_frame(width: usize, height: usize, frame: usize) -> String {
    let mut character = AnimatedCharacter::new();
    let angle = (frame as f32 * 0.1) % (2.0 * PI);
    character.set_rotation(-0.2, angle);
    character.tick(0.1);
    character.render(width, height)
}

/// Render a 3D cube at the given rotation
pub fn render_cube(width: usize, height: usize, angle_y: f32, angle_x: f32) -> String {
    let mut renderer = Ascii3DRenderer::new(width, height);
    renderer.framebuffer.clear();

    let mesh = Mesh::cube();
    let transform = Mat4::rotation_y(angle_y).multiply(Mat4::rotation_x(angle_x));

    renderer.render(&mesh, transform);
    renderer.framebuffer.to_ascii()
}

/// Render a 3D sphere
pub fn render_sphere(width: usize, height: usize, angle_y: f32) -> String {
    let mut renderer = Ascii3DRenderer::new(width, height);
    renderer.framebuffer.clear();

    let mesh = Mesh::sphere(16, 12);
    let transform = Mat4::rotation_y(angle_y);

    renderer.render(&mesh, transform);
    renderer.framebuffer.to_ascii()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);

        assert!((a.dot(b) - 0.0).abs() < 0.001);
        assert!((a.length() - 1.0).abs() < 0.001);

        let cross = a.cross(b);
        assert!((cross.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_luminosity_to_char() {
        assert_eq!(luminosity_to_char(0.0), '@'); // Darkest
        assert_eq!(luminosity_to_char(1.0), ' '); // Brightest
    }

    #[test]
    fn test_framebuffer_creation() {
        let fb = AsciiFramebuffer::new(40, 20);
        assert_eq!(fb.width, 40);
        assert_eq!(fb.height, 20);
        assert_eq!(fb.pixels.len(), 800);
    }

    #[test]
    fn test_cube_mesh() {
        let cube = Mesh::cube();
        assert_eq!(cube.vertices.len(), 8);
        assert_eq!(cube.faces.len(), 12);
    }

    #[test]
    fn test_sphere_mesh() {
        let sphere = Mesh::sphere(8, 6);
        assert!(!sphere.vertices.is_empty());
        assert!(!sphere.faces.is_empty());
    }

    #[test]
    fn test_character_mesh() {
        let character = Mesh::character();
        assert!(!character.vertices.is_empty());
        assert!(!character.faces.is_empty());
    }

    #[test]
    fn test_render_character() {
        let result = render_character_frame(30, 15, 0);
        assert!(!result.is_empty());
        assert!(result.contains('\n'));
    }

    #[test]
    fn test_render_cube() {
        let result = render_cube(20, 10, 0.5, 0.3);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_animated_character() {
        let mut char = AnimatedCharacter::new();
        char.tick(0.1);
        char.set_expression(CharacterExpression::Happy);
        let rendered = char.render(25, 15);
        assert!(!rendered.is_empty());
    }

    #[test]
    fn test_matrix_operations() {
        let m = Mat4::identity();
        let p = Vec3::new(1.0, 2.0, 3.0);
        let transformed = m.transform_point(p);
        assert!((transformed.x - 1.0).abs() < 0.001);
        assert!((transformed.y - 2.0).abs() < 0.001);
    }
}
