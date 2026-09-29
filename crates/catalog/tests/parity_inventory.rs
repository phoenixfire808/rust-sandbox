use std::{collections::BTreeSet, path::Path};

#[test]
fn ordered_work_queue_assigns_every_gap_without_dangling_dependencies() {
    let sheets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
    let gaps: BTreeSet<String> = csv::Reader::from_path(sheets.join("parity_gaps.csv"))
        .unwrap()
        .records()
        .map(|r| r.unwrap()[0].to_owned())
        .collect();
    let mut tasks = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for row in csv::Reader::from_path(sheets.join("work_queue.csv"))
        .unwrap()
        .records()
    {
        let row = row.unwrap();
        assert_eq!(row.len(), 8);
        assert!(row.iter().all(|s| !s.trim().is_empty()));
        for dep in row[3].split('|').filter(|s| *s != "none") {
            assert!(
                tasks.contains(dep),
                "dependency must exist before task: {dep}"
            );
        }
        assert!(tasks.insert(row[0].to_owned()));
        assert!([
            "planned",
            "in_progress",
            "implemented_partial",
            "ongoing",
            "verified"
        ]
        .contains(&&row[4]));
        for gap in row[2].split('|') {
            assert!(gaps.contains(gap), "unknown gap {gap}");
            assert!(covered.insert(gap.to_owned()), "gap assigned twice: {gap}");
        }
    }
    assert_eq!(gaps, covered, "every gap needs an assigned work package");
}

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
