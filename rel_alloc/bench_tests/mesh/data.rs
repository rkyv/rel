use ::rand::Rng;

use crate::gen::Generate;

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Generate for Vector3 {
    fn generate<R: Rng>(rand: &mut R) -> Self {
        Self {
            x: rand.gen(),
            y: rand.gen(),
            z: rand.gen(),
        }
    }
}

pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    pub normal: Vector3,
}

impl Generate for Triangle {
    fn generate<R: Rng>(rand: &mut R) -> Self {
        Self {
            v0: Generate::generate(rand),
            v1: Generate::generate(rand),
            v2: Generate::generate(rand),
            normal: Generate::generate(rand),
        }
    }
}

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}
