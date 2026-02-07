//! Face splitting utilities for Boolean operations
//!
//! Provides functionality to split faces at intersection curves.

use crate::{OpsError, OpsResult};
use nova_math::{Point3, Vec3, ToleranceContext};
use nova_geom::{Curve, Surface, CurveEvaluation, SurfaceEvaluation};
use nova_topo::{Body, Face, Edge, Loop, Coedge, Vertex, EulerOps, Sense, Orientation, Entity};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// A split operation on a face
#[derive(Debug, Clone)]
pub struct FaceSplit {
    /// The face being split
    pub face_id: nova_topo::EntityId,
    /// Intersection curves to split at
    pub split_curves: Vec<SplitCurve>,
    /// New vertices created at intersections
    pub intersection_vertices: Vec<Arc<Vertex>>,
}

/// A curve used to split a face
#[derive(Debug, Clone)]
pub struct SplitCurve {
    /// The intersection curve
    pub curve: Arc<dyn Curve>,
    /// Start parameter on the curve
    pub start_param: f64,
    /// End parameter on the curve
    pub end_param: f64,
    /// UV coordinates on the face surface
    pub uv_coords: Vec<(f64, f64)>,
}

/// Split a face using intersection curves
pub fn split_face_at_curves(
    face: &Face,
    intersection_curves: &[Box<dyn Curve>],
    tolerance: &ToleranceContext,
) -> OpsResult<Vec<Face>> {
    if intersection_curves.is_empty() {
        return Ok(vec![face.clone()]);
    }
    
    let surface = face.surface()
        .ok_or_else(|| OpsError::InvalidBodies("Face has no surface".to_string()))?;
    
    // Find all intersection points between curves and face boundary
    let mut all_intersections: Vec<IntersectionPoint> = Vec::new();
    
    // Get face boundary edges
    let boundary_edges: Vec<_> = face.loops().iter()
        .flat_map(|lp| lp.coedges().iter())
        .map(|c| c.edge().clone())
        .collect();
    
    // Find intersections of each curve with face boundary
    for curve in intersection_curves {
        let intersections = find_curve_face_intersections(
            curve.as_ref(),
            face,
            &boundary_edges,
            tolerance,
        )?;
        all_intersections.extend(intersections);
    }
    
    // Also find intersections between the curves themselves
    let curve_intersections = find_curve_curve_intersections(intersection_curves, tolerance)?;
    all_intersections.extend(curve_intersections);
    
    if all_intersections.is_empty() {
        // No valid intersections found
        return Ok(vec![face.clone()]);
    }
    
    // Sort intersections along each curve
    let sorted_curves = sort_intersections_along_curves(intersection_curves, &all_intersections)?;
    
    // Create new edges along the split curves
    let split_edges = create_split_edges(&sorted_curves, tolerance)?;
    
    // Rebuild face topology with new edges
    let new_faces = rebuild_face_topology(face, &split_edges, tolerance)?;
    
    Ok(new_faces)
}

/// Intersection point on a curve
#[derive(Debug, Clone)]
struct IntersectionPoint {
    /// Position in 3D
    point: Point3,
    /// Parameter on the curve
    curve_param: f64,
    /// Curve index
    curve_idx: usize,
    /// UV on face surface
    uv: (f64, f64),
    /// Whether this is on the face boundary
    on_boundary: bool,
}

/// Find where a curve intersects a face boundary
fn find_curve_face_intersections(
    curve: &dyn Curve,
    face: &Face,
    boundary_edges: &[Arc<Edge>],
    tolerance: &ToleranceContext,
) -> OpsResult<Vec<IntersectionPoint>> {
    let mut intersections = Vec::new();
    let curve_range = curve.param_range();
    
    // Sample points along the curve and check if they're on the face
    let num_samples = 100;
    let mut entering = false;
    let mut last_inside = false;
    
    for i in 0..=num_samples {
        let t = curve_range.start + (curve_range.end - curve_range.start) * (i as f64 / num_samples as f64);
        let point = curve.evaluate(t);
        
        // Check if point is inside the face (using winding number or ray casting)
        let inside = is_point_inside_face(point, face, tolerance)?;
        
        if i == 0 {
            entering = inside;
            last_inside = inside;
        } else if inside != last_inside {
            // Found a boundary crossing - refine the intersection
            let crossing_t = find_boundary_crossing(
                curve,
                t - curve_range.length() / num_samples as f64,
                t,
                face,
                tolerance,
            )?;
            
            let crossing_point = curve.evaluate(crossing_t);
            
            // Find which edge was crossed
            let (uv, on_boundary) = if let Some(uv) = project_point_to_surface(crossing_point, face) {
                (uv, true)
            } else {
                ((0.0, 0.0), false)
            };
            
            intersections.push(IntersectionPoint {
                point: crossing_point,
                curve_param: crossing_t,
                curve_idx: 0, // Will be set later
                uv,
                on_boundary,
            });
            
            last_inside = inside;
        }
    }
    
    // Also check intersections with boundary edges
    for (edge_idx, edge) in boundary_edges.iter().enumerate() {
        if let Some(edge_curve) = edge.curve() {
            let edge_intersections = nova_geom::intersection::curve_curve_intersection(
                curve,
                edge_curve.as_ref(),
                tolerance.tolerance(),
            ).map_err(|e| OpsError::Geometry(format!("Intersection failed: {}", e)))?;
            
            for intersection in edge_intersections {
                if let nova_geom::IntersectionResult::Point(pt) = intersection {
                    let (curve_t, _) = curve.closest_point(pt);
                    
                    intersections.push(IntersectionPoint {
                        point: pt,
                        curve_param: curve_t,
                        curve_idx: edge_idx,
                        uv: project_point_to_surface(pt, face).unwrap_or((0.0, 0.0)),
                        on_boundary: true,
                    });
                }
            }
        }
    }
    
    Ok(intersections)
}

/// Check if a point is inside a face (using winding number)
fn is_point_inside_face(
    point: Point3,
    face: &Face,
    tolerance: &ToleranceContext,
) -> OpsResult<bool> {
    // Project point to surface
    let surface = face.surface()
        .ok_or_else(|| OpsError::InvalidBodies("Face has no surface".to_string()))?;
    
    let (u, v) = surface.closest_point(point);
    let surf_eval = surface.evaluate(u, v);
    
    // Check if point is on surface
    if point.distance_to(&surf_eval.point) > tolerance.tolerance() {
        return Ok(false);
    }
    
    // Use ray casting in UV space to determine if point is inside
    // Cast ray in positive U direction
    let ray_start = (u, v);
    let mut intersection_count = 0;
    
    for loop_ in face.loops() {
        let crossings = count_uv_ray_crossings(ray_start, loop_, surface.as_ref())?;
        // Outer loop: odd = inside, even = outside
        // Inner loops: odd = outside, even = inside
        intersection_count += crossings;
    }
    
    Ok(intersection_count % 2 == 1)
}

/// Count how many times a ray crosses a loop in UV space
fn count_uv_ray_crossings(
    ray_start: (f64, f64),
    loop_: &Loop,
    surface: &dyn Surface,
) -> OpsResult<i32> {
    let mut crossings = 0;
    
    for coedge in loop_.coedges() {
        let edge = coedge.edge();
        if let Some(curve) = edge.curve() {
            // Sample curve and check for crossings
            let range = curve.param_range();
            let num_samples = 20;
            
            let mut last_uv: Option<(f64, f64)> = None;
            
            for i in 0..=num_samples {
                let t = range.start + (range.end - range.start) * (i as f64 / num_samples as f64);
                let pt = curve.evaluate(t);
                let uv = surface.closest_point(pt);
                
                if let Some((last_u, last_v)) = last_uv {
                    // Check if edge crosses the ray (v = constant, u increasing)
                    if (last_v < ray_start.1 && uv.1 >= ray_start.1) ||
                       (last_v >= ray_start.1 && uv.1 < ray_start.1) {
                        // Calculate intersection u coordinate
                        let t_cross = (ray_start.1 - last_v) / (uv.1 - last_v);
                        let u_cross = last_u + t_cross * (uv.0 - last_u);
                        
                        if u_cross > ray_start.0 {
                            crossings += 1;
                        }
                    }
                }
                
                last_uv = Some(uv);
            }
        }
    }
    
    Ok(crossings)
}

/// Find precise boundary crossing using bisection
fn find_boundary_crossing(
    curve: &dyn Curve,
    t1: f64,
    t2: f64,
    face: &Face,
    tolerance: &ToleranceContext,
) -> OpsResult<f64> {
    let mut low = t1;
    let mut high = t2;
    
    for _ in 0..20 { // Max iterations
        let mid = (low + high) / 2.0;
        let point = curve.evaluate(mid);
        let inside = is_point_inside_face(point, face, tolerance)?;
        
        if inside {
            high = mid;
        } else {
            low = mid;
        }
        
        if high - low < tolerance.tolerance() {
            break;
        }
    }
    
    Ok((low + high) / 2.0)
}

/// Project a point to a face's surface
fn project_point_to_surface(point: Point3, face: &Face) -> Option<(f64, f64)> {
    let surface = face.surface()?;
    let uv = surface.closest_point(point);
    Some(uv)
}

/// Find intersections between curves
fn find_curve_curve_intersections(
    curves: &[Box<dyn Curve>],
    tolerance: &ToleranceContext,
) -> OpsResult<Vec<IntersectionPoint>> {
    let mut intersections = Vec::new();
    
    for (i, curve1) in curves.iter().enumerate() {
        for (j, curve2) in curves.iter().enumerate().skip(i + 1) {
            let results = nova_geom::intersection::curve_curve_intersection(
                curve1.as_ref(),
                curve2.as_ref(),
                tolerance.tolerance(),
            ).map_err(|e| OpsError::Geometry(format!("Curve-curve intersection failed: {}", e)))?;
            
            for result in results {
                if let nova_geom::IntersectionResult::Point(pt) = result {
                    let (t1, _) = curve1.closest_point(pt);
                    let (t2, _) = curve2.closest_point(pt);
                    
                    intersections.push(IntersectionPoint {
                        point: pt,
                        curve_param: t1,
                        curve_idx: i,
                        uv: (0.0, 0.0), // Will be computed if needed
                        on_boundary: false,
                    });
                    
                    intersections.push(IntersectionPoint {
                        point: pt,
                        curve_param: t2,
                        curve_idx: j,
                        uv: (0.0, 0.0),
                        on_boundary: false,
                    });
                }
            }
        }
    }
    
    Ok(intersections)
}

/// Sort intersections along curves
fn sort_intersections_along_curves(
    curves: &[Box<dyn Curve>],
    intersections: &[IntersectionPoint],
) -> OpsResult<Vec<SortedCurve>> {
    let mut sorted_curves: Vec<SortedCurve> = Vec::new();
    
    for (idx, _curve) in curves.iter().enumerate() {
        let mut curve_intersections: Vec<_> = intersections
            .iter()
            .filter(|ip| ip.curve_idx == idx)
            .cloned()
            .collect();
        
        // Sort by parameter
        curve_intersections.sort_by(|a, b| {
            a.curve_param.partial_cmp(&b.curve_param).unwrap()
        });
        
        // Remove duplicates
        curve_intersections.dedup_by(|a, b| {
            (a.curve_param - b.curve_param).abs() < 1e-9
        });
        
        sorted_curves.push(SortedCurve {
            curve_idx: idx,
            intersections: curve_intersections,
        });
    }
    
    Ok(sorted_curves)
}

/// A curve with sorted intersections
#[derive(Debug, Clone)]
struct SortedCurve {
    curve_idx: usize,
    intersections: Vec<IntersectionPoint>,
}

/// Create edges along split curves
fn create_split_edges(
    sorted_curves: &[SortedCurve],
    tolerance: &ToleranceContext,
) -> OpsResult<Vec<Arc<Edge>>> {
    let mut edges = Vec::new();
    
    for sorted in sorted_curves {
        // Create edges between consecutive intersection points
        for i in 0..sorted.intersections.len().saturating_sub(1) {
            let start = &sorted.intersections[i];
            let end = &sorted.intersections[i + 1];
            
            // Create vertices
            let start_vertex = Arc::new(Vertex::new(start.point));
            let end_vertex = Arc::new(Vertex::new(end.point));
            
            // Create edge (curve will be approximated or trimmed)
            let edge = Arc::new(Edge::new(start_vertex, end_vertex));
            edges.push(edge);
        }
    }
    
    Ok(edges)
}

/// Rebuild face topology with split edges
fn rebuild_face_topology(
    original_face: &Face,
    split_edges: &[Arc<Edge>],
    tolerance: &ToleranceContext,
) -> OpsResult<Vec<Face>> {
    use nova_topo::EulerOps;
    
    // This is a complex operation that requires:
    // 1. Trimming the original face boundary at split points
    // 2. Creating new loops that include the split edges
    // 3. Creating new faces for each region
    
    // For now, return the original face if no valid splits
    if split_edges.is_empty() {
        return Ok(vec![original_face.clone()]);
    }
    
    // TODO: Implement proper face reconstruction using Euler operators
    // This requires:
    // - KEMR (Kill Edge Make Ring) to split edges
    // - MEF (Make Edge Face) to create new faces
    // - Proper handling of inner/outer loops
    
    Ok(vec![original_face.clone()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_geom::Line;
    use nova_math::Plane;

    #[test]
    fn test_intersection_point() {
        let ip = IntersectionPoint {
            point: Point3::new(1.0, 2.0, 3.0),
            curve_param: 0.5,
            curve_idx: 0,
            uv: (0.5, 0.5),
            on_boundary: true,
        };
        
        assert_eq!(ip.point.x(), 1.0);
        assert!(ip.on_boundary);
    }
}
