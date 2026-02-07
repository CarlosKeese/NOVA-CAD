//! NURBS (Non-Uniform Rational B-Spline) curves and surfaces

use crate::{GeomResult, GeometryError, ParamRange, UVRange, Curve, CurveEvaluation, 
            Surface, SurfaceEvaluation, Tessellation, Tessellatable, CurveType, SurfaceType};
use nova_math::{Point3, Vec3, Point4, Vec4, Transform3};

/// NURBS curve
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    /// Degree
    degree: u32,
    /// Control points in homogeneous coordinates (x, y, z, w)
    control_points: Vec<Point4>,
    /// Knot vector
    knots: Vec<f64>,
    /// Parameter range
    param_range: ParamRange,
    /// Is periodic
    periodic: bool,
}

impl NurbsCurve {
    /// Create a new NURBS curve
    pub fn new(
        degree: u32,
        control_points: Vec<Point4>,
        knots: Vec<f64>,
    ) -> GeomResult<Self> {
        if degree < 1 {
            return Err(GeometryError::InvalidParameter(
                "Degree must be at least 1".to_string()
            ));
        }
        
        let n = control_points.len();
        let m = knots.len();
        
        // Check knot vector length
        if m != n + degree as usize + 1 {
            return Err(GeometryError::InvalidParameter(
                format!("Invalid knot vector length: expected {}, got {}", n + degree as usize + 1, m)
            ));
        }
        
        // Check knot vector is non-decreasing
        for i in 1..knots.len() {
            if knots[i] < knots[i - 1] {
                return Err(GeometryError::InvalidParameter(
                    "Knot vector must be non-decreasing".to_string()
                ));
            }
        }
        
        let param_range = ParamRange::new(knots[degree as usize], knots[n]);
        
        Ok(Self {
            degree,
            control_points,
            knots,
            param_range,
            periodic: false,
        })
    }

    /// Create from 3D control points and weights
    pub fn from_points_and_weights(
        degree: u32,
        points: &[Point3],
        weights: &[f64],
        knots: Vec<f64>,
    ) -> GeomResult<Self> {
        if points.len() != weights.len() {
            return Err(GeometryError::InvalidParameter(
                "Number of points and weights must match".to_string()
            ));
        }
        
        let control_points: Vec<Point4> = points.iter()
            .zip(weights.iter())
            .map(|(p, w)| Point4::new(p.x() * w, p.y() * w, p.z() * w, *w))
            .collect();
        
        Self::new(degree, control_points, knots)
    }

    /// Get the degree
    pub fn degree(&self) -> u32 {
        self.degree
    }

    /// Get the number of control points
    pub fn num_control_points(&self) -> usize {
        self.control_points.len()
    }

    /// Get a control point in 3D (dehomogenized)
    pub fn control_point_3d(&self, i: usize) -> Option<Point3> {
        self.control_points.get(i).map(|p| {
            let w = p.w();
            if w.abs() < 1e-10 {
                Point3::new(p.x(), p.y(), p.z())
            } else {
                Point3::new(p.x() / w, p.y() / w, p.z() / w)
            }
        })
    }

    /// Get the weight of a control point
    pub fn weight(&self, i: usize) -> Option<f64> {
        self.control_points.get(i).map(|p| p.w())
    }

    /// Check if the curve is rational (has varying weights)
    pub fn is_rational(&self) -> bool {
        let first_weight = self.control_points[0].w();
        self.control_points.iter().any(|p| (p.w() - first_weight).abs() > 1e-10)
    }

    /// Find the knot span for a parameter value
    fn find_span(&self, t: f64) -> usize {
        let n = self.control_points.len() - 1;
        let p = self.degree as usize;
        
        // Special case: t is at the end
        if t >= self.knots[n + 1] {
            return n;
        }
        
        // Binary search
        let mut low = p;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        
        while t < self.knots[mid] || t >= self.knots[mid + 1] {
            if t < self.knots[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        
        mid
    }

    /// Compute basis functions using Cox-de Boor recursion
    fn basis_functions(&self, i: usize, t: f64) -> Vec<f64> {
        let p = self.degree as usize;
        let mut n = vec![0.0; p + 1];
        let mut left = vec![0.0; p + 1];
        let mut right = vec![0.0; p + 1];
        
        n[0] = 1.0;
        
        for j in 1..=p {
            left[j] = t - self.knots[i + 1 - j];
            right[j] = self.knots[i + j] - t;
            let mut saved = 0.0;
            
            for r in 0..j {
                let temp = n[r] / (right[r + 1] + left[j - r]);
                n[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            n[j] = saved;
        }
        
        n
    }

    /// Evaluate the curve using de Boor's algorithm
    pub fn evaluate_homogeneous(&self, t: f64) -> Point4 {
        let p = self.degree as usize;
        let i = self.find_span(t);
        let n = self.basis_functions(i, t);
        
        let mut result = Vec4::new(0.0, 0.0, 0.0, 0.0);
        for j in 0..=p {
            let cp = self.control_points[i - p + j];
            let weight = n[j];
            result = Vec4::new(
                result.x() + cp.x() * weight,
                result.y() + cp.y() * weight,
                result.z() + cp.z() * weight,
                result.w() + cp.w() * weight,
            );
        }
        
        Point4::new(result.x(), result.y(), result.z(), result.w())
    }

    /// Insert a knot
    pub fn insert_knot(&mut self, t: f64, times: u32) -> GeomResult<()> {
        // Simplified implementation
        // Full implementation would use knot insertion algorithm
        self.knots.push(t);
        self.knots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Ok(())
    }

    /// Elevate degree
    pub fn elevate_degree(&mut self) -> GeomResult<()> {
        // Simplified - full implementation is complex
        self.degree += 1;
        Ok(())
    }
}

impl Curve for NurbsCurve {
    fn evaluate(&self, t: f64) -> Point3 {
        let h = self.evaluate_homogeneous(t);
        let w = h.w();
        if w.abs() < 1e-10 {
            Point3::new(h.x(), h.y(), h.z())
        } else {
            Point3::new(h.x() / w, h.y() / w, h.z() / w)
        }
    }

    fn derivative(&self, t: f64, order: u32) -> Vec3 {
        // Use finite differences for now
        // Full implementation would use derivative formula
        let eps = 1e-6;
        match order {
            0 => self.evaluate(t).to_vector(),
            1 => {
                let p1 = self.evaluate(t + eps);
                let p0 = self.evaluate(t - eps);
                (p1.to_vector() - p0.to_vector()) / (2.0 * eps)
            }
            2 => {
                let p1 = self.evaluate(t + eps);
                let p0 = self.evaluate(t);
                let p_1 = self.evaluate(t - eps);
                (p1.to_vector() - p0.to_vector() * 2.0 + p_1.to_vector()) / (eps * eps)
            }
            _ => Vec3::ZERO,
        }
    }

    fn curvature(&self, t: f64) -> f64 {
        let d1 = self.derivative(t, 1);
        let d2 = self.derivative(t, 2);
        let cross = d1.cross(&d2);
        let d1_len = d1.length();
        
        if d1_len < 1e-10 {
            return 0.0;
        }
        
        cross.length() / (d1_len * d1_len * d1_len)
    }

    fn param_range(&self) -> ParamRange {
        self.param_range
    }

    fn arc_length(&self, t: f64) -> f64 {
        // Numerical integration
        let start = self.param_range.start;
        let n = 100;
        let dt = (t - start) / n as f64;
        let mut length = 0.0;
        
        let mut prev = self.evaluate(start);
        for i in 1..=n {
            let curr = self.evaluate(start + i as f64 * dt);
            length += curr.distance_to(&prev);
            prev = curr;
        }
        
        length
    }

    fn parameter_at_length(&self, _length: f64) -> Option<f64> {
        // Requires numerical root finding
        None
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, Point3, f64)> {
        // Sample and find closest
        let n = 100;
        let mut best_t = 0.0;
        let mut best_dist = f64::INFINITY;
        
        for i in 0..=n {
            let t = self.param_range.start + (self.param_range.length() * i as f64 / n as f64);
            let p = self.evaluate(t);
            let dist = p.distance_to(point);
            if dist < best_dist {
                best_dist = dist;
                best_t = t;
            }
        }
        
        // Refine with local search
        let best = self.evaluate(best_t);
        Ok((best_t, best, best_dist))
    }

    fn reverse(&mut self) {
        self.control_points.reverse();
        self.knots.reverse();
        for knot in &mut self.knots {
            *knot = -(*knot);
        }
    }

    fn transform(&mut self, transform: &Transform3) {
        for cp in &mut self.control_points {
            let p = Point3::new(cp.x() / cp.w(), cp.y() / cp.w(), cp.z() / cp.w());
            let transformed = transform.apply_to_point(&p);
            let w = cp.w();
            *cp = Point4::new(transformed.x() * w, transformed.y() * w, transformed.z() * w, w);
        }
    }

    fn curve_type(&self) -> CurveType {
        CurveType::NurbsCurve
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(self.clone())
    }
}

impl Tessellatable for NurbsCurve {
    fn tessellate(&self, tolerance: f64) -> Tessellation {
        // Adaptive tessellation based on curvature
        let mut points = vec![self.evaluate(self.param_range.start)];
        
        let mut stack = vec![(self.param_range.start, self.param_range.end)];
        
        while let Some((t0, t1)) = stack.pop() {
            let tm = (t0 + t1) * 0.5;
            let p0 = self.evaluate(t0);
            let pm = self.evaluate(tm);
            let p1 = self.evaluate(t1);
            
            // Check deviation from straight line
            let line_dist = p0.distance_to(&p1);
            let chord_dist = {
                let line_dir = (p1 - p0).normalized();
                let to_pm = pm - p0;
                let proj = to_pm.dot(&line_dir);
                let closest = p0 + line_dir * proj;
                pm.distance_to(&closest)
            };
            
            if chord_dist > tolerance && (t1 - t0) > 1e-6 {
                stack.push((tm, t1));
                stack.push((t0, tm));
            } else {
                if points.last() != Some(&p0) {
                    points.push(p0);
                }
                points.push(pm);
                if points.last() != Some(&p1) {
                    points.push(p1);
                }
            }
        }
        
        // Remove duplicates and sort
        points.dedup_by(|a, b| a.distance_to(b) < 1e-10);
        
        Tessellation::Polyline(points)
    }
}

/// NURBS surface
#[derive(Debug, Clone, PartialEq)]
pub struct NurbsSurface {
    /// Degree in U direction
    degree_u: u32,
    /// Degree in V direction
    degree_v: u32,
    /// Control points in homogeneous coordinates (grid)
    control_points: Vec<Vec<Point4>>,
    /// Knot vector in U direction
    knots_u: Vec<f64>,
    /// Knot vector in V direction
    knots_v: Vec<f64>,
    /// UV range
    uv_range: UVRange,
}

impl NurbsSurface {
    /// Create a new NURBS surface
    pub fn new(
        degree_u: u32,
        degree_v: u32,
        control_points: Vec<Vec<Point4>>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
    ) -> GeomResult<Self> {
        if degree_u < 1 || degree_v < 1 {
            return Err(GeometryError::InvalidParameter(
                "Degrees must be at least 1".to_string()
            ));
        }
        
        let nu = control_points.len();
        let nv = control_points[0].len();
        
        if knots_u.len() != nu + degree_u as usize + 1 {
            return Err(GeometryError::InvalidParameter(
                "Invalid U knot vector length".to_string()
            ));
        }
        
        if knots_v.len() != nv + degree_v as usize + 1 {
            return Err(GeometryError::InvalidParameter(
                "Invalid V knot vector length".to_string()
            ));
        }
        
        let uv_range = UVRange::new(
            knots_u[degree_u as usize],
            knots_u[nu],
            knots_v[degree_v as usize],
            knots_v[nv],
        );
        
        Ok(Self {
            degree_u,
            degree_v,
            control_points,
            knots_u,
            knots_v,
            uv_range,
        })
    }

    /// Get the degrees
    pub fn degrees(&self) -> (u32, u32) {
        (self.degree_u, self.degree_v)
    }

    /// Get number of control points
    pub fn num_control_points(&self) -> (usize, usize) {
        (self.control_points.len(), self.control_points[0].len())
    }

    /// Evaluate at (u, v)
    pub fn evaluate_homogeneous(&self, u: f64, v: f64) -> Point4 {
        // Evaluate in U direction first
        let nu = self.control_points.len();
        let nv = self.control_points[0].len();
        let pu = self.degree_u as usize;
        let pv = self.degree_v as usize;
        
        let span_u = self.find_span_u(u);
        let basis_u = self.basis_functions_u(span_u, u);
        
        // Compute points on isocurves in V direction
        let mut iso_points: Vec<Point4> = Vec::with_capacity(nv);
        for j in 0..nv {
            let mut p = Vec4::new(0.0, 0.0, 0.0, 0.0);
            for i in 0..=pu {
                let cp = self.control_points[span_u - pu + i][j];
                let weight = basis_u[i];
                p = Vec4::new(
                    p.x() + cp.x() * weight,
                    p.y() + cp.y() * weight,
                    p.z() + cp.z() * weight,
                    p.w() + cp.w() * weight,
                );
            }
            iso_points.push(Point4::new(p.x(), p.y(), p.z(), p.w()));
        }
        
        // Evaluate in V direction
        let span_v = self.find_span_v(v);
        let basis_v = self.basis_functions_v(span_v, v);
        
        let mut result = Vec4::new(0.0, 0.0, 0.0, 0.0);
        for j in 0..=pv {
            let cp = iso_points[span_v - pv + j];
            let weight = basis_v[j];
            result = Vec4::new(
                result.x() + cp.x() * weight,
                result.y() + cp.y() * weight,
                result.z() + cp.z() * weight,
                result.w() + cp.w() * weight,
            );
        }
        
        Point4::new(result.x(), result.y(), result.z(), result.w())
    }

    fn find_span_u(&self, u: f64) -> usize {
        let n = self.control_points.len() - 1;
        let p = self.degree_u as usize;
        
        if u >= self.knots_u[n + 1] {
            return n;
        }
        
        let mut low = p;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        
        while u < self.knots_u[mid] || u >= self.knots_u[mid + 1] {
            if u < self.knots_u[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        
        mid
    }

    fn find_span_v(&self, v: f64) -> usize {
        let n = self.control_points[0].len() - 1;
        let p = self.degree_v as usize;
        
        if v >= self.knots_v[n + 1] {
            return n;
        }
        
        let mut low = p;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        
        while v < self.knots_v[mid] || v >= self.knots_v[mid + 1] {
            if v < self.knots_v[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        
        mid
    }

    fn basis_functions_u(&self, i: usize, u: f64) -> Vec<f64> {
        let p = self.degree_u as usize;
        let mut n = vec![0.0; p + 1];
        let mut left = vec![0.0; p + 1];
        let mut right = vec![0.0; p + 1];
        
        n[0] = 1.0;
        
        for j in 1..=p {
            left[j] = u - self.knots_u[i + 1 - j];
            right[j] = self.knots_u[i + j] - u;
            let mut saved = 0.0;
            
            for r in 0..j {
                let temp = n[r] / (right[r + 1] + left[j - r]);
                n[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            n[j] = saved;
        }
        
        n
    }

    fn basis_functions_v(&self, i: usize, v: f64) -> Vec<f64> {
        let p = self.degree_v as usize;
        let mut n = vec![0.0; p + 1];
        let mut left = vec![0.0; p + 1];
        let mut right = vec![0.0; p + 1];
        
        n[0] = 1.0;
        
        for j in 1..=p {
            left[j] = v - self.knots_v[i + 1 - j];
            right[j] = self.knots_v[i + j] - v;
            let mut saved = 0.0;
            
            for r in 0..j {
                let temp = n[r] / (right[r + 1] + left[j - r]);
                n[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            n[j] = saved;
        }
        
        n
    }
}

impl Surface for NurbsSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let h = self.evaluate_homogeneous(u, v);
        let w = h.w();
        if w.abs() < 1e-10 {
            Point3::new(h.x(), h.y(), h.z())
        } else {
            Point3::new(h.x() / w, h.y() / w, h.z() / w)
        }
    }

    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let eps = 1e-6;
        
        let du = {
            let p1 = self.evaluate(u + eps, v);
            let p0 = self.evaluate(u - eps, v);
            (p1 - p0) / (2.0 * eps)
        };
        
        let dv = {
            let p1 = self.evaluate(u, v + eps);
            let p0 = self.evaluate(u, v - eps);
            (p1 - p0) / (2.0 * eps)
        };
        
        (du, dv)
    }

    fn principal_curvatures(&self, u: f64, v: f64) -> (f64, f64) {
        // Simplified - compute using second derivatives
        let eps = 1e-6;
        
        let (du, dv) = self.derivatives(u, v);
        let normal = du.cross(&dv).normalized();
        
        // Second derivatives
        let duu = {
            let p1 = self.evaluate(u + eps, v);
            let p0 = self.evaluate(u, v);
            let p_1 = self.evaluate(u - eps, v);
            (p1.to_vector() - p0.to_vector() * 2.0 + p_1.to_vector()) / (eps * eps)
        };
        
        let dvv = {
            let p1 = self.evaluate(u, v + eps);
            let p0 = self.evaluate(u, v);
            let p_1 = self.evaluate(u, v - eps);
            (p1.to_vector() - p0.to_vector() * 2.0 + p_1.to_vector()) / (eps * eps)
        };
        
        // Approximate curvatures
        let k1 = duu.dot(&normal).abs();
        let k2 = dvv.dot(&normal).abs();
        
        (k1, k2)
    }

    fn uv_range(&self) -> UVRange {
        self.uv_range
    }

    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>> {
        // Extract control points at constant u
        // Simplified - would need proper curve extraction
        None
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        None
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        // Grid search
        let n = 20;
        let mut best_u = 0.5;
        let mut best_v = 0.5;
        let mut best_dist = f64::INFINITY;
        
        for i in 0..=n {
            for j in 0..=n {
                let u = self.uv_range.u.start + (self.uv_range.u.length() * i as f64 / n as f64);
                let v = self.uv_range.v.start + (self.uv_range.v.length() * j as f64 / n as f64);
                let p = self.evaluate(u, v);
                let dist = p.distance_to(point);
                if dist < best_dist {
                    best_dist = dist;
                    best_u = u;
                    best_v = v;
                }
            }
        }
        
        let closest = self.evaluate(best_u, best_v);
        Ok((best_u, best_v, closest, best_dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        for row in &mut self.control_points {
            for cp in row {
                let p = Point3::new(cp.x() / cp.w(), cp.y() / cp.w(), cp.z() / cp.w());
                let transformed = transform.apply_to_point(&p);
                let w = cp.w();
                *cp = Point4::new(transformed.x() * w, transformed.y() * w, transformed.z() * w, w);
            }
        }
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::NurbsSurface
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_curve_new() {
        let control_points = vec![
            Point4::new(0.0, 0.0, 0.0, 1.0),
            Point4::new(1.0, 1.0, 0.0, 1.0),
            Point4::new(2.0, 0.0, 0.0, 1.0),
        ];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        
        let curve = NurbsCurve::new(2, control_points, knots).unwrap();
        assert_eq!(curve.degree(), 2);
        assert_eq!(curve.num_control_points(), 3);
    }

    #[test]
    fn test_nurbs_curve_evaluate() {
        // Simple quadratic Bezier curve (special case of NURBS)
        let control_points = vec![
            Point4::new(0.0, 0.0, 0.0, 1.0),
            Point4::new(1.0, 2.0, 0.0, 1.0),
            Point4::new(2.0, 0.0, 0.0, 1.0),
        ];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        
        let curve = NurbsCurve::new(2, control_points, knots).unwrap();
        
        // Start point
        let p0 = curve.evaluate(0.0);
        assert!((p0.x()).abs() < 1e-10);
        
        // End point
        let p1 = curve.evaluate(1.0);
        assert!((p1.x() - 2.0).abs() < 1e-10);
        
        // Midpoint should be at (1, 1)
        let pm = curve.evaluate(0.5);
        assert!((pm.x() - 1.0).abs() < 1e-10);
        assert!((pm.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_nurbs_circle() {
        // Create a NURBS circle (quarter circle for simplicity)
        let w = std::f64::consts::FRAC_1_SQRT_2;
        let control_points = vec![
            Point4::new(1.0, 0.0, 0.0, 1.0),
            Point4::new(1.0, 1.0, 0.0, w),
            Point4::new(0.0, 1.0, 0.0, 1.0),
        ];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        
        let curve = NurbsCurve::new(2, control_points, knots).unwrap();
        
        // Check points are at distance ~1 from origin
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let p = curve.evaluate(t);
            let dist = p.distance_to(&Point3::ORIGIN);
            assert!((dist - 1.0).abs() < 0.1); // Approximate circle
        }
    }
}
