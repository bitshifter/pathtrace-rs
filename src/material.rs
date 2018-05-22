use collision::{ray, Ray, RayHit};
use math::{random_in_unit_sphere, random_unit_vector, reflect, refract, schlick};
use rand::{Rng, XorShiftRng};
use vmath::{dot, normalize, vec3, Length, Vec3};

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertian { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { ref_idx: f32 },
}

impl Material {
    fn scatter_lambertian(
        albedo: Vec3,
        _: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray)> {
        let target = ray_hit.point + ray_hit.normal + random_unit_vector(rng);
        Some((
            albedo,
            ray(ray_hit.point, normalize(target - ray_hit.point)),
        ))
    }
    fn scatter_metal(
        albedo: Vec3,
        fuzz: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray)> {
        let reflected = reflect(ray_in.direction, ray_hit.normal);
        if dot(reflected, ray_hit.normal) > 0.0 {
            Some((
                albedo,
                ray(
                    ray_hit.point,
                    normalize(reflected + fuzz * random_in_unit_sphere(rng)),
                ),
            ))
        } else {
            None
        }
    }
    fn scatter_dielectric(
        ref_idx: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray)> {
        let attenuation = vec3(1.0, 1.0, 1.0);
        let rdotn = dot(ray_in.direction, ray_hit.normal);
        let (outward_normal, ni_over_nt, cosine) = if rdotn > 0.0 {
            let cosine = rdotn / ray_in.direction.length();
            let cosine = (1.0 - ref_idx * ref_idx * (1.0 - cosine * cosine)).sqrt();
            (-ray_hit.normal, ref_idx, cosine)
        } else {
            let cosine = -rdotn / ray_in.direction.length();
            (ray_hit.normal, 1.0 / ref_idx, cosine)
        };
        if let Some(refracted) = refract(ray_in.direction, outward_normal, ni_over_nt) {
            let reflect_prob = schlick(cosine, ref_idx);
            if rng.next_f32() > reflect_prob {
                return Some((attenuation, ray(ray_hit.point, normalize(refracted))));
            }
        }
        Some((
            attenuation,
            ray(
                ray_hit.point,
                normalize(reflect(ray_in.direction, ray_hit.normal)),
            ),
        ))
    }
    pub fn scatter(
        &self,
        ray: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray)> {
        match *self {
            Material::Lambertian { albedo } => {
                Material::scatter_lambertian(albedo, ray, ray_hit, rng)
            }
            Material::Metal { albedo, fuzz } => {
                Material::scatter_metal(albedo, fuzz, ray, ray_hit, rng)
            }
            Material::Dielectric { ref_idx } => {
                Material::scatter_dielectric(ref_idx, ray, ray_hit, rng)
            }
        }
    }
}
