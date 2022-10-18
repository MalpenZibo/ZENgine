#[macro_use]
extern crate bencher;

use bencher::Bencher;

use zengine_ecs::{query::QueryIter, World};

#[derive(Debug)]
struct Component1 {
    #[allow(dead_code)]
    data: Vec<u32>,
}
impl zengine_ecs::Component for Component1 {}

#[derive(Debug)]
struct Component2 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine_ecs::Component for Component2 {}

#[derive(Debug)]
struct Component3 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine_ecs::Component for Component3 {}

#[derive(Debug)]
struct Component4 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine_ecs::Component for Component4 {}

fn populate_word(world: &mut World) {
    let mut vector: Vec<u32> = Vec::default();
    for i in 0..1 {
        vector.push(i);
    }

    for _i in 0..4976 {
        world.spawn(Component1 {
            data: vector.clone(),
        });
    }

    for _i in 0..3298 {
        world.spawn((
            Component1 {
                data: vector.clone(),
            },
            Component2 { data: 7 },
            Component4 { data: 2 },
        ));
    }

    for _i in 0..7421 {
        world.spawn((
            Component1 {
                data: vector.clone(),
            },
            Component2 { data: 7 },
            Component3 { data: 3 },
            Component4 { data: 2 },
        ));
    }
}

fn archetype_storage(bench: &mut Bencher) {
    let mut world = World::default();

    populate_word(&mut world);

    bench.iter(|| {
        let mut query = world.query::<(&Component1, &Component2, &Component4)>();
        let query = query.run(&world);
        let iter = query.iter();
        for _d in iter {}
    });
}

benchmark_group!(benches, archetype_storage);
benchmark_main!(benches);
