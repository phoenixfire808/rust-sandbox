//! Readable live HUD. Source-inspired placement, not native font/layout parity.
use super::*;
#[derive(Component, Clone, Copy)]
enum Readout {
    Fps,
    Health,
    Ammo,
}
#[derive(Resource)]
struct FrameMeter {
    seconds: f32,
    frames: u32,
    interval: f32,
    label: String,
}
impl Default for FrameMeter {
    fn default() -> Self {
        Self {
            seconds: 0.,
            frames: 0,
            interval: crate::compiled_spawn_catalog().runtime.fps_interval,
            label: "FPS: measuring...".into(),
        }
    }
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<FrameMeter>()
        .add_systems(Startup, setup)
        .add_systems(Update, update.after(super::update));
}
fn setup(mut commands: Commands) {
    let c = crate::compiled_layout_config();
    for kind in [Readout::Fps, Readout::Health, Readout::Ammo] {
        let fps = matches!(kind, Readout::Fps);
        commands.spawn((
            kind,
            Text::new(""),
            TextFont {
                font_size: if fps { c.font_size } else { c.row_height },
                ..default()
            },
            TextColor(if fps {
                Color::srgb(0.6, 1., 0.6)
            } else {
                Color::srgb(1., 0.8, 0.2)
            }),
            Node {
                position_type: PositionType::Absolute,
                left: if matches!(kind, Readout::Ammo) {
                    Val::Auto
                } else {
                    Val::Px(c.min_margin)
                },
                right: if matches!(kind, Readout::Ammo) {
                    Val::Px(c.min_margin)
                } else {
                    Val::Auto
                },
                top: if fps {
                    Val::Px(c.min_margin)
                } else {
                    Val::Auto
                },
                bottom: if fps {
                    Val::Auto
                } else {
                    Val::Px(c.min_margin)
                },
                padding: UiRect::all(Val::Px(c.panel_padding)),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.65)),
            bevy::ui::FocusPolicy::Pass,
            GlobalZIndex(if fps { 95 } else { 1 }),
        ));
    }
}
fn update(world: &mut World) {
    let dt = world.resource::<Time<Real>>().delta_secs();
    let fps = {
        let mut m = world.resource_mut::<FrameMeter>();
        if dt > 0. {
            m.seconds += dt;
            m.frames += 1;
        }
        if m.seconds >= m.interval && m.frames > 0 {
            m.label = format!(
                "FPS {:.0}  |  {:.1} ms",
                m.frames as f32 / m.seconds,
                m.seconds * 1000. / m.frames as f32
            );
            m.seconds = 0.;
            m.frames = 0;
        }
        m.label.clone()
    };
    let show = world.get_resource::<PlayState>().is_some() && !super::active(world);
    let health = world
        .get_resource::<crate::source_play::npcs::PlayerLife>()
        .map(|p| p.health)
        .unwrap_or(0.);
    let ammo = world
        .get_resource::<PlayState>()
        .and_then(|p| {
            let definition = p
                .spawn_catalog
                .weapons
                .iter()
                .find(|w| w.id == p.active_weapon)?;
            if definition.clip == 0 {
                return None;
            }
            let a = world
                .get_resource::<crate::source_play::weapons::WeaponState>()?
                .ammo
                .get(&p.active_weapon)?;
            Some(format!("AMMO  {}  |  {}", a.loaded, a.reserve))
        })
        .unwrap_or_default();
    for (kind, mut text, mut visibility) in world
        .query::<(&Readout, &mut Text, &mut Visibility)>()
        .iter_mut(world)
    {
        let value = match kind {
            Readout::Fps => fps.clone(),
            Readout::Health => format!("HEALTH  {health:.0}"),
            Readout::Ammo => ammo.clone(),
        };
        *visibility = if matches!(kind, Readout::Fps) || (show && !value.is_empty()) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
