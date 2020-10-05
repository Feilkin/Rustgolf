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

const EVENT_MARGIN: f64 = 0.001;
const COLLISION_MARGIN: f64 = 0.0000000000001;

#[derive(Clone, Debug)]
pub struct PhysicsBody<T, N: RealField = f64> {
    pub body: T,
    pub position: na::Point2<N>,
    pub velocity: na::Vector2<N>,
    pub acceleration: na::Vector2<N>
}

#[derive(Clone, Debug)]
pub struct Wall<N: RealField = f64> {
    pub start: Point2<N>,
    pub end: Point2<N>
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

#[derive(Debug, Clone)]
pub enum Event {
    BallCollision(usize, usize),
    BallStopped(usize),
}

#[derive(Clone, Debug)]
pub struct Snapshot<N: RealField> {
    pub start_time: Duration,
    pub end_time: Duration,
    pub balls: Vec<PhysicsBody<Ball<N>, N>>,
    pub ignore_collisions: Vec<(usize, usize)>,
    pub index: usize,
    pub walls: Rc<RefCell<Vec<Wall>>>,
}

impl Snapshot<f64> {
    pub fn new(balls: Vec<PhysicsBody<Ball<f64>, f64>>, walls: Rc<RefCell<Vec<Wall<f64>>>>) -> Snapshot<f64> {
        Snapshot {
            start_time: Duration::new(0, 0),
            end_time: Duration::new(u64::MAX, 999_999_999),
            balls,
            ignore_collisions: Vec::new(),
            index: 0,
            walls,
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
        if self.index >= 1000 - 1 {
            return None
        }

        // find next collision
        let mut ignored = Vec::new();
        let mut smallest = std::f64::INFINITY;
        let mut events = Vec::new();

        use mela::itertools::Itertools;

        for (i, ball) in self.balls.iter().enumerate() {
            let stop_t = self.ball_stop_time(ball);

            if stop_t < smallest - EVENT_MARGIN  {
                smallest = stop_t;
                events.clear();
                events.push(Event::BallStopped(i));
            } else if (stop_t - smallest).abs() <= EVENT_MARGIN {
                events.push(Event::BallStopped(i));
            }
        }

        for ((i, ball), (j, other)) in self.balls.iter().enumerate().tuple_combinations() {
            let toi = self.ball_ball_toi(ball, other);

            if let Some(toi) = toi {

                if toi < smallest - EVENT_MARGIN  {
                    if self.ignore_collisions.contains(&(i, j)) || self.ignore_collisions.contains(&(j, i)) {
                        ignored.push((i, j));
                        continue;
                    } else {
                        smallest = toi;
                        ignored.clear();
                        events.clear();
                        events.push(Event::BallCollision(i, j));
                    }
                } else if (toi - smallest).abs() <= EVENT_MARGIN {
                    if self.ignore_collisions.contains(&(i, j)) || self.ignore_collisions.contains(&(j, i)) {
                        ignored.push((i, j));
                        continue;
                    } else {
                        events.push(Event::BallCollision(i, j));
                    }
                }
            }
        }

        if smallest < std::f64::INFINITY {
            self.end_time = self.start_time + Duration::from_secs_f64(smallest);
            let mut new = self.advance_to(smallest);
            new.ignore_collisions = ignored;

            for event in &events {

                match &event {
                    Event::BallCollision(ball, other) => {
                        new.ignore_collisions = new.ignore_collisions.iter().filter(|(a, b)| a != ball && b != ball && a != other && b != other).cloned().collect();
                        new.ignore_collisions.push((*ball, *other));
                        new = new.handle_collision_pair(*ball, *other);
                    },
                    Event::BallStopped(ball) => {
                        new.balls[*ball].velocity = Vector2::new(0., 0.);
                    }
                    _ => {}
                }
            }

            Some(new)
        } else {
            None
        }
    }

    pub fn advance_to(&self, t: f64) -> Snapshot<f64> {
        let mut new_balls = Vec::with_capacity(self.balls.len());

        for ball in &self.balls {
            let acc = self.ball_acceleration(ball);
            let mut new_velocity = Vector2::new(ball.velocity.x + &acc.x * t, ball.velocity.y + &acc.y * t);

            if new_velocity.norm() <= 1.0 {
                new_velocity = Vector2::new(0., 0.)
            }

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
            balls: new_balls,
            index: self.index + 1,
            ignore_collisions: Vec::new(),
            walls: Rc::clone(&self.walls),
        }
    }

    fn handle_collision_pair(mut self, ball: usize, other: usize) -> Snapshot<f64> {
        let x1 = self.balls[ball].position.clone();
        let x2 = self.balls[other].position.clone();
        let v1 = self.balls[ball].velocity.clone_owned();
        let v2 = self.balls[other].velocity.clone_owned();
        let v1v2 = &v1 - &v2;
        let v2v1 = &v2 - &v1;
        let x1x2 = &x1 - &x2;
        let x2x1 = &x2 - &x1;

        self.balls[ball].velocity = &v1 - v1v2.dot(&x1x2)/x1x2.norm_squared()*&x1x2;
        self.balls[other].velocity = &v2 - v2v1.dot(&x2x1)/x2x1.norm_squared()*&x2x1;

        self
    }

    fn ball_stop_time(&self, ball: &PhysicsBody<Ball>) -> f64 {
        if ball.velocity.norm_squared() == 0. {
            f64::INFINITY
        } else {
            if ball.acceleration.norm_squared() != 0. {
                todo!("Acceleration")
            }

            let vel_normalized = ball.velocity.normalize();
            let ff = Vector2::new(0.22 * 9.81 * vel_normalized.x, 0.22 * 9.81 * vel_normalized.y);

            ball.velocity.norm() / ff.norm()
        }
    }

    fn ball_acceleration(&self, ball: &PhysicsBody<Ball>) -> Vector2<f64> {
        if ball.velocity.norm_squared() <= 1.0 {
            Vector2::new(0., 0.)
        } else {
            let vel_normalized = ball.velocity.normalize();
            let ff = Vector2::new(0.22 * 9.81 * vel_normalized.x, 0.22 * 9.81 * vel_normalized.y);

            ball.acceleration.clone_owned() - ff
        }
    }

    fn ball_wall_toi(&self, ball: &PhysicsBody<Ball>, wall: &Wall) -> Option<f64> {
        if wall.start.x == wall.end.x {
            // along y axis
            None
        } else {
            let k = (&wall.end - &wall.start).norm();
            let c = wall.start.y - wall.start.x * k;
            let acc = self.ball_acceleration(ball);
            let a = acc.x * k - acc.y;
            let b = k * ball.velocity.x - ball.velocity.y;
            let c = k * ball.position.x - ball.position.y + c;

            let t1 = (-b + (b.powf(2.) - 2. * a * c).sqrt()) / a;
            let t2 = (-b + (b.powf(2.) - 2. * a * c).sqrt()) / a;

            None
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

        let dax = a1x - a2x;
        let day = a1y - a2y;
        let dx = x1 - x2;
        let dy = y1 - y2;
        let dvx = v1x - v2x;
        let dvy = v1y - v2y;
        let r = Complex::new((ball.body.radius + other.body.radius).powf(2.), 0.);

        let var_127 = dax.powf(2.);
        let var_128 = day.powf(2.);

        if (var_127 + var_128).norm_sqr() == 0. {
            // use constant velocity formula instead
            let var_194 = dvy.powf(2.);
            let var_191 = dvx.powf(2.);
            let var_195 = var_191 + var_194;
            let var_196 = 1./var_195;
            let var_197 = -dvx * dx;
            let var_198 = -dvy * dy;
            let var_202 = dx.powf(2.);
            let var_204 = -var_194 * var_202;
            let var_205 = 2. * dvx * dvy * dx * dy;
            let var_206 = dy.powf(2.);
            let var_207 = -var_191 * var_206;
            let var_208 = var_191 * r;
            let var_209 = var_194 * r;
            let var_210 = var_204 + var_205 + var_207 + var_208 + var_209;
            let var_211 = var_210.sqrt();

            let t1 = var_196 * (var_197 + var_198 - var_211);
            let t2 = var_196 * (var_197 + var_198 + var_211);

            let mut smallest = None;

            for t in &[t1, t2] {
                if t.im.abs() <= 0.0001 && (t.re >= 0. || t.re.abs() <= COLLISION_MARGIN) {
                    if smallest.is_none() {
                        smallest = Some(t.re.max(0.));
                    } else if t.re <= smallest.unwrap() {
                        smallest = Some(t.re.max(0.))
                    }
                }
            }

            return smallest;
        }

        let var_129 = var_127 + var_128;
        let var_131 = dax * dvx;
        let var_132 = day * dvy;
        let var_133 = var_131 + var_132;
        let var_130 = 1./var_129;
        let var_135 = 1./var_129.powf(2.);
        let var_138 = dvx.powf(2.);
        let var_139 = dvy.powf(2.);
        let var_140 = dax * dx;
        let var_141 = day * dy;
        let var_142 = var_138 + var_139 + var_140 + var_141;
        let var_136 = var_133.powf(2.);
        let var_148 = dvx * dx;
        let var_149 = dvy * dy;
        let var_150 = var_148 + var_149;
        let var_156 = var_142.powf(2.);
        let var_157 = 16. * var_156;
        let var_158 = -96. * var_133 * var_150;
        let var_159 = dx.powf(2.);
        let var_160 = dy.powf(2.);
        let var_161 = -r;
        let var_162 = var_159 + var_160 + var_161;
        let var_163 = 48. * var_129 * var_162;
        let var_164 = var_157 + var_158 + var_163;
        let var_165 = var_142.powf(3.);
        let var_166 = 128. * var_165;
        let var_167 = -1152. * var_133 * var_142 * var_150;
        let var_168 = var_150.powf(2.);
        let var_169 = 1728. * var_129 * var_168;
        let var_172 = 1728. * var_136 * var_162;
        let var_173 = -1152. * var_129 * var_142 * var_162;
        let var_170 = var_164.powf(3.);
        let var_171 = -4. * var_170;
        let var_174 = var_166 + var_167 + var_169 + var_172 + var_173;
        let var_175 = var_174.powf(2.);
        let var_176 = var_171 + var_175;
        let var_177 = var_176.sqrt();
        let var_178 = var_166 + var_167 + var_169 + var_177 + var_172 + var_173;
        let var_155 = Complex::new((2f64).cbrt(), 0.);
        let var_179 = 1./var_178.cbrt();
        let var_181 = Complex::new(0.5f64.cbrt(), 0.);
        let var_182 = var_178.cbrt();
        let var_153 = 4. * var_135 * var_136;
        let var_154 = -((8. * var_130 * var_142)/3.);
        let var_180 = 1./3. * var_155 * var_130 * var_164 * var_179;
        let var_183 = (var_181 * var_130 * var_182)/3.;
        let var_184 = var_153 + var_154 + var_180 + var_183;
        let var_134 = -var_130 * var_133;
        let var_137 = 8. * var_135 * var_136;
        let var_143 = -((16. * var_130 * var_142)/ 3.);
        let var_144 = 1./var_129.powf(3.);
        let var_145 = var_133.powf(3.);
        let var_146 = -64. * var_144 * var_145;
        let var_147 = 64. * var_135 * var_133 * var_142;
        let var_151 = -64. * var_130 * var_150;
        let var_152 = var_146 + var_147 + var_151;
        let var_185 = 1./var_184.sqrt();
        let var_186 = -((var_152 * var_185)/ 4.);
        let var_187 = -(1./3.) * var_155 * var_130 * var_164 * var_179;
        let var_188 = -(1./3.) * var_181 * var_130 * var_182;
        let var_189 = var_137 + var_143 + var_186 + var_187 + var_188;
        let var_190 = var_189.sqrt();
        let var_192 = var_184.sqrt();
        let var_199 = (var_152 * var_185)/4.;
        let var_200 = var_137 + var_143 + var_199 + var_187 + var_188;
        let var_201 = var_200.sqrt();
        let var_203 = var_192/2.;

        let t1 = var_134 - var_190 / 2. - var_203;
        let t2 = var_134 + var_190 / 2. - var_203;
        let t3 = var_134 - var_201 / 2. + var_203;
        let t4 = var_134 + var_201 / 2. + var_203;

        let mut smallest = None;

        for t in &[t1, t2, t3, t4] {
            if t.im.abs() <= 0.0001 && (t.re >= 0. || t.re.abs() <= COLLISION_MARGIN) {
                if smallest.is_none() {
                    smallest = Some(t.re.max(0.));
                } else if t.re <= smallest.unwrap() {
                    smallest = Some(t.re.max(0.))
                }
            }
        }

        smallest
    }
}

pub struct PhysicsAnimator<N: RealField> {
    snapshots: Rc<RefCell<Vec<Snapshot<N>>>>,
    timer: Rc<RefCell<Duration>>,
    paused: bool,
}

impl<N> PhysicsAnimator<N> where N: RealField {
    pub fn new(snapshots: Rc<RefCell<Vec<Snapshot<N>>>>, timer: Rc<RefCell<Duration>>) -> PhysicsAnimator<N> {
        PhysicsAnimator {
            snapshots,
            timer,
            paused: true
        }
    }
}

impl<W> System<W> for PhysicsAnimator<f64> where W: World + WorldStorage<Transform<f64>> + WorldStorage<BallComponent> {
    type SystemData<'a> = (Write<'a, Transform<f64>>, Read<'a, BallComponent>);

    fn name(&self) -> &'static str {
        "PhysicsAnimator"
    }

    fn update<'f>(&mut self, (mut transforms, balls): Self::SystemData<'f>, delta: Duration, _io_state: &IoState, _render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) -> () {
        use mela::imgui::im_str;
        let ui = &_debug_ctx.ui;

        let mut current_time = self.timer.borrow_mut();

        if !self.paused {
            *current_time += delta;
        }

        if let Some(current_snapshot) = {
            let snapshots = (*self.snapshots).borrow();
            let mut found = None;
            for snapshot in &*snapshots {
                if snapshot.end_time >= *current_time { found = Some(snapshot.clone()); break }
            }

            found
        } {
            ui.text(im_str!("Snapshot: {}", current_snapshot.index));

            for (entity, mut transform) in transforms.iter_mut() {
                if let Some(ball) = balls.fetch(entity) {
                    let BallComponent {
                        index, hidden
                    } = ball;

                    if !hidden {
                        let (pos, ball) = current_snapshot.ball_pos(*index, *current_time);
                        transform.0 = Isometry2::translation(pos.x, pos.y);
                    }
                }
            }

            if current_snapshot.end_time >= Duration::new(u64::MAX, 0) {
                *current_time = Duration::new(0, 0);
            }
        }


        if self.paused {
            if ui.button(im_str!("Paused"), [80., 25.]) {
                self.paused = false;
            }
            let mut temp = current_time.as_secs_f32();

            if ui.drag_float(im_str!("Timer"), &mut temp)
                .speed(0.1)
                .min(0.)
                .build() {

                *current_time = Duration::from_secs_f32(temp.max(0.));
            }
        } else {
            if ui.button(im_str!("Running"), [80., 25.]) {
                self.paused = true;
            }
            ui.text(im_str!("Timer: {:?}", &current_time));
        }
    }
}