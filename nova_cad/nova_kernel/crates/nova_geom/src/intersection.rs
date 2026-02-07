//! Curve-curve and surface-surface intersection algorithms

use crate::{GeomResult, GeometryError, Curve, Surface, Point3, Vec3};

/// Intersection between two curves
pub fn curve_curve_intersection(
    curve1: &dyn Curve,
    curve2: &dyn Curve,
    tolerance: f64,
) -> GeomResult<Vec<(f64, f64, Point3)>> {
    let mut result = Vec::new();
    
    // Sample both curves and find close points
    let n1 = 50;
    let n2 = 50;
    
    let range1 = curve1.param_range();
    let range2 = curve2.param_range();
    
    for i in 0..=n1 {
        let t1 = range1.denormalize(i as f64 / n1 as f64);
        let p1 = curve1.evaluate(t1);
        
        for j in 0..=n2 {
            let t2 = range2.denormalize(j as f64 / n2 as f64);
            let p2 = curve2.evaluate(t2);
            
            if p1.distance_to(&p2) < tolerance {
                // Refine with closest point
                if let Ok((refined_t2, refined_p2, dist)) = curve2.closest_point(&p1) {
                    if dist < tolerance {
                        result.push((t1, refined_t2, p1.midpoint(&refined_p2)));
                    }
                }
            }
        }
    }
    
    // Remove duplicates
    result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    result.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6);
    
    Ok(result)
}

/// Intersection between a curve and a surface
pub fn curve_surface_intersection(
    curve: &dyn Curve,
    surface: &dyn Surface,
    tolerance: f64,
) -> GeomResult<Vec<(f64, f64, f64, Point3)>> {
    let mut result = Vec::new();
    
    // Sample curve and check distance to surface
    let n = 100;
    let range = curve.param_range();
    
    for i in 0..=n {
        let t = range.denormalize(i as f64 / n as f64);
        let p = curve.evaluate(t);
        
        if let Ok((u, v, closest, dist)) = surface.closest_point(&p) {
            if dist < tolerance {
                result.push((t, u, v, closest));
            }
        }
    }
    
    Ok(result)
}

/// Intersection between two surfaces
pub fn surface_surface_intersection(
    surface1: &dyn Surface,
    surface2: &dyn Surface,
    tolerance: f64,
) -> GeomResult<Vec<Box<dyn Curve>>> {
    // Surface-surface intersection is complex
    // This is a simplified implementation that returns empty
    // Full implementation would use marching methods
    Ok(Vec::new())
}

/// Ray-surface intersection
pub fn ray_surface_intersection(
    origin: &Point3,
    direction: &Vec3,
    surface: &dyn Surface,
    max_distance: f64,
) -> Option<(f64, f64, f64, Point3)> {
    // Sample along ray and find intersection
    let n = 100;
    let dir = direction.normalized();
    
    for i in 0..=n {
        let t = max_distance * i as f64 / n as f64;
        let p = *origin + dir * t;
        
        if let Ok((u, v, closest, dist)) = surface.closest_point(&p) {
            if dist < 1e-6 {
                return Some((t, u, v, closest));
            }
        }
    }
    
    None
}

/// Check if a point is inside a closed surface (ray casting)
pub fn point_in_surface(point: &Point3, surface: &dyn Surface) -> bool {
    // Cast ray in +X direction and count intersections
    // Odd count = inside, even count = outside
    // Simplified - full implementation needed
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::{Line, CircularArc};
    use crate::surface::{PlanarSurface, CylindricalSurface};

    #[test]
    fn test_line_line_intersection() {
        let line1 = Line::infinite(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::X,
        ).unwrap();
        
        let line2 = Line::infinite(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::Y,
        ).unwrap();
        
        let intersections = curve_curve_intersection(
            &line1 as &dyn Curve,
            &line2 as &dyn Curve,
            1e-6,
        ).unwrap();
        
        // Lines intersect at origin
        assert!(!intersections.is_empty());
    }

    #[test]
    fn test_line_plane_intersection() {
        let line = Line::infinite(
            Point3::new(0.0, 0.0, 5.0),
            Vec3::Z,
        ).unwrap();
        
        let plane = PlanarSurface::new(
            Point3::ORIGIN,
            Vec3::X,
            Vec3::Y,
        ).unwrap();
        
        let intersections = curve_surface_intersection(
            &line as &dyn Curve,
            &plane as &dyn Surface,
            1e-6,
        ).unwrap();
        
        assert!(!intersections.is_empty());
        
        // Intersection should be at z=0
        let (_, _, _, p) = intersections[0];
        assert!(p.z().abs() < 1e-6);
    }
}
