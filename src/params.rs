use crate::{
    collision::{BVHNode, Hitable, SpheresSoA},
    scene::{Scene, World},
    storage::Storage,
};
use glam::Vec3;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub random_seed: bool,
    pub use_bvh: bool,
    pub use_soa: bool,
}

impl Params {
    pub fn new_rng(&self) -> Xoshiro256Plus {
        if self.random_seed {
            Xoshiro256Plus::seed_from_u64(rand::random())
        } else {
            Xoshiro256Plus::seed_from_u64(0)
        }
    }

    pub fn new_scene<'a>(
        &self,
        rng: &mut Xoshiro256Plus,
        storage: &'a Storage<'a>,
        mut hitables: Vec<Hitable<'a>>,
        sky: Option<Vec3>,
    ) -> Scene<'a> {
        if self.use_soa {
            assert!(
                hitables.iter().all(|h| matches!(h, Hitable::Sphere(..))),
                "--soa only supports all-sphere presets"
            );
            return Scene::new(World::Spheres(SpheresSoA::new(&hitables)), sky);
        }

        let hitable_list = if self.use_bvh {
            let bvh_root = BVHNode::new(rng, &mut hitables, &storage.bvhnode_arena).unwrap();
            dbg!(bvh_root.get_stats());

            Hitable::BVHNode(bvh_root)
        } else {
            Hitable::List(storage.alloc_hitables(hitables))
        };

        Scene::new(World::Hitable(hitable_list), sky)
    }
}
