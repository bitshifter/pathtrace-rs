use crate::{
    collision::{Hitable, Ray, RayHit, AABB},
    material::Material,
};
use rand::{Rng, XorShiftRng};
use typed_arena::Arena;

const MISS_OR_HIT: [&str; 2] = ["Miss", "Hit"];

#[derive(Copy, Clone, Debug, Default)]
pub struct BVHStats {
    num_nodes: usize,
    max_depth: usize,
    num_spheres: usize,
    num_rects: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct BVHNode<'a> {
    aabb: AABB,
    lhs: Hitable<'a>,
    rhs: Hitable<'a>,
}

impl<'a> BVHNode<'a> {
    #[inline]
    pub fn bounding_box(&self) -> AABB {
        self.aabb
    }

    #[inline]
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        if self.aabb.ray_hit(ray, t_min, t_max) {
            let hit_lhs = self.lhs.ray_hit(ray, t_min, t_max);
            let hit_rhs = self.rhs.ray_hit(ray, t_min, t_max);
            match (hit_lhs, hit_rhs) {
                (Some(hit_lhs), Some(hit_rhs)) => {
                    if hit_lhs.0.t < hit_rhs.0.t {
                        Some(hit_lhs)
                    } else {
                        Some(hit_rhs)
                    }
                }
                (Some(hit_lhs), None) => Some(hit_lhs),
                (None, Some(hit_rhs)) => Some(hit_rhs),
                (None, None) => None,
            }
        } else {
            None
        }
    }

    pub fn new(
        rng: &mut XorShiftRng,
        hitables: &mut [Hitable<'a>],
        arena: &'a Arena<BVHNode<'a>>,
    ) -> Option<&'a BVHNode<'a>> {
        let t0 = 0.0;
        let t1 = 0.0;
        match hitables.len() {
            0 => None,
            1 => {
                // special case for 1 hitable where lhs == rhs
                let lhs = hitables[0];
                let rhs = lhs;
                let aabb = lhs.bounding_box(t0, t1).unwrap();
                Some(BVHNode::alloc_bvhnode(arena, lhs, rhs, aabb))
            }
            2 => {
                // special case for 2 hitables returns a single node
                let lhs = hitables[0];
                let rhs = hitables[1];
                let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
                let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
                let aabb = lhs_aabb.add(&rhs_aabb);
                Some(BVHNode::alloc_bvhnode(arena, lhs, rhs, aabb))
            }
            _ => {
                // create a new bvh root node
                Some(BVHNode::new_root(rng, hitables, arena, t0, t1))
            }
        }
    }

    pub fn print_ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) {
        let mut stats = BVHStats::default();
        println!("Starting ray trace {:?}", ray);
        let ray_hit = self.print_ray_hit_node(0, &mut stats, ray, t_min, t_max);
        println!("Result: {:?}", ray_hit);
        println!("Visit status: {:?}", stats);
    }

    fn print_ray_hit_node(
        &self,
        depth: usize,
        stats: &mut BVHStats,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material)> {
        stats.num_nodes += 1;
        let hit = self.aabb.ray_hit(ray, t_min, t_max);
        println!(
            "{:+2$}BVHNode {1} {3}! min: {4:?} max: {5:?}",
            "", stats.num_nodes, depth, MISS_OR_HIT[hit as usize], self.aabb.min, self.aabb.max
        );
        if hit {
            let hit_lhs = self.print_ray_hit_child(depth, stats, &self.lhs, ray, t_min, t_max);
            let hit_rhs = self.print_ray_hit_child(depth, stats, &self.rhs, ray, t_min, t_max);
            match (hit_lhs, hit_rhs) {
                (Some(hit_lhs), Some(hit_rhs)) => {
                    if hit_lhs.0.t < hit_rhs.0.t {
                        Some(hit_lhs)
                    } else {
                        Some(hit_rhs)
                    }
                }
                (Some(hit_lhs), None) => Some(hit_lhs),
                (None, Some(hit_rhs)) => Some(hit_rhs),
                (None, None) => None,
            }
        } else {
            None
        }
    }

    fn print_ray_hit_child(
        &self,
        depth: usize,
        stats: &mut BVHStats,
        hitable: &'a Hitable,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material)> {
        match hitable {
            Hitable::BVHNode(node) => {
                return node.print_ray_hit_node(depth + 1, stats, ray, t_min, t_max);
            }
            Hitable::Sphere(sphere, material) => {
                stats.num_spheres += 1;
                let ray_hit = sphere.ray_hit(ray, t_min, t_max);
                println!(
                    " {:+2$}Sphere {1} centre: {4:?} radius: {5} hit: {3:?}",
                    "", stats.num_spheres, depth, ray_hit, sphere.centre, sphere.radius
                );
                if let Some(ray_hit) = ray_hit {
                    return Some((ray_hit, material));
                }
            }
            Hitable::XYRect(rect, material) => {
                stats.num_rects += 1;
                let ray_hit = rect.ray_hit(ray, t_min, t_max);
                println!(
                    " {:+2$}XYRect {1} {3}!",
                    "",
                    stats.num_rects,
                    depth,
                    MISS_OR_HIT[ray_hit.is_some() as usize]
                );
                if let Some(ray_hit) = ray_hit {
                    return Some((ray_hit, material));
                }
            }
            Hitable::List(_) => unimplemented!(),
        }
        None
    }

    pub fn get_stats(&self) -> BVHStats {
        let mut stats = BVHStats::default();
        stats.max_depth = self.get_node_stats(0, &mut stats);
        stats
    }

    pub fn get_node_stats(&self, depth: usize, stats: &mut BVHStats) -> usize {
        stats.num_nodes += 1;
        let lhs_depth = self.get_child_stats(depth, &self.lhs, stats);
        let rhs_depth = self.get_child_stats(depth, &self.rhs, stats);
        lhs_depth.max(rhs_depth)
    }

    pub fn get_child_stats(&self, depth: usize, hitable: &Hitable, stats: &mut BVHStats) -> usize {
        match hitable {
            Hitable::BVHNode(node) => {
                return node.get_node_stats(depth + 1, stats);
            }
            Hitable::Sphere(_, _) => {
                stats.num_spheres += 1;
            }
            Hitable::XYRect(_, _) => {
                stats.num_rects += 1;
            }
            Hitable::List(_) => unimplemented!(),
        }
        depth
    }

    #[inline]
    fn sort_by_axis(rng: &mut XorShiftRng, hitables: &mut [Hitable<'a>], t0: f32, t1: f32) {
        let axis = rng.next_u32() % 3;
        hitables.sort_unstable_by(|lhs, rhs| {
            let lhs_min = lhs.bounding_box(t0, t1).unwrap().min;
            let rhs_min = rhs.bounding_box(t0, t1).unwrap().min;
            let ord = match axis {
                0 => lhs_min.get_x().partial_cmp(&rhs_min.get_x()),
                1 => lhs_min.get_y().partial_cmp(&rhs_min.get_y()),
                2 => lhs_min.get_z().partial_cmp(&rhs_min.get_z()),
                _ => unreachable!(),
            };
            ord.unwrap()
        });
    }

    #[inline]
    fn new_root(
        rng: &mut XorShiftRng,
        hitables: &mut [Hitable<'a>],
        arena: &'a Arena<BVHNode<'a>>,
        t0: f32,
        t1: f32,
    ) -> &'a BVHNode<'a> {
        BVHNode::sort_by_axis(rng, hitables, t0, t1);
        BVHNode::new_node(rng, hitables, arena, t0, t1)
    }

    #[inline]
    fn new_node(
        rng: &mut XorShiftRng,
        hitables: &mut [Hitable<'a>],
        arena: &'a Arena<BVHNode<'a>>,
        t0: f32,
        t1: f32,
    ) -> &'a BVHNode<'a> {
        let pivot = hitables.len() / 2;
        let lhs = BVHNode::new_split(rng, &mut hitables[..pivot], arena, t0, t1);
        let rhs = BVHNode::new_split(rng, &mut hitables[pivot..], arena, t0, t1);
        let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
        let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
        let aabb = lhs_aabb.add(&rhs_aabb);
        BVHNode::alloc_bvhnode(arena, lhs, rhs, aabb)
    }

    fn new_split(
        rng: &mut XorShiftRng,
        hitables: &mut [Hitable<'a>],
        arena: &'a Arena<BVHNode<'a>>,
        t0: f32,
        t1: f32,
    ) -> Hitable<'a> {
        BVHNode::sort_by_axis(rng, hitables, t0, t1);
        match hitables.len() {
            0 => unreachable!(),
            1 => hitables[0],
            2 => {
                let lhs = hitables[0];
                let rhs = hitables[1];
                let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
                let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
                let aabb = lhs_aabb.add(&rhs_aabb);
                Hitable::BVHNode(BVHNode::alloc_bvhnode(arena, lhs, rhs, aabb))
            }
            _ => Hitable::BVHNode(BVHNode::new_node(rng, hitables, arena, t0, t1)),
        }
    }

    #[inline]
    fn alloc_bvhnode(
        arena: &'a Arena<BVHNode<'a>>,
        lhs: Hitable<'a>,
        rhs: Hitable<'a>,
        aabb: AABB,
    ) -> &'a BVHNode<'a> {
        arena.alloc(BVHNode {
            aabb,
            lhs: lhs,
            rhs: rhs,
        })
    }
}
