use num::clamp;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Uniform};

use super::utils::{Axis, Coordinate};
use crate::error::Result;

// Tracks areas on the grid and supports overlapping and orthogonal adjacency checks.
// is also responsible for dividing space when an area is split into two.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BoxExtends {
    pub top_left: Coordinate,
    pub bottom_right: Coordinate,
}

impl BoxExtends {
    pub fn new_square(size: u32) -> Self {
        Self {
            top_left: Coordinate::zero(),
            bottom_right: Coordinate::uniform(size - 1),
        }
    }

    pub fn contains(&self, other: &BoxExtends) -> bool {
        self.contains_point(other.top_left) && self.contains_point(other.bottom_right)
    }

    pub fn center(&self) -> Coordinate {
        let x = (self.bottom_right.x - self.top_left.x) / 2 + self.top_left.x;
        let y = (self.bottom_right.y - self.top_left.y) / 2 + self.top_left.y;
        Coordinate { x, y }
    }

    pub fn overlaps(&self, other: &BoxExtends) -> bool {
        let self_top_right = Coordinate {
            x: self.bottom_right.x,
            y: self.top_left.y,
        };
        let self_bottom_left = Coordinate {
            x: self.top_left.x,
            y: self.bottom_right.y,
        };
        let other_top_right = Coordinate {
            x: other.bottom_right.x,
            y: other.top_left.y,
        };
        let other_bottom_left = Coordinate {
            x: other.top_left.x,
            y: other.bottom_right.y,
        };

        let self_overlaps_other = self.contains_point(other.top_left)
            || self.contains_point(other.bottom_right)
            || self.contains_point(other_top_right)
            || self.contains_point(other_bottom_left);

        let other_overlaps_self = other.contains_point(self.top_left)
            || other.contains_point(self.bottom_right)
            || other.contains_point(self_top_right)
            || other.contains_point(self_bottom_left);

        self_overlaps_other || other_overlaps_self
    }

    pub fn contains_point(&self, point: Coordinate) -> bool {
        self.top_left.x <= point.x
            && point.x <= self.bottom_right.x
            && self.top_left.y <= point.y
            && point.y <= self.bottom_right.y
    }

    pub fn get_axis_size(&self, axis: Axis) -> i32 {
        // Axis size counts whole length of the side, i.e. x:3 to x:5 is len 3.
        match axis {
            Axis::Horizontal => self.bottom_right.x - self.top_left.x + 1,
            Axis::Vertical => self.bottom_right.y - self.top_left.y + 1,
        }
    }

    pub fn get_area(&self) -> i32 {
        let delta_x = self.bottom_right.x - self.top_left.x + 1;
        let delta_y = self.bottom_right.y - self.top_left.y + 1;

        delta_x * delta_y
    }

    pub fn get_inner_area(&self) -> i32 {
        // inner area ignores walls in area calculation
        let inner_delta_x = self.bottom_right.x - self.top_left.x - 1;
        let inner_delta_y = self.bottom_right.y - self.top_left.y - 1;

        if inner_delta_x <= 0 || inner_delta_y <= 0 {
            return 0;
        };

        inner_delta_x * inner_delta_y
    }
}

// Create a new box randomly shrinked from the provided one.
// Used in randomizing size of rooms in a partition from the bsp.
pub fn random_subbox(area: &BoxExtends, min_side_length: i32) -> BoxExtends {
    // calculate shrink budget per axis
    // randomize shrink amount per axis
    // randomize offset per axis
    // return self if I can't shrink

    let distribution = Uniform::new(0.0, 1.0);

    let old_x_size = area.get_axis_size(Axis::Horizontal);
    let old_y_size = area.get_axis_size(Axis::Vertical);

    let shrink_allowance_x = (old_x_size - min_side_length).max(0);
    let shrink_allowance_y = (old_y_size - min_side_length).max(0);

    let random_x: f64 = distribution.sample(&mut thread_rng());
    let random_y: f64 = distribution.sample(&mut thread_rng());

    let shrink_x = (random_x * shrink_allowance_x as f64) as i32;
    let shrink_y = (random_y * shrink_allowance_y as f64) as i32;

    let offset_x: i32;
    match shrink_x <= 0 {
        true => offset_x = 0,
        false => offset_x = thread_rng().gen_range(0..=shrink_x),
    }

    let offset_y: i32;
    match shrink_y <= 0 {
        true => offset_y = 0,
        false => offset_y = thread_rng().gen_range(0..=shrink_y),
    }

    let top_left = Coordinate {
        x: area.top_left.x + offset_x,
        y: area.top_left.y + offset_y,
    };

    let bottom_right = Coordinate {
        x: area.bottom_right.x - shrink_x + offset_x - 1,
        y: area.bottom_right.y - shrink_y + offset_y - 1,
    };

    BoxExtends {
        top_left,
        bottom_right,
    }
}

pub fn split_box(area: &BoxExtends) -> Result<(BoxExtends, BoxExtends)> {
    let threshold = 8.0;
    // which side to split is weighted by side_x / side_y
    let horizontal_size = area.get_axis_size(Axis::Horizontal) as f64;
    let vertical_size = area.get_axis_size(Axis::Vertical) as f64;

    let mut side_ratio = horizontal_size / (horizontal_size + vertical_size);

    match (horizontal_size <= threshold, vertical_size <= threshold) {
        (true, true) => return Err("Can't split further".into()),
        (true, false) => side_ratio = 0.0,
        (false, true) => side_ratio = 1.0,
        (false, false) => {}
    }
    // smoothstep means low probability is lower and high probability is higher
    fn smootherstep(x: f64) -> f64 {
        let x = clamp((x - 0.0) / (1.0 - 0.0), 0.0, 1.0);
        return x * x * x * (x * (6.0 * x - 15.0) + 10.0);
    }

    let split_axis = match thread_rng().gen_bool(smootherstep(side_ratio)) {
        true => Axis::Horizontal,
        false => Axis::Vertical,
    };

    let least_margin = 0.35;

    let mut rng = rand::thread_rng()
        .sample_iter::<f32, _>(rand_distr::StandardNormal)
        .map(|val| val.clamp(least_margin, 1.0 - least_margin));

    let (left, top) = (area.top_left.x, area.top_left.y);
    let (right, bottom) = (area.bottom_right.x, area.bottom_right.y);

    match split_axis {
        Axis::Vertical => {
            let split_point = (rng.next().unwrap() * vertical_size as f32) as i32 + top;

            let upper = BoxExtends {
                top_left: area.top_left,
                bottom_right: Coordinate {
                    x: right,
                    y: split_point,
                },
            };

            let lower = BoxExtends {
                top_left: Coordinate {
                    x: left,
                    y: split_point,
                },
                bottom_right: area.bottom_right,
            };

            Ok((upper, lower))
        }
        Axis::Horizontal => {
            let split_point = (rng.next().unwrap() * horizontal_size as f32) as i32 + left;

            let left = BoxExtends {
                top_left: area.top_left,
                bottom_right: Coordinate {
                    x: split_point,
                    y: bottom,
                },
            };

            let right = BoxExtends {
                top_left: Coordinate {
                    x: split_point,
                    y: top,
                },
                bottom_right: area.bottom_right,
            };

            Ok((left, right))
        }
    }
}

pub fn make_edge_vicinity_boxes(
    area: &BoxExtends,
    scan_distance: i32,
    overlap: i32,
) -> Vec<BoxExtends> {
    // Creates hitboxes that check the sides of 'area' up to 'scan_distance'
    let above = BoxExtends {
        top_left: Coordinate {
            x: area.top_left.x + overlap,
            y: if scan_distance <= area.top_left.y {
                area.top_left.y - scan_distance
            } else {
                0
            },
        },
        bottom_right: Coordinate {
            x: area.bottom_right.x - overlap,
            y: area.top_left.y,
        },
    };

    let below = BoxExtends {
        top_left: Coordinate {
            x: area.top_left.x + overlap,
            y: area.bottom_right.y,
        },
        bottom_right: Coordinate {
            x: area.bottom_right.x - overlap,
            y: area.bottom_right.y + scan_distance,
        },
    };

    let left = BoxExtends {
        top_left: Coordinate {
            x: if scan_distance <= area.top_left.x {
                area.top_left.x - scan_distance
            } else {
                0
            },
            y: area.top_left.y + overlap,
        },
        bottom_right: Coordinate {
            x: area.top_left.x,
            y: area.bottom_right.y - overlap,
        },
    };

    let right = BoxExtends {
        top_left: Coordinate {
            x: area.bottom_right.x,
            y: area.top_left.y + overlap,
        },
        bottom_right: Coordinate {
            x: area.bottom_right.x + scan_distance,
            y: area.bottom_right.y - overlap,
        },
    };

    vec![above, below, left, right]
}
