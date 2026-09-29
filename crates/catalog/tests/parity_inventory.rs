use std::{collections::BTreeSet, path::Path};

#[test]
fn parity_gap_inventory_covers_every_cataloged_system_and_tool() {
    let sheets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
    let specs = sandbox_catalog::behavior::load(&sheets).unwrap();
    let expected: BTreeSet<_> = specs.iter().map(|s| s.id.as_str()).collect();
    let mut csv = csv::Reader::from_path(sheets.join("parity_gaps.csv")).unwrap();
    assert_eq!(
        csv.headers().unwrap(),
        &csv::StringRecord::from(vec![
            "id",
            "spec_id",
            "area",
            "feature",
            "status",
            "priority",
            "current_state",
            "missing_work",
            "acceptance",
            "evidence",
        ])
    );
    let mut ids = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for row in csv.records() {
        let row = row.unwrap();
        assert_eq!(row.len(), 10);
        assert!(
            row.iter().all(|v| !v.trim().is_empty()),
            "empty field: {row:?}"
        );
        assert!(
            ids.insert(row[0].to_owned()),
            "duplicate gap ID: {}",
            &row[0]
        );
        assert!(expected.contains(&row[1]), "unknown spec: {}", &row[1]);
        assert!([
            "missing",
            "partial",
            "unverified",
            "reference_only",
            "verified"
        ]
        .contains(&&row[4]));
        assert!(["P0", "P1", "P2", "P3"].contains(&&row[5]));
        covered.insert(row[1].to_owned());
    }
    for spec in expected {
        assert!(
            covered.contains(spec),
            "no gap or acceptance row for {spec}"
        );
    }
    for important in [
        "physgun_glow",
        "physgun_light",
        "menu_layout",
        "context_layout",
        "reference_options",
    ] {
        assert!(
            ids.contains(important),
            "missing explicitly requested feature {important}"
        );
    }
}
