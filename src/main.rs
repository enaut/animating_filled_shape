use std::time::Duration;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{
    component_animator_system, Animator, EaseFunction, Lens, Sequence, Tween, TweeningPlugin,
    TweeningType,
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup_system)
        .add_system(component_animator_system::<Path>)
        .run();
}

fn setup_system(mut commands: Commands) {
    // the list of lines that will be drawn... there are lines and arcs (`Circle`)
    let ops = vec![
        GElem::Circle {
            center: Vec2::ZERO,
            radii: Vec2::splat(100.),
            angle: 90.,
        },
        GElem::Line {
            start: Vec2::new(0., 100.),
            target: Vec2::splat(200.),
        },
        GElem::Line {
            start: Vec2::splat(200.),
            target: Vec2::new(100., 0.),
        },
        GElem::Circle {
            center: Vec2::ZERO,
            radii: Vec2::splat(100.),
            angle: -90.,
        },
        GElem::Line {
            start: Vec2::new(0., -100.),
            target: Vec2::new(200., -200.),
        },
        GElem::Line {
            start: Vec2::new(200., -200.),
            target: Vec2::new(100., 0.),
        },
    ];

    commands.spawn_bundle(Camera2dBundle::default());

    // Generate the sequence of animation steps.
    let mut seq = Sequence::with_capacity(ops.len());
    for (step, op) in ops.clone().into_iter().enumerate() {
        let mut done = ops.clone();
        done.truncate(step);
        let next = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            Duration::from_millis(1000),
            FilledAnimation { current: op, done },
        );
        seq = seq.then(next);
    }
    let animator = Animator::new(seq);

    // add the geometry that will be modified by the animation builder.
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &PathBuilder::new().build(),
            DrawMode::Outlined {
                fill_mode: FillMode {
                    options: FillOptions::default(),
                    color: Default::default(),
                },
                outline_mode: StrokeMode::new(Color::BLACK, 3.),
            },
            Transform::default(),
        ))
        .insert(animator);
}

/// now only lines and circles, but will be extended later.
#[derive(Clone, Copy)]
enum GElem {
    Circle {
        center: Vec2,
        radii: Vec2,
        angle: f32,
    },
    Line {
        start: Vec2,
        target: Vec2,
    },
}

impl GElem {
    /// Draw the graphical segment to a given builder.
    fn draw_to_builder(self, b: &mut PathBuilder) {
        match self {
            GElem::Circle {
                center,
                radii,
                angle,
            } => {
                b.arc(center, radii, angle.to_radians(), 0.);
            }
            GElem::Line { target, start: _ } => {
                b.line_to(target);
            }
        }
    }
}

/// The animation needs access to the segments that were already animated as well as the current segment. The Segments in done will be drawn completely, the `current` one will be drawn according to the ratio in lerp.
struct FilledAnimation {
    current: GElem,
    done: Vec<GElem>,
}

impl Lens<Path> for FilledAnimation {
    fn lerp(&mut self, target: &mut Path, ratio: f32) {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2::new(100., 0.));
        for x in &self.done {
            x.draw_to_builder(&mut path_builder)
        }

        let part = match self.current {
            GElem::Circle {
                center,
                radii,
                angle,
            } => GElem::Circle {
                center,
                radii,
                angle: angle * ratio,
            },
            GElem::Line { target, start } => GElem::Line {
                target: start + ((target - start) * ratio),
                start,
            },
        };
        part.draw_to_builder(&mut path_builder);
        *target = path_builder.build();
    }
}
