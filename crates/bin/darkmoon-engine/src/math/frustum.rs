use kajiya_simple::{Mat4, Vec3};
use super::aabb::Aabb;

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Self {
        let normalized = normal.normalize();
        Self {
            normal: normalized,
            distance: normalized.dot(point),
        }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6], // left, right, bottom, top, near, far
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntersectionResult {
    Outside,
    Intersecting,
    Inside,
}

impl Frustum {
    pub fn from_view_projection_matrix(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array_2d();
        
        // Extract frustum planes from view-projection matrix
        let planes = [
            // Left plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]).normalize(),
                m[3][3] + m[3][0],
            ),
            // Right plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]).normalize(),
                m[3][3] - m[3][0],
            ),
            // Bottom plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]).normalize(),
                m[3][3] + m[3][1],
            ),
            // Top plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]).normalize(),
                m[3][3] - m[3][1],
            ),
            // Near plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]).normalize(),
                m[3][3] + m[3][2],
            ),
            // Far plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]).normalize(),
                m[3][3] - m[3][2],
            ),
        ];

        Self { planes }
    }

    pub fn test_point(&self, point: Vec3) -> IntersectionResult {
        for plane in &self.planes {
            if !plane.is_point_in_front(point) {
                return IntersectionResult::Outside;
            }
        }
        IntersectionResult::Inside
    }

    pub fn test_sphere(&self, center: Vec3, radius: f32) -> IntersectionResult {
        let mut intersecting = false;

        for plane in &self.planes {
            let distance = plane.distance_to_point(center);
            
            if distance < -radius {
                return IntersectionResult::Outside;
            } else if distance < radius {
                intersecting = true;
            }
        }

        if intersecting {
            IntersectionResult::Intersecting
        } else {
            IntersectionResult::Inside
        }
    }

    pub fn test_aabb(&self, aabb: &Aabb) -> IntersectionResult {
        let mut intersecting = false;

        for plane in &self.planes {
            // Get the positive vertex (vertex that is most positive to plane normal)
            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            // Get the negative vertex (vertex that is most negative to plane normal)
            let negative_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.min.x } else { aabb.max.x },
                if plane.normal.y >= 0.0 { aabb.min.y } else { aabb.max.y },
                if plane.normal.z >= 0.0 { aabb.min.z } else { aabb.max.z },
            );

            // If positive vertex is outside, then the whole AABB is outside
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return IntersectionResult::Outside;
            }

            // If negative vertex is outside, then the AABB is intersecting
            if plane.distance_to_point(negative_vertex) < 0.0 {
                intersecting = true;
            }
        }

        if intersecting {
            IntersectionResult::Intersecting
        } else {
            IntersectionResult::Inside
        }
    }

    pub fn is_visible_point(&self, point: Vec3) -> bool {
        matches!(self.test_point(point), IntersectionResult::Inside | IntersectionResult::Intersecting)
    }

    pub fn is_visible_sphere(&self, center: Vec3, radius: f32) -> bool {
        matches!(self.test_sphere(center, radius), IntersectionResult::Inside | IntersectionResult::Intersecting)
    }

    pub fn is_visible_aabb(&self, aabb: &Aabb) -> bool {
        matches!(self.test_aabb(aabb), IntersectionResult::Inside | IntersectionResult::Intersecting)
    }
}
