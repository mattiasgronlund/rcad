//! Two bars of radius 1/4 whose axes meet at a slant (Q1646).
//!
//! **Unfinished: this is Q1646's reproduction probe, handed over at the lane
//! budget (§416), and not yet the pins.** `probe` builds the row's pairs —
//! the first bar along `x` from `through − (2, 0, 0)`, length 4; the second
//! turned onto `x` and then by `degrees` about `z`, either 32 long centred on
//! `through` or 2 long starting at it — runs the six fuses of each, and
//! prints what each answers or is refused. It asserts nothing. The next lane
//! replaces it with one pin per fuse, as the row says.

use rcad_geom::{Axis, Deflection, Isometry, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

#[test]
fn probe() {
    for through in [[11.0, 0.0, 0.0], [11.0, -3.25, 0.5]] {
        for degrees in [50.0, 60.0, 70.0, 100.0, 120.0] {
            for (rule, first_first) in six_fuses() {
                let mut model = a_model();
                let [first, second] = two_bars(&mut model, degrees, through);
                if rule == Rule::Union && first_first {
                    for solid in [first, second] {
                        let view = model.solid(solid).unwrap();
                        for face in view.faces() {
                            if let Surface::Cylinder(pipe) = face.surface() {
                                eprintln!(
                                    "  pipe {:?} {:?}",
                                    pipe.axis().origin(),
                                    pipe.axis().direction()
                                );
                            }
                        }
                    }
                }
                let (arguments, tools) = if first_first {
                    ([first], [second])
                } else {
                    ([second], [first])
                };
                let said = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let what = match said {
                    Ok(answer) => {
                        let mut volume = 0.0;
                        let mut faces = Vec::new();
                        for solid in answer.solids() {
                            let view = model.solid(*solid).unwrap();
                            faces.push(view.faces().count());
                            volume += view
                                .measure(
                                    Deflection::of(0.01).unwrap(),
                                    &Policy::certified(),
                                    &mut ProvenanceLog::new(),
                                )
                                .map(|measure| measure.volume())
                                .unwrap_or(f64::NAN);
                        }
                        let valid = model
                            .validate(&Policy::certified(), &mut ProvenanceLog::new())
                            .is_ok();
                        format!("answers {faces:?} volume {volume} valid {valid}")
                    }
                    Err(why) => format!("refused {why:?}"),
                };
                eprintln!("{through:?} {degrees} {rule:?} {first_first}: {what}");
            }
        }
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn a_model() -> Model {
    Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn two_bars(model: &mut Model, degrees: f64, through: [f64; 3]) -> [SolidKey; 2] {
    let onto_x = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let first_at = Isometry::translation(Vector::new(through[0] - 2.0, through[1], through[2]))
        .expect("a finite move")
        .compose(&onto_x);
    let first = model.add_round_bar(0.25, 4.0, &first_at).expect("a bar");
    let turned = Isometry::rotation(&Axis::z(), f64::to_radians(degrees))
        .expect("a turn about z")
        .compose(&onto_x);
    let along = turned.apply_to_vector(Vector::new(0.0, 0.0, 1.0));
    let (start, length) = if through[1] == 0.0 {
        (
            [
                through[0] - 16.0 * along.x(),
                through[1] - 16.0 * along.y(),
                through[2] - 16.0 * along.z(),
            ],
            32.0,
        )
    } else {
        (through, 2.0)
    };
    let second_at = Isometry::translation(Vector::new(start[0], start[1], start[2]))
        .expect("a finite move")
        .compose(&turned);
    let second = model
        .add_round_bar(0.25, length, &second_at)
        .expect("a bar");
    [first, second]
}
