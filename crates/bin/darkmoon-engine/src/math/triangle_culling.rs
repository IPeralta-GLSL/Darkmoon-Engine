use dolly::glam::{Vec2, Vec3, Vec4, Mat4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub uvs: Option<[Vec2; 3]>,
    pub material_id: Option<u32>,
}

impl Triangle {
    pub fn new(vertices: [Vec3; 3]) -> Self {
        let edge1 = vertices[1] - vertices[0];
        let edge2 = vertices[2] - vertices[0];
        let normal = edge1.cross(edge2).normalize();
        
        Self {
            vertices,
            normals: [normal; 3], 
            uvs: None,
            material_id: None,
        }
    }

    pub fn face_normal(&self) -> Vec3 {
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        edge1.cross(edge2).normalize()
    }

    pub fn area(&self) -> f32 {
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        edge1.cross(edge2).length() * 0.5
    }

    pub fn center(&self) -> Vec3 {
        (self.vertices[0] + self.vertices[1] + self.vertices[2]) / 3.0
    }

    pub fn is_degenerate(&self, epsilon: f32) -> bool {
        self.area() < epsilon
    }

    pub fn transform(&self, matrix: &Mat4) -> Self {
        Self {
            vertices: [
                (*matrix * Vec4::new(self.vertices[0].x, self.vertices[0].y, self.vertices[0].z, 1.0)).truncate(),
                (*matrix * Vec4::new(self.vertices[1].x, self.vertices[1].y, self.vertices[1].z, 1.0)).truncate(),
                (*matrix * Vec4::new(self.vertices[2].x, self.vertices[2].y, self.vertices[2].z, 1.0)).truncate(),
            ],
            normals: self.normals,
            uvs: self.uvs,
            material_id: self.material_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveCullingMethod {
    BackFace,           
    SmallTriangle,      
    ZeroArea,          
    ViewDependent,     
    Combined,          
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriangleCullingConfig {
    pub enabled: bool,
    pub methods: Vec<PrimitiveCullingMethod>,
    pub backface_epsilon: f32,          
    pub min_triangle_area: f32,         
    pub min_world_area: f32,           
    pub degenerate_epsilon: f32,       
    pub max_distance: f32,             
    pub angle_threshold: f32,          
    pub debug_logging: bool,           
    pub log_interval_frames: u32,      
}

impl Default for TriangleCullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            methods: vec![
                PrimitiveCullingMethod::BackFace,
                PrimitiveCullingMethod::ZeroArea,
                PrimitiveCullingMethod::SmallTriangle,
            ],
            backface_epsilon: 0.001,
            min_triangle_area: 4.0,        
            min_world_area: 0.0001,        
            degenerate_epsilon: 0.0001,
            max_distance: 1000.0,
            angle_threshold: 0.1,          
            debug_logging: false,
            log_interval_frames: 60,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TriangleCullingStats {
    pub triangles_tested: u32,
    pub backface_culled: u32,
    pub small_triangle_culled: u32,
    pub degenerate_culled: u32,
    pub view_dependent_culled: u32,
    pub triangles_rendered: u32,
    pub total_culled: u32,
}

impl TriangleCullingStats {
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    pub fn culling_efficiency(&self) -> f32 {
        if self.triangles_tested == 0 {
            0.0
        } else {
            (self.total_culled as f32 / self.triangles_tested as f32) * 100.0
        }
    }
}

pub struct TriangleCuller {
    config: TriangleCullingConfig,
    statistics: TriangleCullingStats,
    frame_count: u32,
}

impl TriangleCuller {
    pub fn new(config: TriangleCullingConfig) -> Self {
        Self {
            config,
            statistics: Default::default(),
            frame_count: 0,
        }
    }

    pub fn update_config(&mut self, config: TriangleCullingConfig) {
        self.config = config;
    }

    pub fn is_backface(&self, triangle: &Triangle, camera_pos: Vec3) -> bool {
        if !self.config.methods.contains(&PrimitiveCullingMethod::BackFace) &&
           !self.config.methods.contains(&PrimitiveCullingMethod::Combined) {
            return false;
        }

        let face_normal = triangle.face_normal();
        let to_camera = (camera_pos - triangle.center()).normalize();

        face_normal.dot(to_camera) <= self.config.backface_epsilon
    }

    pub fn is_small_triangle(&self, triangle: &Triangle, view_proj_matrix: &Mat4, viewport_size: Vec2) -> bool {
        if !self.config.methods.contains(&PrimitiveCullingMethod::SmallTriangle) &&
           !self.config.methods.contains(&PrimitiveCullingMethod::Combined) {
            return false;
        }

        let screen_vertices: Vec<Vec2> = triangle.vertices.iter()
            .map(|v| {
                let clip = *view_proj_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                let ndc = clip / clip.w;
                Vec2::new(
                    (ndc.x * 0.5 + 0.5) * viewport_size.x,
                    (ndc.y * 0.5 + 0.5) * viewport_size.y,
                )
            })
            .collect();

        let edge1 = screen_vertices[1] - screen_vertices[0];
        let edge2 = screen_vertices[2] - screen_vertices[0];
        let screen_area = (edge1.x * edge2.y - edge1.y * edge2.x).abs() * 0.5;

        screen_area < self.config.min_triangle_area
    }

    pub fn is_degenerate_triangle(&self, triangle: &Triangle) -> bool {
        if !self.config.methods.contains(&PrimitiveCullingMethod::ZeroArea) &&
           !self.config.methods.contains(&PrimitiveCullingMethod::Combined) {
            return false;
        }

        triangle.is_degenerate(self.config.degenerate_epsilon) || 
        triangle.area() < self.config.min_world_area
    }

    pub fn is_view_dependent_culled(&self, triangle: &Triangle, camera_pos: Vec3) -> bool {
        if !self.config.methods.contains(&PrimitiveCullingMethod::ViewDependent) &&
           !self.config.methods.contains(&PrimitiveCullingMethod::Combined) {
            return false;
        }

        let center = triangle.center();
        let distance = (camera_pos - center).length();

        if distance > self.config.max_distance {
            return true;
        }

        let face_normal = triangle.face_normal();
        let to_camera = (camera_pos - center).normalize();
        let angle = face_normal.dot(to_camera).abs();
        
        angle < self.config.angle_threshold
    }

    pub fn should_cull_triangle(
        &mut self,
        triangle: &Triangle,
        camera_pos: Vec3,
        view_proj_matrix: &Mat4,
        viewport_size: Vec2,
    ) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.statistics.triangles_tested += 1;

        if self.is_backface(triangle, camera_pos) {
            self.statistics.backface_culled += 1;
            self.statistics.total_culled += 1;
            return true;
        }

        if self.is_degenerate_triangle(triangle) {
            self.statistics.degenerate_culled += 1;
            self.statistics.total_culled += 1;
            return true;
        }

        if self.is_small_triangle(triangle, view_proj_matrix, viewport_size) {
            self.statistics.small_triangle_culled += 1;
            self.statistics.total_culled += 1;
            return true;
        }

        if self.is_view_dependent_culled(triangle, camera_pos) {
            self.statistics.view_dependent_culled += 1;
            self.statistics.total_culled += 1;
            return true;
        }

        self.statistics.triangles_rendered += 1;
        false
    }

    pub fn cull_triangles(
        &mut self,
        triangles: &[Triangle],
        camera_pos: Vec3,
        view_proj_matrix: &Mat4,
        viewport_size: Vec2,
    ) -> Vec<Triangle> {
        if !self.config.enabled {
            return triangles.to_vec();
        }

        triangles.iter()
            .filter(|triangle| {
                !self.should_cull_triangle(triangle, camera_pos, view_proj_matrix, viewport_size)
            })
            .cloned()
            .collect()
    }

    pub fn test_triangle(&mut self, triangle: &Triangle, view_proj_matrix: Option<&Mat4>) {
        if !self.config.enabled {
            return;
        }

        let camera_pos = Vec3::new(0.0, 0.0, 5.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);

        let view_proj = view_proj_matrix.cloned().unwrap_or(Mat4::IDENTITY);
        
        let _ = self.should_cull_triangle(triangle, camera_pos, &view_proj, viewport_size);
    }

    pub fn end_frame(&mut self) {
        self.frame_count += 1;
        
        if self.config.debug_logging && 
           self.frame_count % self.config.log_interval_frames == 0 &&
           self.statistics.triangles_tested > 0 {
            
            println!("Triangle Culling Stats: {}/{} triangles rendered ({:.1}% culled)",
                self.statistics.triangles_rendered,
                self.statistics.triangles_tested,
                self.statistics.culling_efficiency()
            );
            
            if self.statistics.total_culled > 0 {
                println!("  Breakdown: {} backface, {} degenerate, {} small, {} view-dependent",
                    self.statistics.backface_culled,
                    self.statistics.degenerate_culled,
                    self.statistics.small_triangle_culled,
                    self.statistics.view_dependent_culled
                );
            }
        }
    }

    pub fn get_statistics(&self) -> &TriangleCullingStats {
        &self.statistics
    }

    pub fn reset_statistics(&mut self) {
        self.statistics.reset();
        self.frame_count = 0;
    }
}

pub fn extract_triangles_from_mesh(
    vertices: &[Vec3],
    indices: &[u32],
    normals: Option<&[Vec3]>,
    uvs: Option<&[Vec2]>,
) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;
            
            if i0 < vertices.len() && i1 < vertices.len() && i2 < vertices.len() {
                let triangle_vertices = [vertices[i0], vertices[i1], vertices[i2]];
                
                let triangle_normals = if let Some(normals) = normals {
                    [normals[i0], normals[i1], normals[i2]]
                } else {
                    let face_normal = Triangle::new(triangle_vertices).face_normal();
                    [face_normal; 3]
                };
                
                let triangle_uvs = if let Some(uvs) = uvs {
                    Some([uvs[i0], uvs[i1], uvs[i2]])
                } else {
                    None
                };
                
                triangles.push(Triangle {
                    vertices: triangle_vertices,
                    normals: triangle_normals,
                    uvs: triangle_uvs,
                    material_id: None,
                });
            }
        }
    }
    
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_creation() {
        let vertices = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let triangle = Triangle::new(vertices);
        
        assert_eq!(triangle.vertices, vertices);
        assert!(triangle.area() > 0.0);
        assert!(!triangle.is_degenerate(0.0001));
    }

    #[test]
    fn test_degenerate_triangle() {
        let vertices = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0), 
            Vec3::new(1.0, 0.0, 0.0),
        ];
        let triangle = Triangle::new(vertices);
        
        assert!(triangle.is_degenerate(0.001));
    }

    #[test]
    fn test_backface_culling() {
        let mut culler = TriangleCuller::new(TriangleCullingConfig::default());
        
        let vertices = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let triangle = Triangle::new(vertices);
        let camera_pos = Vec3::new(0.5, 0.5, -1.0); 
        
        assert!(culler.is_backface(&triangle, camera_pos));
    }
}
