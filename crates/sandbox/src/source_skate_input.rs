//! Read-only XInput polling plus keyboard adapters. No synthetic OS input.
use super::*;
#[derive(Default, Clone, Copy)]
struct Pad {
    connected: bool,
    buttons: u16,
    left: Vec2,
    right: Vec2,
    lt: f32,
    rt: f32,
}
#[cfg(windows)]
fn pad() -> Pad {
    use windows_sys::Win32::UI::Input::XboxController::{XINPUT_STATE, XInputGetState};
    for index in 0..4 {
        // SAFETY: initialized writable XINPUT_STATE lives through the synchronous call;
        // only read the structure when the API reports ERROR_SUCCESS.
        let mut state: XINPUT_STATE = unsafe { std::mem::zeroed() };
        if unsafe { XInputGetState(index, &mut state) } == 0 {
            let g = state.Gamepad;
            let axis = |v: i16| (v as f32 / 32767.).clamp(-1., 1.);
            return Pad {
                connected: true,
                buttons: g.wButtons,
                left: Vec2::new(axis(g.sThumbLX), axis(g.sThumbLY)),
                right: Vec2::new(axis(g.sThumbRX), axis(g.sThumbRY)),
                lt: g.bLeftTrigger as f32 / 255.,
                rt: g.bRightTrigger as f32 / 255.,
            };
        }
    }
    Pad::default()
}
#[cfg(not(windows))]
fn pad() -> Pad {
    Pad::default()
}
const A: u16 = 0x1000;
const B: u16 = 0x2000;
const X: u16 = 0x4000;
const Y: u16 = 0x8000;
const LB: u16 = 0x0100;
const RB: u16 = 0x0200;
const DOWN: u16 = 0x0002;
const UP: u16 = 0x0001;
const LEFT: u16 = 0x0004;
const RIGHT: u16 = 0x0008;
pub(crate) fn use_pressed(world: &World) -> bool {
    let p = pad();
    p.buttons & Y != 0 && world.resource::<Controls>().previous_buttons & Y == 0
}
pub(super) fn update(world: &mut World) {
    let p = pad();
    let keys = world.resource::<ButtonInput<KeyCode>>().clone();
    let occupied = world.resource::<vehicles::Occupancy>().vehicle;
    let board = occupied.filter(|e| world.get::<Board>(*e).is_some());
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let blocked = board.is_none()
        || !focused
        || npcs::dead(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || world.resource::<PlayState>().search_focus
        || tools::devices::viewing(world)
        || world.resource::<vehicles::Occupancy>().weight < 1.;
    let catalog = world.resource::<Catalog>().0.clone();
    let cfg = catalog.board;
    let scheme = profile(
        &SkateCatalog {
            board: cfg.clone(),
            profiles: catalog.profiles,
        },
        &world.resource::<vehicles::Occupancy>().definition,
    )
    .map(|p| p.controls.clone())
    .unwrap_or_default();
    let dt = world.resource::<Time>().delta_secs().min(0.1);
    let down = |k| keys.pressed(k);
    let axis = |negative, positive| down(positive) as i32 as f32 - down(negative) as i32 as f32;
    let dead = |v: f32| if v.abs() > cfg.deadzone { v } else { 0. };
    let held_charge = down(KeyCode::Space)
        || (p.connected
            && if scheme == "direct" {
                p.buttons & X != 0
            } else {
                p.right.y < -cfg.flick_threshold
            });
    let neutral = !held_charge
        && p.left.length() < cfg.deadzone
        && p.right.length() < cfg.deadzone
        && p.lt < cfg.deadzone
        && p.rt < cfg.deadzone
        && p.buttons & (A | B | X | LB | RB | DOWN | UP | LEFT | RIGHT) == 0
        && ![
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyJ,
            KeyCode::KeyL,
            KeyCode::KeyI,
            KeyCode::KeyK,
            KeyCode::KeyG,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::KeyC,
            KeyCode::KeyM,
            KeyCode::KeyN,
            KeyCode::KeyT,
            KeyCode::ControlLeft,
            KeyCode::ShiftLeft,
            KeyCode::ArrowUp,
        ]
        .iter()
        .any(|k| down(*k));
    let mut c = world.resource_mut::<Controls>();
    if blocked || c.owner != board || c.previous_pad != p.connected {
        let goofy = c.goofy;
        *c = Controls {
            goofy,
            owner: board,
            was_blocked: true,
            previous_pad: p.connected,
            previous_buttons: p.buttons,
            ..default()
        };
        return;
    }
    c.previous_buttons = p.buttons;
    c.previous_pad = p.connected;
    if c.was_blocked {
        if neutral {
            c.was_blocked = false;
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        c.goofy = !c.goofy;
    }
    c.steer = (axis(KeyCode::KeyD, KeyCode::KeyA)
        + if scheme == "dual" {
            dead(p.lt - p.rt)
        } else {
            -dead(p.left.x)
        })
    .clamp(-1., 1.);
    c.push = down(KeyCode::KeyW) || p.buttons & A != 0;
    c.brake = down(KeyCode::KeyS) || p.buttons & B != 0;
    c.slide = down(KeyCode::ShiftLeft) || p.buttons & LB != 0;
    c.catch = down(KeyCode::KeyC) || p.buttons & RB != 0;
    c.grind = down(KeyCode::KeyG) || p.buttons & LB != 0;
    c.manual = (axis(KeyCode::KeyN, KeyCode::KeyM)
        + if p.buttons & DOWN != 0 {
            1.
        } else if p.buttons & UP != 0 {
            -1.
        } else {
            0.
        })
    .clamp(-1., 1.);
    let flick_x = if scheme == "dual" {
        p.left.x
    } else {
        p.right.x
    };
    c.flip = (axis(KeyCode::KeyJ, KeyCode::KeyL) + dead(flick_x)).clamp(-1., 1.);
    c.spin = (axis(KeyCode::KeyK, KeyCode::KeyI)
        + if p.buttons & LEFT != 0 {
            1.
        } else if p.buttons & RIGHT != 0 {
            -1.
        } else {
            0.
        })
    .clamp(-1., 1.);
    c.nollie = down(KeyCode::ControlLeft);
    if held_charge && !c.previous_charge {
        c.armed = true;
        c.charge = 0.;
    }
    if held_charge && c.armed {
        c.charge = (c.charge + dt / cfg.charge_seconds).min(1.);
        c.release_elapsed = 0.;
    }
    let direct_pop = scheme == "direct" && c.previous_charge && !held_charge;
    let flick_pop = scheme == "flick" && p.right.y > cfg.flick_threshold;
    let front = p.left.y > cfg.flick_threshold;
    let dual_pop =
        scheme == "dual" && ((front && !c.previous_front) || keys.just_pressed(KeyCode::ArrowUp));
    let keyboard_release = keys.just_released(KeyCode::Space) && scheme != "dual";
    if c.armed && (direct_pop || flick_pop || dual_pop || keyboard_release) {
        c.pop = Some(c.charge);
        c.charge = 0.;
        c.armed = false;
    }
    // A cancelled load must not become a delayed jump several seconds later.
    if !held_charge && c.previous_charge && !(scheme == "dual" || scheme == "flick") {
        c.armed = false;
        c.charge = 0.;
    }
    if !held_charge && c.armed {
        c.release_elapsed += dt;
        if c.release_elapsed > cfg.charge_seconds {
            c.armed = false;
            c.charge = 0.;
        }
    }
    c.previous_charge = held_charge;
    c.previous_front = front;
}
