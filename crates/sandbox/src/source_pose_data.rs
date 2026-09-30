//! Bounded MDL pose metadata, independently read from documented studio layouts.
use super::*;
use std::collections::BTreeMap;
#[derive(Clone)]
pub struct Delta {
    pub position: Vec3,
    pub normal: Vec3,
    pub side: f32,
}
#[derive(Clone)]
pub struct Flex {
    pub descriptor: usize,
    pub pair: Option<usize>,
    pub targets: [f32; 4],
    pub vertices: BTreeMap<usize, Delta>,
}
#[derive(Clone)]
pub struct Eye {
    pub bone: usize,
    pub origin: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    pub iris_scale: f32,
}
#[derive(Clone, Default)]
pub struct Part {
    pub flexes: Vec<Flex>,
    pub eye: Option<Eye>,
}
#[derive(Clone)]
pub struct Data {
    pub skeleton: Skeleton,
    pub physics_bones: Vec<i32>,
    pub names: Vec<String>,
    pub parts: Vec<Part>,
}
fn count(d: &[u8], o: usize, cap: usize) -> Result<usize> {
    let n = index(d, o)?;
    if n > cap {
        return Err("pose metadata count exceeds limit".into());
    }
    Ok(n)
}
impl Data {
    pub fn read(d: &[u8]) -> Result<Self> {
        let skeleton = Skeleton::read(d)?;
        let bone_base = index(d, 160)?;
        let physics_bones = (0..skeleton.bones.len())
            .map(|i| i32_at(d, bone_base + i * 216 + 172))
            .collect::<Result<Vec<_>>>()?;
        let desc_count = count(d, 260, 1024)?;
        let desc_base = index(d, 264)?;
        let names = (0..desc_count)
            .map(|i| {
                let o = desc_base + i * 4;
                text(d, relative(d, o, o)?)
            })
            .collect::<Result<Vec<_>>>()?;
        let fixed = i32_at(d, 152)? & 0x00200000 != 0;
        let fixed_scale = if fixed {
            let v = f32_at(d, 392)?;
            if v <= 0. || v > 1. {
                return Err("invalid flex fixed-point scale".into());
            }
            v
        } else {
            1.
        };
        let body_count = count(d, 232, 128)?;
        let body_base = index(d, 236)?;
        let mut parts = Vec::new();
        let mut total_deltas = 0usize;
        for body in 0..body_count {
            let b = body_base + body * 16;
            if count(d, b + 4, 1024)? == 0 {
                continue;
            }
            // Match the existing model importer: bodygroup variant zero only.
            let m = relative(d, b, b + 12)?;
            let mesh_count = count(d, m + 72, 4096)?;
            if parts.len() + mesh_count > 4096 {
                return Err("pose mesh budget exceeded".into());
            }
            let mesh_base = relative(d, m, m + 76)?;
            let vertex_base = index(d, m + 84)?;
            if vertex_base % 48 != 0 {
                return Err("unaligned model vertex base".into());
            }
            let vertex_base = vertex_base / 48;
            let eye_count = count(d, m + 100, 64)?;
            let eye_base = relative(d, m, m + 104)?;
            let mut eyes = Vec::new();
            for i in 0..eye_count {
                let e = eye_base + i * 172;
                let bone = index(d, e + 4)?;
                if bone >= skeleton.bones.len() {
                    return Err("eye bone outside skeleton".into());
                }
                let up = vector(d, e + 28)?;
                let forward = vector(d, e + 40)?;
                let iris_scale = f32_at(d, e + 60)?;
                if up.length_squared() < 0.1
                    || forward.length_squared() < 0.1
                    || up.cross(forward).length_squared() < 0.01
                    || iris_scale <= 0.
                    || iris_scale > 100.
                {
                    return Err("invalid eye projection frame".into());
                }
                eyes.push(Eye {
                    bone,
                    origin: vector(d, e + 8)?,
                    up: up.normalize(),
                    forward: forward.normalize(),
                    iris_scale,
                });
            }
            for mesh in 0..mesh_count {
                let o = mesh_base + mesh * 116;
                let mesh_vertices = count(d, o + 8, 1_000_000)?;
                let base = vertex_base
                    .checked_add(index(d, o + 12)?)
                    .ok_or("vertex index overflow")?;
                let mut part = Part::default();
                if i32_at(d, o + 24)? == 1 {
                    part.eye = Some(
                        eyes.get(index(d, o + 28)?)
                            .ok_or("invalid eye mesh index")?
                            .clone(),
                    );
                }
                let flex_count = count(d, o + 16, 4096)?;
                let flex_base = relative(d, o, o + 20)?;
                for f in 0..flex_count {
                    let a = flex_base + f * 60;
                    let descriptor = index(d, a)?;
                    let pair = index(d, a + 28)?;
                    if descriptor >= names.len() || pair >= names.len().max(1) {
                        return Err("invalid flex descriptor".into());
                    }
                    let targets = [
                        f32_at(d, a + 4)?,
                        f32_at(d, a + 8)?,
                        f32_at(d, a + 12)?,
                        f32_at(d, a + 16)?,
                    ];
                    if targets.windows(2).any(|p| p[0] > p[1]) {
                        return Err("invalid flex target ramp".into());
                    }
                    let n = count(d, a + 20, 65536)?;
                    total_deltas += n;
                    if total_deltas > 2_000_000 {
                        return Err("pose delta budget exceeded".into());
                    }
                    let start = relative(d, a, a + 24)?;
                    let ty = bytes::<1>(d, a + 32)?[0];
                    let stride = match ty {
                        0 => 16,
                        1 => 18,
                        _ => return Err("unsupported vertex animation type".into()),
                    };
                    let mut vertices = BTreeMap::new();
                    for j in 0..n {
                        let v = start + j * stride;
                        let local = u16_at(d, v)? as usize;
                        if local >= mesh_vertices {
                            return Err("flex vertex outside mesh".into());
                        }
                        let scalar = |p| -> Result<f32> {
                            let bits = u16_at(d, p)?;
                            let value = if fixed {
                                (bits as i16) as f32 * fixed_scale
                            } else {
                                half(bits)
                            };
                            if !value.is_finite() || value.abs() > 100000. {
                                return Err("invalid flex delta".into());
                            }
                            Ok(value)
                        };
                        let vec = |p| -> Result<Vec3> {
                            Ok(Vec3::new(scalar(p)?, scalar(p + 2)?, scalar(p + 4)?))
                        };
                        vertices.insert(
                            base + local,
                            Delta {
                                position: source_position(vec(v + 4)?.to_array(), 0.01905),
                                normal: source_position(vec(v + 10)?.to_array(), 1.),
                                side: bytes::<1>(d, v + 3)?[0] as f32 / 255.,
                            },
                        );
                    }
                    part.flexes.push(Flex {
                        descriptor,
                        pair: (pair != 0).then_some(pair),
                        targets,
                        vertices,
                    });
                }
                parts.push(part);
            }
        }
        Ok(Self {
            skeleton,
            physics_bones,
            names,
            parts,
        })
    }
}
impl Flex {
    pub fn weight(&self, value: f32) -> f32 {
        let [a, b, c, d] = self.targets;
        if value <= a || value >= d {
            0.
        } else if value < b {
            (value - a) / (b - a).max(f32::EPSILON)
        } else if value > c {
            (d - value) / (d - c).max(f32::EPSILON)
        } else {
            1.
        }
    }
}
