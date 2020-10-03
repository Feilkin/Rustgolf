//! Analytical physics engine

use mela::nalgebra as na;
use mela::ecs::{System, Component};
use mela::gfx::RenderContext;
use mela::game::IoState;
use std::time::{Instant, Duration};
use mela::debug::DebugContext;
use mela::ecs::system::{Read, Write};
use std::rc::Rc;
use mela::nphysics::ncollide2d::simba::scalar::RealField;
use mela::ecs::component::Transform;
use mela::nalgebra::{Point2, Similarity2, Vector2, Isometry3, Isometry2};
use mela::ecs::world::{World, WorldStorage};
use std::ops::Mul;
use std::cell::RefCell;
use std::borrow::Borrow;

#[derive(Clone, Debug)]
pub struct PhysicsBody<T, N: RealField = f64> {
    pub body: T,
    pub position: na::Point2<N>,
    pub velocity: na::Vector2<N>,
    pub acceleration: na::Vector2<N>
}

#[derive(Clone, Debug)]
pub struct Ball<N: RealField = f64> {
    pub radius: N
}

#[derive(Clone, Debug)]
pub struct BallComponent {
    pub index: usize,
    pub hidden: bool
}

impl Component for BallComponent {}

#[derive(Clone, Debug)]
pub struct Snapshot<N: RealField> {
    pub start_time: Duration,
    pub end_time: Duration,
    pub balls: Vec<PhysicsBody<Ball<N>, N>>
}

impl Snapshot<f64> {
    pub fn new(balls: Vec<PhysicsBody<Ball<f64>, f64>>) -> Snapshot<f64> {
        Snapshot {
            start_time: Duration::new(0, 0),
            end_time: Duration::new(u64::MAX, 999_999_999),
            balls
        }
    }

    pub fn ball_pos(&self, ball_index: usize, t: Duration) -> (Point2<f64>, &Ball<f64>) {
        let body = &self.balls[ball_index];
        let delta = (t - self.start_time).as_secs_f64();

        let acc = self.ball_acceleration(body);

        let x = &body.position.x + &body.velocity.x * delta + 0.5 * &acc.x * delta.powf(2.);
        let y = &body.position.y + &body.velocity.y * delta + 0.5 * &acc.y * delta.powf(2.);

        (Point2::new(x, y), &body.body)
    }

    pub fn next_snapshot(&mut self) -> Option<Snapshot<f64>> {
        // find next collision
        let mut smallest = std::f64::INFINITY;
        let mut first_contact_pair = None;

        for (i, ball) in self.balls.iter().enumerate() {
            for (j, other) in self.balls.iter().enumerate() {
                if i == j {
                    continue;
                }

                let toi = self.ball_ball_toi(ball, other);

                if let Some(toi) = toi {
                    if toi <= smallest {
                        smallest = toi;
                        first_contact_pair = Some((i, j));
                    }
                }
            }
        }

        if smallest < std::f64::INFINITY {
            self.end_time = Duration::from_secs_f64(smallest);
            let new = self.advance_to(smallest).handle_collision_pair(first_contact_pair.unwrap());

            Some(new)
        } else {
            None
        }
    }

    fn advance_to(&self, t: f64) -> Snapshot<f64> {
        let mut new_balls = Vec::with_capacity(self.balls.len());

        for ball in &self.balls {
            let acc = self.ball_acceleration(ball);
            let new_velocity = Vector2::new(ball.velocity.x + &ball.acceleration.x * t, ball.velocity.y + &ball.acceleration.y * t);

            new_balls.push(PhysicsBody {
                body: ball.body.clone(),
                position: &ball.position + Vector2::new(&ball.velocity.x * t + 0.5 * &acc.x * t.powf(2.), &ball.velocity.y * t + 0.5 * &acc.y * t.powf(2.)),
                velocity: new_velocity,
                acceleration: ball.acceleration.clone_owned()
            });
        }

        Snapshot {
            start_time: self.end_time,
            end_time: Duration::new(u64::MAX, 999_999_999),
            balls: new_balls
        }
    }

    fn handle_collision_pair(mut self, (ball, other): (usize, usize)) -> Snapshot<f64> {
        self.balls[ball].velocity = Vector2::new(0., 0.);
        self.balls[other].velocity = Vector2::new(0., 0.);
        self.balls[ball].acceleration = Vector2::new(0., 0.);
        self.balls[other].acceleration = Vector2::new(0., 0.);

        self
    }

    fn ball_acceleration(&self, ball: &PhysicsBody<Ball>) -> Vector2<f64> {
        if ball.velocity.norm_squared() == 0. {
            Vector2::new(0., 0.)
        } else {
            let vel_normalized = ball.velocity.normalize();
            let ff = Vector2::new(0.0 * 9.81 * vel_normalized.x, 0.22 * 9.81 * vel_normalized.y);

            ball.acceleration.clone_owned()
        }
    }

    fn ball_ball_toi(&self, ball: &PhysicsBody<Ball>, other: &PhysicsBody<Ball>) -> Option<f64> {
        use num::Complex;

        let acc = self.ball_acceleration(ball);
        let (a1x, a1y) = (Complex::new(acc.x, 0.), Complex::new(acc.y, 0.));
        let (v1x, v1y) = (Complex::new(ball.velocity.x, 0.),  Complex::new(ball.velocity.y, 0.));
        let (x1, y1) = (Complex::new(ball.position.x, 0.), Complex::new(ball.position.y, 0.));

        let other_acc = self.ball_acceleration(other);
        let (a2x, a2y) = (Complex::new(other_acc.x, 0.), Complex::new(other_acc.y, 0.));
        let (v2x, v2y) = (Complex::new(other.velocity.x, 0.), Complex::new(other.velocity.y, 0.));
        let (x2, y2) = (Complex::new(other.position.x, 0.), Complex::new(other.position.y, 0.));
        let r = Complex::new((ball.body.radius + other.body.radius).powf(2.), 0.);

        let var_1 = a1x.powf(2.);
        let var_9 = a1y.powf(2.);
        let var_23 = -2. * a1x * a2x;
        let var_39 = a2x.powf(2.);
        let var_40 = -2. * a1y * a2y;
        let var_89 = a2y.powf(2.);
        let var_90 = var_1 + var_9 + var_23 + var_39 + var_40 + var_89;

        if var_90.norm_sqr() == 0. {
            return None;
        }

        let var_92 = a1x * v1x;
        let var_93 = -a2x * v1x;
        let var_94 = a1y * v1y;
        let var_95 = -a2y * v1y;
        let var_96 = -a1x * v2x;
        let var_97 = a2x * v2x;
        let var_98 = -a1y * v2y;
        let var_99 = a2y * v2y;
        let var_100 = var_92 + var_93 + var_94 + var_95 + var_96 + var_97 + var_98 + var_99;
        let var_91 = 1./var_90;
        let var_105 = v1x.powf(2.);
        let var_106 = v1y.powf(2.);
        let var_107 = -2. * v1x * v2x;
        let var_108 = v2x.powf(2.);
        let var_109 = -2. * v1y * v2y;
        let var_110 = v2y.powf(2.);
        let var_111 = a1x * x1;
        let var_112 = -a2x * x1;
        let var_113 = -a1x * x2;
        let var_114 = a2x * x2;
        let var_115 = a1y * y1;
        let var_116 = -a2y * y1;
        let var_117 = -a1y * y2;
        let var_118 = a2y * y2;
        let var_119 = var_105 + var_106 + var_107 + var_108 + var_109 + var_110 + var_111 + var_112 + var_113 + var_114 + var_115 + var_116 +  var_117 + var_118;
        let var_124 = v1x * x1;
        let var_125 = -v2x * x1;
        let var_126 = -v1x * x2;
        let var_127 = v2x * x2;
        let var_128 = v1y * y1;
        let var_129 = -v2y * y1;
        let var_130 = -v1y * y2;
        let var_131 = v2y * y2;
        let var_132 = var_124 + var_125 + var_126 + var_127 +  var_128 + var_129 + var_130 + var_131;
        let var_103 = var_100.powf(2.);
        let var_134 = x1.powf(2.);
        let var_135 = -var_134;
        let var_136 = 2. * x1 * x2;
        let var_137 = x2.powf(2.);
        let var_138 = -var_137;
        let var_139 = y1.powf(2.);
        let var_140 = -var_139;
        let var_141 = 2. * y1 * y2;
        let var_142 = y2.powf(2.);
        let var_143 = -var_142;
        let var_144 = r + var_135 + var_136 + var_138 + var_140 + var_141 + var_143;
        let var_122 = var_119.powf(2.);
        let var_123 = 16. * var_122;
        let var_133 = -96. * var_100 * var_132;
        let var_145 = -48. * var_90 * var_144;
        let var_146 = var_123 + var_133 + var_145;
        let var_147 = var_119.powf(3.);
        let var_148 = 128. * var_147;
        let var_149 = -1152. * var_100 * var_119 * var_132;
        let var_150 = var_132.powf(2.);
        let var_151 = 1728. * var_90 * var_150;
        let var_152 = -1728. * var_103 * var_144;
        let var_153 = 1152. * var_90 * var_119 * var_144;
        let var_154 = var_146.powf(3.);
        let var_155 = -4. * var_154;
        let var_156 = var_148 + var_149 + var_151 + var_152 + var_153;
        let var_157 = var_156.powf(2.);
        let var_158 = var_155 + var_157;

        let var_159 = var_158.sqrt();
        let var_160 = var_148 + var_149 + var_151 + var_152 + var_153 + var_159;

        if var_160.norm_sqr() == 0. {
            return None;
        }

        let var_102 = 1./var_90.powf(2.);
        let var_121 = 2_f64.cbrt();
        let var_161 = 1./var_160.cbrt();
        let var_163 = 1./var_121;
        let var_164 = var_160.cbrt();
        let var_104 = 4. * var_102 * var_103;
        let var_120 = -((8. * var_91 * var_119)/3.);
        let var_162 = 1./3. * var_121 * var_91 * var_146 * var_161;
        let var_165 = (var_163 * var_91 * var_164)/3.;
        let var_166 = var_104 + var_120 + var_162 + var_165;


        if var_166.norm_sqr() == 0. {
            return None
        }

        let var_101 = -var_91 * var_100;
        let var_167 = var_166.sqrt();
        let var_168 = -(var_167/2.);
        let var_169 = 8. * var_102 * var_103;
        let var_170 = -((16. * var_91 * var_119)/3.);
        let var_171 = -(1./3.) * var_121 * var_91 * var_146 * var_161;
        let var_172 = -(1./3.) * var_163 * var_91 * var_164;
        let var_173 = 1./var_90.powf(3.);
        let var_174 = var_100.powf(3.);
        let var_175 = -64. * var_173 * var_174;
        let var_176 = 64. * var_102 * var_100 * var_119;
        let var_177 = -64. * var_91 * var_132;
        let var_178 = var_175 + var_176 + var_177;
        let var_179 = 1./var_166.sqrt();
        let var_180 = -((var_178 * var_179)/4.);
        let var_181 = var_169 + var_170 + var_171 + var_172 + var_180;

        let var_182 = var_181.sqrt();
        let var_189 = var_167/2.;
        let var_190 = (var_178 * var_179)/4.;
        let var_191 = var_169 + var_170 + var_171 + var_172 + var_190;

        let var_192 = var_191.sqrt();

        let t1 = var_101 + var_168 - var_182 / 2.;
        let t2 = var_101 + var_168 + var_182 / 2.;
        let t3 = var_101 - var_168 - var_192 / 2.;
        let t4 = var_101 - var_168 + var_192 / 2.;

        let mut smallest = None;

        for t in &[t1, t2, t3, t4] {
            if t.im.abs() <= 0.0001 && t.re >= 0. {
                if smallest.is_none() {
                    smallest = Some(t.re);
                } else if t.re <= smallest.unwrap() {
                    smallest = Some(t.re)
                }
            }
        }

        smallest
    }
}

pub struct PhysicsAnimator<N: RealField> {
    snapshots: Rc<RefCell<Vec<Snapshot<N>>>>,
    timer: Duration,
}

impl<N> PhysicsAnimator<N> where N: RealField {
    pub fn new(snapshots: Rc<RefCell<Vec<Snapshot<N>>>>) -> PhysicsAnimator<N> {
        PhysicsAnimator {
            snapshots,
            timer: Duration::new(0, 0)
        }
    }
}

impl<W> System<W> for PhysicsAnimator<f64> where W: World + WorldStorage<Transform<f64>> + WorldStorage<BallComponent> {
    type SystemData<'a> = (Write<'a, Transform<f64>>, Read<'a, BallComponent>);

    fn name(&self) -> &'static str {
        "PhysicsAnimator"
    }

    fn update<'f>(&mut self, (mut transforms, balls): Self::SystemData<'f>, delta: Duration, _io_state: &IoState, _render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) -> () {
        let current_time = self.timer + delta;
        if let Some(current_snapshot) = {
            let snapshots = (*self.snapshots).borrow();
            let mut found = None;
            for snapshot in &*snapshots {
                if snapshot.end_time >= current_time { found = Some(snapshot.clone()); break }
            }

            found.or_else(|| snapshots.last().cloned())
        } {
            for (entity, mut transform) in transforms.iter_mut() {
                if let Some(ball) = balls.fetch(entity) {
                    let BallComponent {
                        index, hidden
                    } = ball;

                    if !hidden {
                        let (pos, ball) = current_snapshot.ball_pos(*index, current_time);
                        transform.0 = Isometry2::translation(pos.x, pos.y);
                    }
                }
            }

            self.timer = current_time;
        }
    }
}