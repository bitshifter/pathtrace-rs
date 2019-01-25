// TODO: remove
#![allow(dead_code)]
use crate::collision::{Hitable, Ray, RayHitEx, Sphere, AABB};
use crate::material::Material;
use rand::{Rng, XorShiftRng};

pub struct BVHNode {
    aabb: AABB,
    lhs: Box<dyn Hitable + Sync + Send>,
    rhs: Box<dyn Hitable + Sync + Send>,
}

impl Hitable for BVHNode {
    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHitEx> {
        if self.aabb.ray_hit(ray, t_min, t_max) {
            let hit_lhs = self.lhs.ray_hit(ray, t_min, t_max);
            let hit_rhs = self.rhs.ray_hit(ray, t_min, t_max);
            match (hit_lhs, hit_rhs) {
                (Some(hit_lhs), Some(hit_rhs)) => {
                    if hit_lhs.t < hit_rhs.t {
                        Some(hit_lhs)
                    } else {
                        Some(hit_rhs)
                    }
                }
                (Some(hit_lhs), None) => Some(hit_lhs),
                (None, Some(hit_rhs)) => Some(hit_rhs),
                (None, None) => {
                    // TODO: panic?
                    None
                }
            }
        } else {
            None
        }
    }
    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.aabb)
    }
}

impl BVHNode {
    pub fn new(rng: &mut XorShiftRng, hitables: &mut [(Sphere, Material)]) -> Option<BVHNode> {
        BVHNode::new_split(rng, hitables, 0.0, 0.0)
    }
    pub fn new_split(rng: &mut XorShiftRng, hitables: &mut [(Sphere, Material)], t0: f32, t1: f32) -> Option<BVHNode> {
        let axis = rng.next_u32() % 3;
        hitables.sort_unstable_by(|lhs, rhs| {
            let lhs_min = lhs.bounding_box(t0, t1).unwrap().min;
            let rhs_min = rhs.bounding_box(t0, t1).unwrap().min;
            let ord = match axis {
                0 => lhs_min.get_x().partial_cmp(&rhs_min.get_x()),
                1 => lhs_min.get_y().partial_cmp(&rhs_min.get_y()),
                2 => lhs_min.get_z().partial_cmp(&rhs_min.get_z()),
                _ => panic!("got invalid axis {}", axis),
            };
            ord.unwrap()
        });
        match hitables.len() {
            0 => None,
            1 => {
                let lhs = Box::new(hitables[0]);
                let rhs = lhs.clone();
                let aabb = lhs.bounding_box(t0, t1).unwrap();
                Some(BVHNode { lhs, rhs, aabb })
            }
            2 => {
                let lhs = Box::new(hitables[0]);
                let rhs = Box::new(hitables[1]);
                let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
                let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
                let aabb = lhs_aabb.combine(&rhs_aabb);
                Some(BVHNode { lhs, rhs, aabb })
            }
            _ => {
                let pivot = hitables.len() / 2;
                let lhs = Box::new(BVHNode::new_split(rng, &mut hitables[0..pivot], t0, t1).unwrap());
                let rhs = Box::new(BVHNode::new_split(rng, &mut hitables[pivot + 1..], t0, t1).unwrap());
                let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
                let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
                let aabb = lhs_aabb.combine(&rhs_aabb);
                Some(BVHNode { lhs, rhs, aabb })
            }
        }
    }
}

// #[derive(Copy, Clone)]
// enum NodeType {
//     Sphere(u32),
//     BVHNode,
// }

// pub struct BVH {
//     aabbs: Vec<AABB>,
//     nodes: Vec<NodeType>,
//     spheres: Vec<Sphere>,
//     depth: u32,
// }

// impl BVH {
//     fn add_node(
//         &mut self,
//         rng: &mut XorShiftRng,
//         indices: &mut [u32],
//         spheres: &[Sphere],
//         aabbs: &[AABB],
//     ) -> AABB {
//         self.depth += 1;
//         let axis = rng.next_u32() % 3;
//         indices.sort_unstable_by(|lhs, rhs| {
//             let lhs_min = &aabbs[*lhs as usize].min;
//             let rhs_min = &aabbs[*rhs as usize].min;
//             let ord = match axis {
//                 0 => lhs_min.get_x().partial_cmp(&rhs_min.get_x()),
//                 1 => lhs_min.get_y().partial_cmp(&rhs_min.get_y()),
//                 2 => lhs_min.get_z().partial_cmp(&rhs_min.get_z()),
//                 _ => panic!("got invalid axis {}", axis),
//             };
//             ord.unwrap()
//         });
//         let (lhs_node, lhs_aabb, rhs_node, rhs_aabb) = match indices.len() {
//             1 => {
//                 let index = indices[0] as usize;
//                 let sphere = &spheres[index];
//                 let aabb = aabbs[index];
//                 self.spheres.push(*sphere);
//                 let node = NodeType::Sphere(self.spheres.len() as u32);
//                 (node, aabb, node, aabb)
//             }
//             2 => {
//                 let lhs_index = indices[0] as usize;
//                 let rhs_index = indices[1] as usize;
//                 let lhs_sphere = &spheres[lhs_index];
//                 let rhs_sphere = &spheres[rhs_index];
//                 self.spheres.push(*lhs_sphere);
//                 let lhs_node = NodeType::Sphere(self.spheres.len() as u32);
//                 self.spheres.push(*rhs_sphere);
//                 let rhs_node = NodeType::Sphere(self.spheres.len() as u32);
//                 (lhs_node, aabbs[lhs_index], rhs_node, aabbs[rhs_index])
//             }
//             _ => {
//                 let n = indices.len() / 2;
//                 (
//                     NodeType::BVHNode,
//                     self.add_node(rng, &mut indices[0..n], spheres, aabbs),
//                     NodeType::BVHNode,
//                     self.add_node(rng, &mut indices[n..], spheres, aabbs),
//                 )
//             }
//         };
//         let aabb = lhs_aabb.combine(&rhs_aabb);
//         self.nodes.push(NodeType::BVHNode);
//         self.nodes.push(lhs_node);
//         self.nodes.push(rhs_node);
//         self.aabbs.push(aabb);
//         self.aabbs.push(lhs_aabb);
//         self.aabbs.push(rhs_aabb);
//         aabb
//     }

//     pub fn new(rng: &mut XorShiftRng, spheres: &[Sphere]) -> BVH {
//         let num_spheres = spheres.len();
//         let mut sphere_aabbs = Vec::with_capacity(num_spheres);
//         let mut indices = Vec::with_capacity(num_spheres);
//         for (index, sphere) in spheres.iter().enumerate() {
//             indices.push(index as u32);
//             sphere_aabbs.push(sphere.bounding_box(0.0, 0.0).unwrap());
//         }
//         let mut bvh = BVH {
//             aabbs: Vec::with_capacity(num_spheres),
//             nodes: Vec::with_capacity(num_spheres),
//             spheres: Vec::with_capacity(num_spheres),
//             depth: 0,
//         };
//         if num_spheres > 0 {
//             bvh.add_node(rng, &mut indices[..], &spheres, &sphere_aabbs);
//         }
//         bvh
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::collision::Sphere;
    use glam::vec3;
    use rand::{SeedableRng, XorShiftRng};
    use std::{f32, iter};

    const MAX_T: f32 = f32::MAX;
    const MIN_T: f32 = 0.001;
    const FIXED_SEED: [u32; 4] = [0x193a_6754, 0xa8a7_d469, 0x9783_0e05, 0x113b_a7bb];

    fn rand_spheres(rng: &mut Rng, n: usize) -> Vec<Sphere> {
        iter::repeat_with(|| {
            Sphere::new(
                vec3(
                    -50.0 + 100.0 * rng.next_f32(),
                    -50.0 + 100.0 * rng.next_f32(),
                    -50.0 + 100.0 * rng.next_f32(),
                ),
                1.0 + 10.0 * rng.next_f32(),
            )
        })
        .take(n)
        .collect()
    }

    #[test]
    fn test_empty() {
        let mut rng = XorShiftRng::from_seed(FIXED_SEED);
        let bvh = BVHNode::new(&mut rng, &mut vec![]);
        assert!(bvh.is_none());
    }

    #[test]
    fn test_one() {
        let mut rng = XorShiftRng::from_seed(FIXED_SEED);
        let mut spheres = rand_spheres(&mut rng, 1);
        let bvh = BVHNode::new(&mut rng, &mut spheres);
        assert!(bvh.is_some());
        let bvh = bvh.unwrap();
        assert_eq!(
            spheres[0].bounding_box(0.0, 0.0),
            bvh.bounding_box(0.0, 0.0)
        );
        let ray = Ray {
            origin: spheres[0].centre,
            direction: vec3(1.0, 0.0, 0.0),
        };
        let hit = bvh.hit(&ray, MIN_T, MAX_T);
        assert!(hit.is_some());
    }

    #[test]
    fn test_two() {
        let mut rng = XorShiftRng::from_seed(FIXED_SEED);
        let mut spheres = rand_spheres(&mut rng, 2);
        let bvh = BVHNode::new(&mut rng, &mut spheres);
        assert!(bvh.is_some());
        let bvh = bvh.unwrap();
        // assert_eq!(spheres[0].bounding_box(0.0, 0.0), bvh.bounding_box(0.0, 0.0));
        let ray = Ray {
            origin: spheres[0].centre,
            direction: vec3(1.0, 0.0, 0.0),
        };
        let hit = bvh.hit(&ray, MIN_T, MAX_T);
        assert!(hit.is_some());
    }
    // #[test]
    // fn test_empty() {
    //     let mut rng = XorShiftRng::from_seed(FIXED_SEED);
    //     let bvh = BVH::new(&mut rng, &vec![]);
    //     assert_eq!(0, bvh.depth);
    //     assert_eq!(0, bvh.nodes.len());
    // }

    // #[test]
    // fn test_one() {
    //     let mut rng = XorShiftRng::from_seed(FIXED_SEED);
    //     let bvh = BVH::new(&mut rng, &[Sphere::new(vec3(0.0, 0.0, 0.0), 1.0)]);
    //     assert_eq!(1, bvh.depth);
    //     assert_eq!(3, bvh.nodes.len());
    // }

    // #[test]
    // fn test_two() {
    //     let mut rng = XorShiftRng::from_seed(FIXED_SEED);
    //     let spheres = [
    //         Sphere::new(vec3(-5.0, 0.0, 0.0), 1.0),
    //         Sphere::new(vec3(5.0, 0.0, 0.0), 1.0),
    //     ];
    //     let bvh = BVH::new(&mut rng, &spheres);
    //     assert_eq!(1, bvh.depth);
    //     assert_eq!(3, bvh.nodes.len());
    // }

    // #[test]
    // fn test_three() {
    //     let mut rng = XorShiftRng::from_seed(FIXED_SEED);
    //     let spheres = rand_spheres(&mut rng, 3);
    //     let bvh = BVH::new(&mut rng, &spheres);
    //     assert_eq!(3, bvh.depth);
    //     assert_eq!(7, bvh.nodes.len());
    // }
}
