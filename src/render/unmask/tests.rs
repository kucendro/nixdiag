use super::*;

fn r(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect { x, y, w, h }
}

fn area(rects: &[Rect]) -> f64 {
    rects.iter().map(|r| r.w * r.h).sum()
}

#[test]
fn minus_leaves_a_disjoint_rect_alone() {
    assert_eq!(
        r(0.0, 0.0, 10.0, 10.0).minus(r(20.0, 20.0, 5.0, 5.0)),
        vec![r(0.0, 0.0, 10.0, 10.0)]
    );
}

#[test]
fn minus_cuts_an_interior_hole_into_four_bands() {
    let pieces = r(0.0, 0.0, 10.0, 10.0).minus(r(2.0, 3.0, 4.0, 5.0));
    assert_eq!(pieces.len(), 4);
    assert_eq!(area(&pieces), 100.0 - 20.0);
    assert_eq!(pieces[0], r(0.0, 0.0, 10.0, 3.0));
    assert_eq!(pieces[1], r(0.0, 8.0, 10.0, 2.0));
    assert_eq!(pieces[2], r(0.0, 3.0, 2.0, 5.0));
    assert_eq!(pieces[3], r(6.0, 3.0, 4.0, 5.0));
}

#[test]
fn minus_clips_a_hole_that_crosses_the_edge() {
    let pieces = r(0.0, 0.0, 10.0, 10.0).minus(r(8.0, -5.0, 10.0, 8.0));
    assert_eq!(area(&pieces), 100.0 - 2.0 * 3.0);
    assert!(pieces.iter().all(|p| p.w > 0.0 && p.h > 0.0));
}

#[test]
fn overlapping_holes_are_subtracted_once() {
    let m = Mask {
        id: "m".into(),
        base: r(0.0, 0.0, 10.0, 10.0),
        holes: vec![r(1.0, 1.0, 4.0, 4.0), r(3.0, 3.0, 4.0, 4.0)],
    };
    let mut region = vec![m.base];
    for hole in &m.holes {
        region = region.into_iter().flat_map(|p| p.minus(*hole)).collect();
    }
    assert_eq!(area(&region), 100.0 - (16.0 + 16.0 - 4.0));
}

const D2: &str = concat!(
    "<svg>\n",
    "<mask id=\"d2-1\" maskUnits=\"userSpaceOnUse\" x=\"-89\" y=\"-89\" width=\"200\" height=\"200\">\n",
    "<rect x=\"-89\" y=\"-89\" width=\"200\" height=\"200\" fill=\"white\"></rect>\n",
    "<rect x=\"10.000000\" y=\"20.000000\" width=\"30\" height=\"21\" fill=\"black\"></rect>\n",
    "</mask>\n",
    "<path d=\"M 0 0 L 1 1\" class=\"connection\" marker-end=\"url(#mk-d2-1-9)\" mask=\"url(#d2-1)\" />\n",
    "<path d=\"M 0 0 L 2 2\" class=\"connection\" mask=\"url(#d2-1)\" />\n",
    "</svg>\n"
);

#[test]
fn label_mask_becomes_one_clip_path_of_disjoint_rects() {
    let out = unmask(D2);
    assert!(!out.contains("<mask"), "{out}");
    assert!(!out.contains(" mask="), "{out}");
    assert_eq!(out.matches(" clip-path=\"url(#d2-1)\"").count(), 2);
    assert_eq!(out.matches("<clipPath id=\"d2-1\">").count(), 1);
    assert_eq!(
        out.matches("<path d=\"M-89 -89h200v109h-200z").count(),
        1,
        "{out}"
    );
    assert!(out.contains("marker-end=\"url(#mk-d2-1-9)\""));
}

#[test]
fn mask_without_labels_is_dropped_with_its_references() {
    let svg = D2.replace(
        "<rect x=\"10.000000\" y=\"20.000000\" width=\"30\" height=\"21\" fill=\"black\"></rect>\n",
        "",
    );
    let out = unmask(&svg);
    assert!(!out.contains("mask"), "{out}");
    assert!(!out.contains("clip"), "{out}");
    assert_eq!(out.matches("<path d=\"M 0 0").count(), 2);
}

#[test]
fn unfamiliar_mask_is_left_alone() {
    for foreign in [
        D2.replace("fill=\"black\"", "fill=\"#000\""),
        D2.replace(
            "<rect x=\"10.000000\"",
            "<circle r=\"3\"/><rect x=\"10.000000\"",
        ),
        D2.replace("fill=\"white\"", "fill=\"black\""),
    ] {
        assert_eq!(unmask(&foreign), foreign);
    }
}

#[test]
fn attr_needs_the_leading_space() {
    let tag = "<path style=\"stroke-width:2\" width=\"7\"";
    assert_eq!(attr(tag, "width"), Some("7"));
    assert_eq!(attr("<path stroke-width=\"2\"", "width"), None);
}
