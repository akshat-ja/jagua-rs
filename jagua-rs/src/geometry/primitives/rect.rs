use crate::geometry::geo_enums::{GeoPosition, GeoRelation};
use crate::geometry::geo_traits::{
    AlmostCollidesWith, CollidesWith, DistanceTo, SeparationDistance, Shape,
};
use crate::geometry::primitives::Edge;
use crate::geometry::primitives::Point;
use crate::util::FPA;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

///Axis-aligned rectangle
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Rect {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl Rect {
    pub fn new(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Self {
        debug_assert!(
            x_min < x_max && y_min < y_max,
            "invalid rectangle, x_min: {}, x_max: {}, y_min: {}, y_max: {}",
            x_min,
            x_max,
            y_min,
            y_max
        );
        Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    /// Returns the geometric relation between `self` and another [`Rect`].
    pub fn relation_to(&self, other: Rect) -> GeoRelation {
        if self.collides_with(&other) {
            if self.x_min <= other.x_min
                && self.y_min <= other.y_min
                && self.x_max >= other.x_max
                && self.y_max >= other.y_max
            {
                GeoRelation::Surrounding
            } else if self.x_min >= other.x_min
                && self.y_min >= other.y_min
                && self.x_max <= other.x_max
                && self.y_max <= other.y_max
            {
                GeoRelation::Enclosed
            } else {
                GeoRelation::Intersecting
            }
        } else {
            GeoRelation::Disjoint
        }
    }

    /// Returns the [`GeoRelation`] between `self` and another [`Rect`], with a tolerance for floating point precision.
    /// In edge cases, this method will lean towards `Surrounding` and `Enclosed` instead of `Intersecting`.
    pub fn almost_relation_to(&self, other: Rect) -> GeoRelation {
        if self.almost_collides_with(&other) {
            if FPA::from(self.x_min) <= FPA::from(other.x_min)
                && FPA::from(self.y_min) <= FPA::from(other.y_min)
                && FPA::from(self.x_max) >= FPA::from(other.x_max)
                && FPA::from(self.y_max) >= FPA::from(other.y_max)
            {
                GeoRelation::Surrounding
            } else if FPA::from(self.x_min) >= FPA::from(other.x_min)
                && FPA::from(self.y_min) >= FPA::from(other.y_min)
                && FPA::from(self.x_max) <= FPA::from(other.x_max)
                && FPA::from(self.y_max) <= FPA::from(other.y_max)
            {
                GeoRelation::Enclosed
            } else {
                GeoRelation::Intersecting
            }
        } else {
            GeoRelation::Disjoint
        }
    }

    /// Returns a new rectangle with the same centroid but inflated
    /// to be the minimum square that contains `self`.
    pub fn inflate_to_square(&self) -> Rect {
        let width = self.x_max - self.x_min;
        let height = self.y_max - self.y_min;
        let mut dx = 0.0;
        let mut dy = 0.0;
        if height < width {
            dy = (width - height) / 2.0;
        } else if width < height {
            dx = (height - width) / 2.0;
        }
        Rect::new(
            self.x_min - dx,
            self.y_min - dy,
            self.x_max + dx,
            self.y_max + dy,
        )
    }

    /// Returns a new rectangle with the same centroid but scaled by `factor`.
    pub fn scale(self, factor: f32) -> Self {
        let dx = (self.x_max - self.x_min) * (factor - 1.0) / 2.0;
        let dy = (self.y_max - self.y_min) * (factor - 1.0) / 2.0;
        self.resize_by(dx, dy)
            .expect("scaling should not lead to invalid rectangle")
    }

    /// Returns a new rectangle with the same centroid as `self` but expanded by `dx` in both x-directions and by `dy` in both y-directions.
    /// If the new rectangle is invalid (x_min >= x_max or y_min >= y_max), returns None.
    pub fn resize_by(mut self, dx: f32, dy: f32) -> Option<Self> {
        self.x_min -= dx;
        self.y_min -= dy;
        self.x_max += dx;
        self.y_max += dy;

        if self.x_min < self.x_max && self.y_min < self.y_max {
            Some(self)
        } else {
            //resizing would lead to invalid rectangle
            None
        }
    }

    /// For all quadrants, contains indices of the two neighbors of the quadrant at that index.
    pub const QUADRANT_NEIGHBOR_LAYOUT: [[usize; 2]; 4] = [[1, 3], [0, 2], [1, 3], [0, 2]];

    /// Returns the 4 quadrants of `self`.
    /// Ordered in the same way as quadrants in a cartesian plane:
    /// <https://en.wikipedia.org/wiki/Quadrant_(plane_geometry)>
    pub fn quadrants(&self) -> [Self; 4] {
        let mid = self.centroid();
        let corners = self.corners();

        let q1 = Edge::new(corners[0], mid).bbox();
        let q2 = Edge::new(corners[1], mid).bbox();
        let q3 = Edge::new(corners[2], mid).bbox();
        let q4 = Edge::new(corners[3], mid).bbox();

        [q1, q2, q3, q4]
    }

    /// Returns the four corners of `self`, in the same order as [Rect::quadrants].
    pub fn corners(&self) -> [Point; 4] {
        [
            Point(self.x_max, self.y_max),
            Point(self.x_min, self.y_max),
            Point(self.x_min, self.y_min),
            Point(self.x_max, self.y_min),
        ]
    }

    /// Returns the four edges that make up `self`, in the same order as [Rect::quadrants].
    pub fn edges(&self) -> [Edge; 4] {
        let c = self.corners();
        [
            Edge::new(c[0], c[1]),
            Edge::new(c[1], c[2]),
            Edge::new(c[2], c[3]),
            Edge::new(c[3], c[0]),
        ]
    }
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    /// Returns the largest rectangle that is contained in both `a` and `b`.
    pub fn intersection(a: Rect, b: Rect) -> Option<Rect> {
        let x_min = f32::max(a.x_min, b.x_min);
        let y_min = f32::max(a.y_min, b.y_min);
        let x_max = f32::min(a.x_max, b.x_max);
        let y_max = f32::min(a.y_max, b.y_max);
        if x_min < x_max && y_min < y_max {
            Some(Rect::new(x_min, y_min, x_max, y_max))
        } else {
            None
        }
    }

    /// Returns the smallest rectangle that contains both `a` and `b`.
    pub fn bounding_rect(a: Rect, b: Rect) -> Rect {
        let x_min = f32::min(a.x_min, b.x_min);
        let y_min = f32::min(a.y_min, b.y_min);
        let x_max = f32::max(a.x_max, b.x_max);
        let y_max = f32::max(a.y_max, b.y_max);
        Rect::new(x_min, y_min, x_max, y_max)
    }
}

impl Shape for Rect {
    fn centroid(&self) -> Point {
        Point(
            (self.x_min + self.x_max) / 2.0,
            (self.y_min + self.y_max) / 2.0,
        )
    }

    fn area(&self) -> f32 {
        (self.x_max - self.x_min) * (self.y_max - self.y_min)
    }

    fn bbox(&self) -> Rect {
        self.clone()
    }

    fn diameter(&self) -> f32 {
        let dx = self.x_max - self.x_min;
        let dy = self.y_max - self.y_min;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }
}

impl CollidesWith<Rect> for Rect {
    #[inline(always)]
    fn collides_with(&self, other: &Rect) -> bool {
        f32::max(self.x_min, other.x_min) <= f32::min(self.x_max, other.x_max)
            && f32::max(self.y_min, other.y_min) <= f32::min(self.y_max, other.y_max)
    }
}

impl AlmostCollidesWith<Rect> for Rect {
    #[inline(always)]
    fn almost_collides_with(&self, other: &Rect) -> bool {
        FPA(f32::max(self.x_min, other.x_min)) <= FPA(f32::min(self.x_max, other.x_max))
            && FPA(f32::max(self.y_min, other.y_min)) <= FPA(f32::min(self.y_max, other.y_max))
    }
}

impl CollidesWith<Point> for Rect {
    #[inline(always)]
    fn collides_with(&self, point: &Point) -> bool {
        let Point(x, y) = *point;
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }
}

impl AlmostCollidesWith<Point> for Rect {
    #[inline(always)]
    fn almost_collides_with(&self, point: &Point) -> bool {
        let (x, y) = (*point).into();
        FPA(x) >= FPA(self.x_min)
            && FPA(x) <= FPA(self.x_max)
            && FPA(y) >= FPA(self.y_min)
            && FPA(y) <= FPA(self.y_max)
    }
}

impl CollidesWith<Edge> for Rect {
    #[inline(always)]
    fn collides_with(&self, edge: &Edge) -> bool {
        //inspired by: https://stackoverflow.com/questions/99353/how-to-test-if-a-line-segment-intersects-an-axis-aligned-rectange-in-2d

        let x_min = edge.x_min();
        let x_max = edge.x_max();
        let y_min = edge.y_min();
        let y_max = edge.y_max();

        //If both end points of the line are entirely outside the range of the rectangle
        if x_max < self.x_min || x_min > self.x_max || y_max < self.y_min || y_min > self.y_max {
            return false;
        }

        //If either end point of the line is inside the rectangle
        if self.collides_with(&edge.start) || self.collides_with(&edge.end) {
            return true;
        }

        //If all corners of rectangle are on the same side of the edge, no collision is possible
        const POINT_EDGE_RELATION: fn(Point, &Edge) -> Ordering =
            |p: Point, edge: &Edge| -> Ordering {
                let Point(p_x, p_y) = p;
                let Point(s_x, s_y) = edge.start;
                let Point(e_x, e_y) = edge.end;
                // if 0.0, the point is on the line
                // if > 0.0, the point is "above" of the line
                // if < 0.0, the point is "below" the line
                let v = (p_x - s_x) * (e_y - s_y) - (p_y - s_y) * (e_x - s_x);
                v.partial_cmp(&0.0).unwrap()
            };

        let all_corners_same_side = self
            .corners()
            .map(|corner| POINT_EDGE_RELATION(corner, edge))
            .windows(2)
            .all(|w| w[0] == w[1]);

        if all_corners_same_side {
            return false;
        }

        //The only possible that remains is that the edge collides with one of the edges of the rectangle
        self.edges()
            .iter()
            .any(|rect_edge| edge.collides_with(rect_edge))
    }
}

impl DistanceTo<Point> for Rect {
    #[inline(always)]
    fn sq_distance_to(&self, point: &Point) -> f32 {
        let Point(x, y) = *point;
        let mut distance: f32 = 0.0;
        if x < self.x_min {
            distance += (x - self.x_min).powi(2);
        } else if x > self.x_max {
            distance += (x - self.x_max).powi(2);
        }
        if y < self.y_min {
            distance += (y - self.y_min).powi(2);
        } else if y > self.y_max {
            distance += (y - self.y_max).powi(2);
        }
        distance.abs()
    }

    #[inline(always)]
    fn distance_to(&self, point: &Point) -> f32 {
        self.sq_distance_to(point).sqrt()
    }
}

impl SeparationDistance<Point> for Rect {
    #[inline(always)]
    fn separation_distance(&self, point: &Point) -> (GeoPosition, f32) {
        let (position, sq_distance) = self.sq_separation_distance(point);
        (position, sq_distance.sqrt())
    }

    #[inline(always)]
    fn sq_separation_distance(&self, point: &Point) -> (GeoPosition, f32) {
        match self.collides_with(point) {
            false => (GeoPosition::Exterior, self.sq_distance_to(point)),
            true => {
                let Point(x, y) = *point;
                let min_distance = [
                    (x - self.x_min).abs(),
                    (x - self.x_max).abs(),
                    (y - self.y_min).abs(),
                    (y - self.y_max).abs(),
                ]
                .into_iter()
                .min_by_key(|&d| OrderedFloat(d))
                .unwrap();
                (GeoPosition::Interior, min_distance.powi(2))
            }
        }
    }
}
