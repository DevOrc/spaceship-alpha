use super::{
    objects::{Asteroid, Health, ObjectMeshes},
    physics::{Collider, ColliderShape, Hitbox, RigidBody},
    Model, ToBeRemoved, Transform,
};
use crate::item::GameItem;
use cgmath::Vector3;
use rand::{seq::IteratorRandom, Rng};
use specs::{prelude::*, Component};

pub fn register_components(world: &mut World) {
    world.register::<AsteroidField>();
}

pub fn setup_systems(builder: &mut DispatcherBuilder) {
    builder.add(AsteroidFieldSystem, "", &[]);
}

pub fn init_world(world: &mut World) {
    world
        .create_entity()
        .with(AsteroidField {
            asteroids: Vec::new(),
            tick: 0,
            level: 1,
            x_range: 15.0,
        })
        .build();
}

#[derive(Component)]
#[storage(HashMapStorage)]
struct AsteroidField {
    asteroids: Vec<Entity>,
    tick: u16,
    level: u16,
    x_range: f32,
}

struct AsteroidFieldSystem;

impl<'a> System<'a> for AsteroidFieldSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Write<'a, ToBeRemoved>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, AsteroidField>,
        ReadExpect<'a, ObjectMeshes>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, lazy_update, mut to_be_removed, transforms, mut fields, meshes) = data;

        for field in (&mut fields).join() {
            field
                .asteroids
                .retain(|asteroid| entities.is_alive(*asteroid));

            field
                .asteroids
                .iter()
                .filter(|asteroid| {
                    transforms.get(**asteroid).unwrap().position.x.abs() > field.x_range
                })
                .for_each(|asteroid| to_be_removed.add(*asteroid));

            if field.tick > 0 {
                field.tick -= 1;
            } else {
                let max_level = 30;
                if field.level < max_level {
                    println!("Level: {}", field.level);
                    field.level += 1;
                }

                field.tick = 200 - (field.level * 6);

                let mut rng = rand::thread_rng();
                let item = GameItem::iter().choose(&mut rng).unwrap();
                let mut transform = if rng.gen_range(0..(max_level - field.level + 5)) < 4 {
                    println!("Attack!");
                    Transform::from_position(
                        -field.x_range,
                        rng.gen_range(1.0..8.0),
                        rng.gen_range(0.0..5.0),
                    )
                } else {
                    let pos_y: f32 =
                        rng.gen_range(-3.0..3.0) + if rng.gen::<bool>() { 14.0 } else { -10.0 };
                    Transform::from_position(-field.x_range, pos_y, rng.gen_range(6.0..9.0))
                };

                transform.set_rotation_z(rng.gen_range(0.0..crate::PI * 2.0));
                // TODO: Hide Spawning from Camera
                // TODO: Never Spawn collision with ship!
                let entity = lazy_update
                    .create_entity(&entities)
                    .with(transform)
                    .with(Model::new(*meshes.asteroids.get(item).unwrap()))
                    .with(RigidBody {
                        velocity: Vector3::new(Asteroid::VELOCITY, 0.0, 0.0),
                    })
                    .with(Collider::new(
                        Hitbox::with_shape(ColliderShape::Sphere(Asteroid::COLLIDER_RADIUS)),
                        Collider::ASTEROID,
                        vec![Collider::SHIP, Collider::MISSLE],
                    ))
                    .with(Asteroid(*item))
                    .with(Health(Asteroid::HEALTH))
                    .build();
                field.asteroids.push(entity);
            }
        }
    }
}
