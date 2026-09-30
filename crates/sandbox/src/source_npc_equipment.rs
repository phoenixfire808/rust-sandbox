//! Bounded magazine/reload mechanics for the currently supported Combine ranged subset.
use super::*;
use sandbox_catalog::spawn::NpcEquipment;

#[derive(Component)]
pub struct Magazine {
    definition: NpcEquipment,
    rounds: u16,
    shots_in_burst: u8,
    reload_remaining: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum FireResult {
    Fired { next_attack: f32 },
    Reloading { next_attack: f32 },
}

pub fn attach(world: &mut World, entity: Entity, id: &str) -> Result<()> {
    let equipment = world
        .resource::<PlayState>()
        .spawn_catalog
        .npc_equipment
        .iter()
        .find(|equipment| equipment.id == id)
        .cloned()
        .ok_or("ranged NPC has no authored equipment definition")?;
    world.entity_mut(entity).insert(Magazine {
        rounds: equipment.magazine,
        definition: equipment,
        shots_in_burst: 0,
        reload_remaining: 0.,
    });
    Ok(())
}

/// Advance reload timers independently of AI thinking so re-enabling thinking does not
/// freeze weapon maintenance. NPCs still do not fire while the AI is paused.
pub fn tick(world: &mut World, dt: f32) {
    let mut query = world.query::<&mut Magazine>();
    for mut magazine in query.iter_mut(world) {
        if magazine.reload_remaining > 0. {
            magazine.reload_remaining = (magazine.reload_remaining - dt).max(0.);
            if magazine.reload_remaining == 0. {
                magazine.rounds = magazine.definition.magazine;
                magazine.shots_in_burst = 0;
            }
        }
    }
}

/// Consume one round, starting a reload only when the next trigger finds an empty magazine.
/// Each call produces at most one damage event; subsequent burst shots are scheduled by the
/// existing per-actor attack cooldown and re-check the current target/line of sight.
pub fn fire(world: &mut World, entity: Entity, interval: f32) -> FireResult {
    let Some(mut magazine) = world.get_mut::<Magazine>(entity) else {
        return FireResult::Reloading {
            next_attack: interval,
        };
    };
    if magazine.reload_remaining > 0. {
        return FireResult::Reloading {
            next_attack: interval,
        };
    }
    if magazine.rounds == 0 {
        magazine.reload_remaining = magazine.definition.reload_seconds;
        magazine.shots_in_burst = 0;
        return FireResult::Reloading {
            next_attack: interval,
        };
    }
    magazine.rounds -= 1;
    if magazine.shots_in_burst == 0 {
        magazine.shots_in_burst = magazine.definition.burst;
    }
    magazine.shots_in_burst -= 1;
    let next_attack = if magazine.shots_in_burst == 0 {
        interval
    } else {
        magazine.definition.burst_spacing
    };
    drop(magazine);
    let point = world
        .get::<Transform>(entity)
        .map(|transform| transform.translation);
    super::audio::emit(world, "npc.ar2.fire", point, 1.);
    FireResult::Fired { next_attack }
}
