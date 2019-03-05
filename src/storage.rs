use crate::{
    collision::{BVHNode, Cuboid, Hitable, HitableList, MovingSphere, Rect, Sphere},
    material::Material,
    perlin::Perlin,
    texture::{RgbImage, Texture},
};
use rand_xoshiro::Xoshiro256Plus;
use typed_arena::Arena;

pub struct Storage<'a> {
    pub texture_arena: Arena<Texture<'a>>,
    pub material_arena: Arena<Material<'a>>,
    pub image_arena: Arena<RgbImage>,
    pub sphere_arena: Arena<Sphere>,
    pub moving_sphere_arena: Arena<MovingSphere>,
    pub rect_arena: Arena<Rect>,
    pub bvhnode_arena: Arena<BVHNode<'a>>,
    pub hitables_arena: Arena<HitableList<'a>>,
    pub cuboid_arena: Arena<Cuboid>,
    pub perlin_noise: Perlin,
}

impl<'a> Storage<'a> {
    pub fn new(rng: &mut Xoshiro256Plus) -> Storage<'a> {
        Storage {
            texture_arena: Arena::new(),
            material_arena: Arena::new(),
            image_arena: Arena::new(),
            moving_sphere_arena: Arena::new(),
            sphere_arena: Arena::new(),
            rect_arena: Arena::new(),
            bvhnode_arena: Arena::new(),
            hitables_arena: Arena::new(),
            cuboid_arena: Arena::new(),
            perlin_noise: Perlin::new(rng),
        }
    }

    #[inline]
    pub fn alloc_texture(&self, texture: Texture<'a>) -> &mut Texture<'a> {
        self.texture_arena.alloc(texture)
    }

    #[inline]
    pub fn alloc_material(&self, material: Material<'a>) -> &mut Material<'a> {
        self.material_arena.alloc(material)
    }

    #[inline]
    pub fn alloc_image(&self, rgb_image: RgbImage) -> &mut RgbImage {
        self.image_arena.alloc(rgb_image)
    }

    #[inline]
    pub fn alloc_sphere(&self, sphere: Sphere) -> &mut Sphere {
        self.sphere_arena.alloc(sphere)
    }

    #[inline]
    pub fn alloc_moving_sphere(&self, sphere: MovingSphere) -> &mut MovingSphere {
        self.moving_sphere_arena.alloc(sphere)
    }

    #[inline]
    pub fn alloc_rect(&self, rect: Rect) -> &mut Rect {
        self.rect_arena.alloc(rect)
    }

    #[inline]
    pub fn alloc_cuboid(&self, cuboid: Cuboid) -> &mut Cuboid {
        self.cuboid_arena.alloc(cuboid)
    }

    #[inline]
    pub fn alloc_hitables(&self, hitables: Vec<Hitable<'a>>) -> &mut HitableList<'a> {
        self.hitables_arena.alloc(HitableList::new(hitables))
    }
}
