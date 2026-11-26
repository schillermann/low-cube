use glam::{Vec2, Vec3};
use world::MeshTarget;
use crate::drive::{FlightKinematics, TransferArc};
use super::overlays::OpticOverlay;
use super::sight_stage::{COMPASS_RADIUS, HudSightStage};

const COURSE_EMERALD: Vec3 = Vec3::new(0.36, 1.00, 0.76);
const COURSE_AMBER: Vec3 = Vec3::new(1.00, 0.76, 0.24);

/// `OpticOverlay` decorator stamping `ON COURSE` / `OFF COURSE` corridor telemetry below the compass ring (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CourseAlignmentOverlay<O> {
    origin: O,
    stage: HudSightStage,
    kinematics: FlightKinematics,
    arc: TransferArc,
}

impl<O> CourseAlignmentOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: O,
        stage: HudSightStage,
        kinematics: FlightKinematics,
        arc: TransferArc,
    ) -> Self {
        Self { origin, stage, kinematics, arc }
    }
}

impl<O: OpticOverlay> OpticOverlay for CourseAlignmentOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let pen = self.stage.pen();
        self.kinematics.transfer(|ship_pos, nose_axis, velocity| {
            let flight_vec = if velocity.length_squared() > 2.25 {
                velocity.normalize_or_zero()
            } else {
                nose_axis.normalize_or_zero()
            };
            let telemetry = self.arc.sample_telemetry(ship_pos, flight_vec);
            telemetry.transfer(|_, _, angle_deg, aligned| {
                let (col, text) = if aligned {
                    (COURSE_EMERALD, format!("ON COURSE {:.1}", angle_deg))
                } else {
                    (COURSE_AMBER, format!("OFF COURSE {:.1}", angle_deg))
                };
                pen.emit_anchor_caption(
                    &text,
                    Vec2::new(0.0, -COMPASS_RADIUS - 0.056),
                    (0.0044, 0.0064),
                    col,
                    target,
                );
            });
        });
    }
}
