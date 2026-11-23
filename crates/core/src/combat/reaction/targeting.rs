use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Point3D {
    pub x_mm: i64,
    pub y_mm: i64,
    pub z_mm: i64,
}

impl Point3D {
    pub fn distance_squared(&self, other: &Point3D) -> u128 {
        let dx = (i128::from(self.x_mm) - i128::from(other.x_mm)).unsigned_abs();
        let dy = (i128::from(self.y_mm) - i128::from(other.y_mm)).unsigned_abs();
        let dz = (i128::from(self.z_mm) - i128::from(other.z_mm)).unsigned_abs();
        // A saturated distance still exceeds every representable radius squared.
        (dx * dx).saturating_add(dy * dy).saturating_add(dz * dz)
    }

    pub fn is_within_radius(&self, other: &Point3D, radius_mm: u64) -> bool {
        let r = radius_mm as u128;
        self.distance_squared(other) <= r * r
    }

    pub fn interpolate(&self, target: &Point3D, elapsed_us: u64, duration_us: u64) -> Point3D {
        if duration_us == 0 || elapsed_us >= duration_us {
            return *target;
        }
        let num = elapsed_us as u128;
        let den = duration_us as u128;
        let interp = |a: i64, b: i64| -> i64 {
            let diff = (b as i128) - (a as i128);
            let magnitude = (diff.unsigned_abs() * num / den) as i128;
            let step = if diff < 0 { -magnitude } else { magnitude };
            (a as i128 + step) as i64
        };
        Point3D {
            x_mm: interp(self.x_mm, target.x_mm),
            y_mm: interp(self.y_mm, target.y_mm),
            z_mm: interp(self.z_mm, target.z_mm),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetingRule {
    SingleTarget {
        target_id: String,
    },
    AllTargets,
    SplashRadius {
        center_target: String,
        radius_mm: u64,
    },
}
