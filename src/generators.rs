//! DXF test file generators for benchmarking.
//!
//! Generates DXF content in memory (as `Vec<u8>`) at various scales and
//! complexity levels using the `dxf` crate as the canonical writer so both
//! libraries parse the same bytes.

use rand::Rng;

/// Entity-count presets for benchmarks.
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    /// ~100 entities
    Small,
    /// ~1 000 entities
    Medium,
    /// ~10 000 entities
    Large,
    /// ~100 000 entities
    Huge,
    /// ~1 000 000 entities
    ExtraHuge,
}

impl Scale {
    pub fn count(self) -> usize {
        match self {
            Scale::Small => 100,
            Scale::Medium => 1_000,
            Scale::Large => 10_000,
            Scale::Huge => 100_000,
            Scale::ExtraHuge => 1_000_000,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Scale::Small => "small_100",
            Scale::Medium => "medium_1k",
            Scale::Large => "large_10k",
            Scale::Huge => "huge_100k",
            Scale::ExtraHuge => "extrahuge_1m",
        }
    }
}

/// Generate a DXF file containing only `LINE` entities.
pub fn generate_lines_only(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let line = dxf::entities::Line {
            p1: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            p2: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Line(line),
        ));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file containing only `CIRCLE` entities.
pub fn generate_circles_only(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let circle = dxf::entities::Circle {
            center: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            radius: rng.gen_range(1.0..500.0),
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Circle(circle),
        ));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file containing only `ARC` entities.
pub fn generate_arcs_only(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let arc = dxf::entities::Arc {
            center: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            radius: rng.gen_range(1.0..500.0),
            start_angle: rng.gen_range(0.0..180.0),
            end_angle: rng.gen_range(180.0..360.0),
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Arc(arc),
        ));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file containing only `ELLIPSE` entities.
pub fn generate_ellipses_only(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let ellipse = dxf::entities::Ellipse {
            center: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            major_axis: dxf::Vector::new(rng.gen_range(10.0..200.0), 0.0, 0.0),
            minor_axis_ratio: rng.gen_range(0.1..1.0),
            start_parameter: 0.0,
            end_parameter: std::f64::consts::TAU,
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Ellipse(ellipse),
        ));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file with a mixed set of entity types.
pub fn generate_mixed(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for i in 0..n {
        let entity_type = match i % 6 {
            0 => {
                let line = dxf::entities::Line {
                    p1: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    p2: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    ..Default::default()
                };
                dxf::entities::EntityType::Line(line)
            }
            1 => {
                let circle = dxf::entities::Circle {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    radius: rng.gen_range(1.0..500.0),
                    ..Default::default()
                };
                dxf::entities::EntityType::Circle(circle)
            }
            2 => {
                let arc = dxf::entities::Arc {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    radius: rng.gen_range(1.0..500.0),
                    start_angle: rng.gen_range(0.0..180.0),
                    end_angle: rng.gen_range(180.0..360.0),
                    ..Default::default()
                };
                dxf::entities::EntityType::Arc(arc)
            }
            3 => {
                let ellipse = dxf::entities::Ellipse {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    major_axis: dxf::Vector::new(rng.gen_range(10.0..200.0), 0.0, 0.0),
                    minor_axis_ratio: rng.gen_range(0.1..1.0),
                    start_parameter: 0.0,
                    end_parameter: std::f64::consts::TAU,
                    ..Default::default()
                };
                dxf::entities::EntityType::Ellipse(ellipse)
            }
            4 => {
                let text = dxf::entities::Text {
                    location: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    value: format!("Text entity #{i}"),
                    text_height: rng.gen_range(1.0..20.0),
                    ..Default::default()
                };
                dxf::entities::EntityType::Text(text)
            }
            5 => {
                let point = dxf::entities::ModelPoint {
                    location: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    ..Default::default()
                };
                dxf::entities::EntityType::ModelPoint(point)
            }
            _ => unreachable!(),
        };

        let mut entity = dxf::entities::Entity::new(entity_type);
        // Assign to layers cyclically
        entity.common.layer = format!("Layer_{}", i % 10);
        drawing.add_entity(entity);
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file with lightweight polylines containing many vertices.
pub fn generate_polylines(scale: Scale) -> Vec<u8> {
    let polyline_count = (scale.count() / 50).max(1);
    let vertices_per = 50;
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for _ in 0..polyline_count {
        let mut poly = dxf::entities::LwPolyline::default();
        for _ in 0..vertices_per {
            poly.vertices.push(dxf::LwPolylineVertex {
                x: rng.gen_range(-1000.0..1000.0),
                y: rng.gen_range(-1000.0..1000.0),
                ..Default::default()
            });
        }
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(poly),
        ));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Generate a DXF file with 3D entities (lines with varying Z, 3DFace).
pub fn generate_3d_entities(scale: Scale) -> Vec<u8> {
    let n = scale.count();
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();

    for i in 0..n {
        let entity_type = match i % 2 {
            0 => {
                let line = dxf::entities::Line {
                    p1: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    p2: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    ..Default::default()
                };
                dxf::entities::EntityType::Line(line)
            }
            1 => {
                let face = dxf::entities::Face3D {
                    first_corner: dxf::Point::new(
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    second_corner: dxf::Point::new(
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    third_corner: dxf::Point::new(
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    fourth_corner: dxf::Point::new(
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-500.0..500.0),
                    ),
                    ..Default::default()
                };
                dxf::entities::EntityType::Face3D(face)
            }
            _ => unreachable!(),
        };
        drawing.add_entity(dxf::entities::Entity::new(entity_type));
    }

    let mut buf = Vec::new();
    drawing.save(&mut buf).expect("failed to write DXF");
    buf
}

/// Returns all (label, data) pairs for a given scale (ASCII DXF).
pub fn all_variants(scale: Scale) -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("lines_only", generate_lines_only(scale)),
        ("circles_only", generate_circles_only(scale)),
        ("arcs_only", generate_arcs_only(scale)),
        ("ellipses_only", generate_ellipses_only(scale)),
        ("mixed", generate_mixed(scale)),
        ("polylines", generate_polylines(scale)),
        ("3d_entities", generate_3d_entities(scale)),
    ]
}

// ---------------------------------------------------------------------------
// Binary DXF generators
// ---------------------------------------------------------------------------

/// Generate a binary DXF from an existing dxf::Drawing.
fn drawing_to_binary(drawing: &dxf::Drawing) -> Vec<u8> {
    let mut buf = Vec::new();
    drawing.save_binary(&mut buf).expect("failed to write binary DXF");
    buf
}

/// Generate a binary DXF file with mixed entities at the given scale.
pub fn generate_mixed_binary(scale: Scale) -> Vec<u8> {
    let ascii = generate_mixed(scale);
    let drawing = dxf::Drawing::load(&mut std::io::Cursor::new(&ascii)).unwrap();
    drawing_to_binary(&drawing)
}

/// Generate a binary DXF file with lines at the given scale.
pub fn generate_lines_binary(scale: Scale) -> Vec<u8> {
    let ascii = generate_lines_only(scale);
    let drawing = dxf::Drawing::load(&mut std::io::Cursor::new(&ascii)).unwrap();
    drawing_to_binary(&drawing)
}

// ---------------------------------------------------------------------------
// DWG generators (acadrust only – dxf-rs has no DWG support)
// ---------------------------------------------------------------------------

/// Generate a DWG file with mixed entities at the given scale.
pub fn generate_mixed_dwg(scale: Scale) -> Vec<u8> {
    let doc = build_acadrust_mixed(scale.count());
    acadrust::DwgWriter::write_to_vec(&doc).expect("failed to write DWG")
}

/// Generate a DWG file with lines at the given scale.
pub fn generate_lines_dwg(scale: Scale) -> Vec<u8> {
    let doc = build_acadrust_lines(scale.count());
    acadrust::DwgWriter::write_to_vec(&doc).expect("failed to write DWG")
}

/// Builds an `acadrust::CadDocument` with lines for writing benchmarks.
pub fn build_acadrust_lines(n: usize) -> acadrust::CadDocument {
    let mut doc = acadrust::CadDocument::new();
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        let line = acadrust::entities::Line::from_coords(
            rng.gen_range(-1000.0..1000.0),
            rng.gen_range(-1000.0..1000.0),
            0.0,
            rng.gen_range(-1000.0..1000.0),
            rng.gen_range(-1000.0..1000.0),
            0.0,
        );
        let _ = doc.add_entity(acadrust::entities::EntityType::Line(line));
    }
    doc
}

/// Builds an `acadrust::CadDocument` with mixed entities for writing benchmarks.
pub fn build_acadrust_mixed(n: usize) -> acadrust::CadDocument {
    let mut doc = acadrust::CadDocument::new();
    let mut rng = rand::thread_rng();
    for i in 0..n {
        let et = match i % 4 {
            0 => {
                let line = acadrust::entities::Line::from_coords(
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                    0.0,
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                    0.0,
                );
                acadrust::entities::EntityType::Line(line)
            }
            1 => {
                let mut circle = acadrust::entities::Circle::new();
                circle.center = acadrust::types::Vector3::new(
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                    0.0,
                );
                circle.radius = rng.gen_range(1.0..500.0);
                acadrust::entities::EntityType::Circle(circle)
            }
            2 => {
                let mut arc = acadrust::entities::Arc::new();
                arc.center = acadrust::types::Vector3::new(
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                    0.0,
                );
                arc.radius = rng.gen_range(1.0..500.0);
                arc.start_angle = rng.gen_range(0.0..180.0);
                arc.end_angle = rng.gen_range(180.0..360.0);
                acadrust::entities::EntityType::Arc(arc)
            }
            3 => {
                let mut ellipse = acadrust::entities::Ellipse::new();
                ellipse.center = acadrust::types::Vector3::new(
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                    0.0,
                );
                ellipse.major_axis = acadrust::types::Vector3::new(
                    rng.gen_range(10.0..200.0),
                    0.0,
                    0.0,
                );
                ellipse.minor_axis_ratio = rng.gen_range(0.1..1.0);
                acadrust::entities::EntityType::Ellipse(ellipse)
            }
            _ => unreachable!(),
        };
        let _ = doc.add_entity(et);
    }
    doc
}

/// Builds a `dxf::Drawing` with lines for writing benchmarks.
pub fn build_dxf_lines(n: usize) -> dxf::Drawing {
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        let line = dxf::entities::Line {
            p1: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            p2: dxf::Point::new(
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
                0.0,
            ),
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Line(line),
        ));
    }
    drawing
}

/// Builds a `dxf::Drawing` with mixed entities for writing benchmarks.
pub fn build_dxf_mixed(n: usize) -> dxf::Drawing {
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2000;
    let mut rng = rand::thread_rng();
    for i in 0..n {
        let entity_type = match i % 4 {
            0 => {
                let line = dxf::entities::Line {
                    p1: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    p2: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    ..Default::default()
                };
                dxf::entities::EntityType::Line(line)
            }
            1 => {
                let circle = dxf::entities::Circle {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    radius: rng.gen_range(1.0..500.0),
                    ..Default::default()
                };
                dxf::entities::EntityType::Circle(circle)
            }
            2 => {
                let arc = dxf::entities::Arc {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    radius: rng.gen_range(1.0..500.0),
                    start_angle: rng.gen_range(0.0..180.0),
                    end_angle: rng.gen_range(180.0..360.0),
                    ..Default::default()
                };
                dxf::entities::EntityType::Arc(arc)
            }
            3 => {
                let ellipse = dxf::entities::Ellipse {
                    center: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    major_axis: dxf::Vector::new(rng.gen_range(10.0..200.0), 0.0, 0.0),
                    minor_axis_ratio: rng.gen_range(0.1..1.0),
                    start_parameter: 0.0,
                    end_parameter: std::f64::consts::TAU,
                    ..Default::default()
                };
                dxf::entities::EntityType::Ellipse(ellipse)
            }
            _ => unreachable!(),
        };
        drawing.add_entity(dxf::entities::Entity::new(entity_type));
    }
    drawing
}
