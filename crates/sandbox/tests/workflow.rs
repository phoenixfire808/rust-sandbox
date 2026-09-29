use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rust_sandbox::*;

#[test]
fn generated_behavior_registry_matches_research_sheets() {
    assert_eq!(
        compiled_behaviors(),
        sandbox_catalog::behavior::load(&project_root().join("sheets")).unwrap()
    );
}

#[test]
fn generated_dimensions_and_mass_reach_real_physics_components() {
    let mut app = headless_app();
    let defs = app.world().resource::<Catalog>().0.props.clone();
    let w = app.world_mut();
    for (prop, collider, mass) in w
        .query::<(&Prop, &Collider, &ColliderMassProperties)>()
        .iter(w)
    {
        let p = &defs[prop.definition];
        assert_eq!(
            collider.as_cuboid().unwrap().half_extents(),
            Vec3::new(p.size_x, p.size_y, p.size_z) / 2.
        );
        assert!(matches!(mass,ColliderMassProperties::Mass(v) if *v==p.mass));
    }
}

#[test]
fn held_body_tracks_target_and_freezes_across_physics_ticks() {
    let mut app = headless_app();
    let e = {
        let w = app.world_mut();
        w.query_filtered::<Entity, With<Prop>>()
            .iter(w)
            .next()
            .unwrap()
    };
    {
        let mut aim = app.world_mut().resource_mut::<Aim>();
        aim.origin = Vec3::new(8., 6., 0.);
        aim.direction = Vec3::NEG_Z;
        aim.distance = 3.;
    }
    apply_action(app.world_mut(), Action::Grab(e)).unwrap();
    for _ in 0..180 {
        app.update();
    }
    let position = app.world().get::<Transform>(e).unwrap().translation;
    assert!(
        position.distance(Vec3::new(8., 6., -3.)) < 0.3,
        "grab position: {position}"
    );
    apply_action(app.world_mut(), Action::Freeze(e, true)).unwrap();
    for _ in 0..120 {
        app.update();
    }
    assert!(
        app.world()
            .get::<Transform>(e)
            .unwrap()
            .translation
            .distance(position)
            < 0.001
    );
    assert!(app.world().resource::<Session>().held.is_none());
}

#[test]
fn disk_save_roundtrip_and_bad_load_are_transactional() {
    let mut app = headless_app();
    let before = snapshot(app.world_mut());
    let content = app.world().resource::<Catalog>().0.clone();
    let dir = std::env::temp_dir().join(format!(
        "rust-sandbox-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = before.save_new(&content, &dir).unwrap();
    let second = before.save_new(&content, &dir).unwrap();
    assert_ne!(path, second);
    apply_action(app.world_mut(), Action::Spawn(0, Vec3::new(8., 2., 0.))).unwrap();
    apply_action(app.world_mut(), Action::Load(path)).unwrap();
    assert_eq!(snapshot(app.world_mut()), before);
    let bad = dir.join("malformed.json");
    std::fs::write(&bad, b"{broken").unwrap();
    assert!(apply_action(app.world_mut(), Action::Load(bad)).is_err());
    assert_eq!(snapshot(app.world_mut()), before);
    std::fs::remove_dir_all(dir).unwrap();
}
