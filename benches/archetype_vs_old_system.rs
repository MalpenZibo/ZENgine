#[macro_use]
extern crate bencher;

use bencher::Bencher;
use zengine::core::join::Join;
use zengine::core::system::Data;
use zengine::core::system::ReadSet;
use zengine::core::Store;

use zengine_ecs::query::QueryIter;
use zengine_ecs::world::World;

#[derive(Debug)]
struct Component1 {
    #[allow(dead_code)]
    data: Vec<u32>,
}
impl zengine::core::Component for Component1 {}
impl zengine_ecs::component::Component for Component1 {}

#[derive(Debug)]
struct Component2 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine::core::Component for Component2 {}
impl zengine_ecs::component::Component for Component2 {}

#[derive(Debug)]
struct Component3 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine::core::Component for Component3 {}
impl zengine_ecs::component::Component for Component3 {}

#[derive(Debug)]
struct Component4 {
    #[allow(dead_code)]
    data: usize,
}
impl zengine::core::Component for Component4 {}
impl zengine_ecs::component::Component for Component4 {}

fn populate_store(store: &mut Store) {
    let mut vector: Vec<u32> = Vec::default();
    for i in 0..1 {
        vector.push(i);
    }

    for _i in 0..4976 {
        store
            .build_entity()
            .with(Component1 {
                data: vector.clone(),
            })
            .build();
    }

    for _i in 0..3298 {
        store
            .build_entity()
            .with(Component1 {
                data: vector.clone(),
            })
            .with(Component2 { data: 7 })
            .with(Component4 { data: 2 })
            .build();
    }

    for _i in 0..7421 {
        store
            .build_entity()
            .with(Component1 {
                data: vector.clone(),
            })
            .with(Component2 { data: 7 })
            .with(Component3 { data: 3 })
            .with(Component4 { data: 2 })
            .build();
    }
}

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

type Data1<'a> = (
    ReadSet<'a, Component1>,
    ReadSet<'a, Component2>,
    ReadSet<'a, Component3>,
    ReadSet<'a, Component4>,
);

fn hashmap_storage(bench: &mut Bencher) {
    let mut store = Store::default();

    populate_store(&mut store);

    bench.iter(|| {
        let (component1, component2, _component3, component4) = Data1::fetch(&store);
        let iter = component1.join((&component2, &component4));
        for _d in iter {}
    });
}

fn archetype_storage(bench: &mut Bencher) {
    let mut world = World::default();

    populate_word(&mut world);

    bench.iter(|| {
        let mut query = world.query::<(&Component1, &Component2, &Component4)>();
        let iter = query.iter();
        for _d in iter {}
    });
}

benchmark_group!(benches, hashmap_storage, archetype_storage);
benchmark_main!(benches);
