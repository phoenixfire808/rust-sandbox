//! Bounded Source skeletal sampling. No engine code or asset payloads are embedded.
use crate::source_assets::{source_position, Mounts};
use bevy::prelude::*;
use sandbox_catalog::Result;

fn bytes<const N: usize>(data: &[u8], at: usize) -> Result<[u8; N]> {
    Ok(data
        .get(at..at.checked_add(N).ok_or("model offset overflow")?)
        .ok_or("truncated model")?
        .try_into()?)
}
fn u16_at(d: &[u8], o: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(bytes(d, o)?))
}
fn i32_at(d: &[u8], o: usize) -> Result<i32> {
    Ok(i32::from_le_bytes(bytes(d, o)?))
}
fn index(d: &[u8], o: usize) -> Result<usize> {
    Ok(usize::try_from(i32_at(d, o)?)?)
}
fn f32_at(d: &[u8], o: usize) -> Result<f32> {
    let v = f32::from_le_bytes(bytes(d, o)?);
    if !v.is_finite() {
        return Err("nonfinite model value".into());
    }
    Ok(v)
}
fn vector(d: &[u8], o: usize) -> Result<Vec3> {
    Ok(Vec3::new(
        f32_at(d, o)?,
        f32_at(d, o + 4)?,
        f32_at(d, o + 8)?,
    ))
}
fn text(d: &[u8], o: usize) -> Result<String> {
    let s = d.get(o..).ok_or("string outside model")?;
    let n = s
        .iter()
        .take(4096)
        .position(|b| *b == 0)
        .ok_or("unterminated model string")?;
    Ok(std::str::from_utf8(&s[..n])?.to_string())
}
fn relative(d: &[u8], base: usize, field: usize) -> Result<usize> {
    usize::try_from(base as i64 + i32_at(d, field)? as i64).map_err(Into::into)
}
fn matrix(d: &[u8], o: usize) -> Result<Mat4> {
    let mut rows = [0.; 16];
    for r in 0..3 {
        for c in 0..4 {
            rows[c * 4 + r] = f32_at(d, o + (r * 4 + c) * 4)?;
        }
    }
    rows[15] = 1.;
    Ok(Mat4::from_cols_array(&rows))
}
#[derive(Clone)]
pub struct Bone {
    pub name: String,
    pub parent: i32,
    pub bind: Transform,
    pub inverse: Mat4,
    angles: Vec3,
    pos_scale: Vec3,
    rot_scale: Vec3,
}
#[derive(Clone)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub attachments: Vec<(String, usize, Mat4)>,
}
impl Skeleton {
    pub fn read(d: &[u8]) -> Result<Self> {
        if bytes::<4>(d, 0)? != *b"IDST" || !(44..=49).contains(&i32_at(d, 4)?) {
            return Err("unsupported MDL header".into());
        }
        let count = index(d, 156)?;
        let base = index(d, 160)?;
        if count > 256 {
            return Err("too many bones".into());
        }
        let mut bones = Vec::new();
        for i in 0..count {
            let o = base + i * 216;
            let parent = i32_at(d, o + 4)?;
            if parent >= i as i32 || parent < -1 {
                return Err("invalid bone hierarchy".into());
            }
            let q = Quat::from_xyzw(
                f32_at(d, o + 44)?,
                f32_at(d, o + 48)?,
                f32_at(d, o + 52)?,
                f32_at(d, o + 56)?,
            );
            if q.length_squared() < 0.1 {
                return Err("invalid bind quaternion".into());
            }
            bones.push(Bone {
                name: text(d, relative(d, o, o)?)?,
                parent,
                bind: Transform::from_translation(vector(d, o + 32)?).with_rotation(q.normalize()),
                inverse: matrix(d, o + 96)?,
                angles: vector(d, o + 60)?,
                pos_scale: vector(d, o + 72)?,
                rot_scale: vector(d, o + 84)?,
            });
        }
        let mut attachments = Vec::new();
        let count = index(d, 240)?;
        let base = index(d, 244)?;
        if count > 1024 {
            return Err("too many attachments".into());
        }
        for i in 0..count {
            let o = base + i * 92;
            let bone = index(d, o + 8)?;
            if bone >= bones.len() {
                return Err("invalid attachment bone".into());
            }
            attachments.push((text(d, relative(d, o, o)?)?, bone, matrix(d, o + 12)?));
        }
        Ok(Self { bones, attachments })
    }
    pub fn globals(&self, local: &[Transform]) -> Vec<Mat4> {
        let mut out = vec![Mat4::IDENTITY; self.bones.len()];
        for (i, b) in self.bones.iter().enumerate() {
            let m = local.get(i).unwrap_or(&b.bind).compute_matrix();
            out[i] = if b.parent < 0 {
                m
            } else {
                out[b.parent as usize] * m
            };
        }
        out
    }
    pub fn bind_pose(&self) -> Vec<Transform> {
        self.bones.iter().map(|b| b.bind).collect()
    }
}
#[derive(Clone)]
pub struct Clip {
    pub name: String,
    pub fps: f32,
    pub frames: Vec<Vec<Transform>>,
}
impl Clip {
    pub fn sample(&self, seconds: f32) -> Vec<Transform> {
        let f =
            (seconds * self.fps).rem_euclid((self.frames.len().saturating_sub(1)).max(1) as f32);
        let a = f.floor() as usize;
        let b = (a + 1).min(self.frames.len() - 1);
        let t = f.fract();
        self.frames[a]
            .iter()
            .zip(&self.frames[b])
            .map(|(a, b)| {
                Transform::from_translation(a.translation.lerp(b.translation, t))
                    .with_rotation(a.rotation.slerp(b.rotation, t))
            })
            .collect()
    }
}
// Run values are SIGNED i16. Frame indices must not wrap at 255.
fn rle(d: &[u8], mut o: usize, mut frame: usize) -> Result<f32> {
    for _ in 0..65536 {
        let [valid, total] = bytes::<2>(d, o)?;
        if total == 0 || valid == 0 || valid > total {
            return Err("invalid animation run".into());
        }
        if frame < total as usize {
            let slot = frame.min(valid as usize - 1) + 1;
            return Ok(i16::from_le_bytes(bytes(d, o + slot * 2)?) as f32);
        }
        frame -= total as usize;
        o += (valid as usize + 1) * 2;
    }
    Err("animation run limit".into())
}
fn channels(d: &[u8], o: usize, frame: usize) -> Result<Vec3> {
    let mut v = Vec3::ZERO;
    for i in 0..3 {
        let rel = u16_at(d, o + i * 2)? as usize;
        if rel != 0 {
            v[i] = rle(d, o + rel, frame)?;
        }
    }
    Ok(v)
}
fn half(v: u16) -> f32 {
    let sign = if v & 0x8000 == 0 { 1. } else { -1. };
    let exp = (v >> 10) & 31;
    let mant = (v & 1023) as f32;
    sign * if exp == 0 {
        mant * 2f32.powi(-24)
    } else if exp == 31 {
        f32::INFINITY
    } else {
        (1. + mant / 1024.) * 2f32.powi(exp as i32 - 15)
    }
}
fn quat(d: &[u8], o: usize, wide: bool) -> Result<Quat> {
    let (x, y, z, negative) = if wide {
        let n = u64::from_le_bytes(bytes(d, o)?);
        let v = |shift: u32| (((n >> shift) & 0x1fffffu64) as f32 - 1048576.) / 1048576.5;
        (v(0), v(21), v(42), n >> 63 != 0)
    } else {
        let z = u16_at(d, o + 4)?;
        (
            (u16_at(d, o)? as f32 - 32768.) / 32768.,
            (u16_at(d, o + 2)? as f32 - 32768.) / 32768.,
            ((z & 32767) as f32 - 16384.) / 16384.,
            z & 32768 != 0,
        )
    };
    let w = (1. - x * x - y * y - z * z).max(0.).sqrt() * if negative { -1. } else { 1. };
    Ok(Quat::from_xyzw(x, y, z, w).normalize())
}
fn frame_pose(d: &[u8], mut o: usize, frame: usize, skeleton: &Skeleton) -> Result<Vec<Transform>> {
    let mut pose = skeleton.bind_pose();
    for _ in 0..=skeleton.bones.len() {
        let [bone, flags] = bytes::<2>(d, o)?;
        if bone == 255 {
            break;
        }
        let b = skeleton
            .bones
            .get(bone as usize)
            .ok_or("animation bone outside skeleton")?;
        if flags & 0x10 != 0 {
            return Err("delta clip needs sequence-layer evaluation".into());
        }
        let mut p = o + 4;
        let mut t = b.bind;
        if flags & 0x20 != 0 {
            t.rotation = quat(d, p, true)?;
            p += 8;
        }
        if flags & 2 != 0 {
            t.rotation = quat(d, p, false)?;
            p += 6;
        }
        if flags & 1 != 0 {
            t.translation = Vec3::new(
                half(u16_at(d, p)?),
                half(u16_at(d, p + 2)?),
                half(u16_at(d, p + 4)?),
            );
            p += 6;
        }
        if flags & 8 != 0 {
            let a = b.angles + channels(d, p, frame)? * b.rot_scale;
            t.rotation = Quat::from_euler(EulerRot::ZYX, a.z, a.y, a.x);
            p += 6;
        }
        if flags & 4 != 0 {
            t.translation = b.bind.translation + channels(d, p, frame)? * b.pos_scale;
        }
        if !t.translation.is_finite() {
            return Err("invalid animation translation".into());
        }
        pose[bone as usize] = t;
        let next = u16_at(d, o + 2)? as usize;
        if next == 0 {
            break;
        }
        if next < 4 {
            return Err("invalid animation link".into());
        }
        o += next;
    }
    Ok(pose)
}
/// Read a selected authored clip, including sectioned data and external .ani blocks.
pub fn load_clip(
    mounts: &Mounts,
    bsp: &vbsp::Bsp,
    path: &str,
    wanted: &str,
    target: &Skeleton,
) -> Result<Clip> {
    let d = mounts.read(bsp, path)?;
    let source = Skeleton::read(&d)?;
    let count = index(&d, 180)?;
    let base = index(&d, 184)?;
    if count > 16384 {
        return Err("too many animations".into());
    }
    let mut found = None;
    for i in 0..count {
        let o = base + i * 100;
        if text(&d, relative(&d, o, o + 4)?)? == wanted {
            found = Some(o);
            break;
        }
    }
    let o = found.ok_or_else(|| format!("animation {wanted} not in {path}"))?;
    let frames = index(&d, o + 16)?;
    let fps = f32_at(&d, o + 8)?;
    if frames == 0 || frames > 10000 || fps <= 0. {
        return Err("invalid clip dimensions".into());
    }
    let section_frames = index(&d, o + 84)?;
    let section_offset = index(&d, o + 80)?;
    let mut external = None;
    let mut output = Vec::new();
    for frame in 0..frames {
        let (block, offset, local_frame) = if section_frames > 0 {
            let section = if frame == frames - 1 && frames > section_frames {
                frames / section_frames + 1
            } else {
                frame / section_frames
            };
            let p = o + section_offset + section * 8;
            (
                index(&d, p)?,
                index(&d, p + 4)?,
                if frame == frames - 1 && frames > section_frames {
                    0
                } else {
                    frame % section_frames
                },
            )
        } else {
            (index(&d, o + 52)?, index(&d, o + 56)?, frame)
        };
        let local = if offset == 0 && block == 0 {
            source.bind_pose()
        } else if block == 0 {
            frame_pose(&d, o + offset, local_frame, &source)?
        } else {
            if external.is_none() {
                let name = text(&d, index(&d, 348)?)?;
                external = Some(mounts.read(bsp, &name)?);
            }
            if block >= index(&d, 352)? {
                return Err("invalid animation block".into());
            }
            let start = index(&d, index(&d, 356)? + block * 8)?;
            frame_pose(
                external.as_ref().unwrap(),
                start + offset,
                local_frame,
                &source,
            )?
        };
        let mut mapped = target.bind_pose();
        for (i, b) in target.bones.iter().enumerate() {
            if let Some(j) = source.bones.iter().position(|s| s.name == b.name) {
                mapped[i] = local[j];
            }
        }
        output.push(mapped);
    }
    Ok(Clip {
        name: wanted.into(),
        fps,
        frames: output,
    })
}
pub fn bevy_matrix(source: Mat4, scale: f32) -> Mat4 {
    let basis = Mat4::from_cols(
        Vec3::NEG_Z.extend(0.),
        Vec3::NEG_X.extend(0.),
        Vec3::Y.extend(0.),
        Vec4::W,
    );
    let mut m = basis * source * basis.inverse();
    m.w_axis = source_position(source.w_axis.truncate().to_array(), scale).extend(1.);
    m
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_runs_repeat_and_cross_255() {
        let mut d = vec![1, 255];
        d.extend_from_slice(&(-123i16).to_le_bytes());
        d.extend_from_slice(&[1, 10]);
        d.extend_from_slice(&456i16.to_le_bytes());
        assert_eq!(rle(&d, 0, 0).unwrap(), -123.);
        assert_eq!(rle(&d, 0, 254).unwrap(), -123.);
        assert_eq!(rle(&d, 0, 256).unwrap(), 456.);
        assert!(rle(&[0, 0], 0, 0).is_err());
    }
    #[test]
    fn matrix_translation_and_rotation_share_basis() {
        let m = Mat4::from_rotation_translation(Quat::from_rotation_z(0.7), Vec3::new(3., 4., 5.));
        let p = Vec3::new(6., 7., 8.);
        assert!(bevy_matrix(m, 0.02)
            .transform_point3(source_position(p.to_array(), 0.02))
            .abs_diff_eq(
                source_position(m.transform_point3(p).to_array(), 0.02),
                1e-5
            ));
    }
}
