//! bodies & stuff

use mela::ecs::{Component, Entity};
use mela::nalgebra::{Point2, Vector2};
use mela::ncollide2d::query::Contact;
use std::ops::Deref;

#[derive(Debug)]
pub enum Shape {
    Ball { radius: f32 },
}

/// Position of physics body (its center point)
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Point2<f32>);

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position(Point2::new(x, y))
    }
}

impl Deref for Position {
    type Target = Point2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Point2<f32>> for Position {
    fn from(p: Point2<f32>) -> Self {
        Position(p)
    }
}

impl Component for Position {}

#[derive(Debug, Clone, Copy)]
pub struct Velocity(Vector2<f32>);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Velocity {
        Velocity(Vector2::new(x, y))
    }

    pub fn new_from(vector: Vector2<f32>) -> Velocity {
        Velocity(vector)
    }
}

impl Component for Velocity {}

impl Deref for Velocity {
    type Target = Vector2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vector2<f32>> for Velocity {
    fn from(v: Vector2<f32>) -> Self {
        Velocity(v)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acceleration(Vector2<f32>);

impl Acceleration {
    pub fn new(x: f32, y: f32) -> Acceleration {
        Acceleration(Vector2::new(x, y))
    }
}

impl Component for Acceleration {}

impl Deref for Acceleration {
    type Target = Vector2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vector2<f32>> for Acceleration {
    fn from(v: Vector2<f32>) -> Self {
        Acceleration(v)
    }
}

#[derive(Debug)]
pub enum PhysicsEvent {
    Collision {
        cause: Entity,
        other: Entity,
        contact: Contact<f32>,
        toi: f32,
    },
    CollisionEnded {
        cause: Entity,
        other: Entity,
    }
}

impl Component for PhysicsEvent {}
