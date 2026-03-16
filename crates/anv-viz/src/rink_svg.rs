// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! SVG rink diagram renderer.
//!
//! Generates SVG visualizations of skating programs on a rink diagram.

use anv_ir::rink::RinkDimensions;
use anv_ir::timeline::{EventKind, Timeline};
use std::fmt::Write as FmtWrite;

/// SVG rendering options.
#[derive(Debug, Clone)]
pub struct SvgOptions {
    /// Width of the SVG in pixels.
    pub width: f64,
    /// Height of the SVG in pixels.
    pub height: f64,
    /// Padding around the rink.
    pub padding: f64,
    /// Rink dimensions to use.
    pub rink: RinkDimensions,
    /// Show element labels.
    pub show_labels: bool,
    /// Show element paths.
    pub show_paths: bool,
    /// Show ice markings (center line, circles, etc.).
    pub show_markings: bool,
    /// Color for jump elements.
    pub jump_color: String,
    /// Color for spin elements.
    pub spin_color: String,
    /// Color for step sequences.
    pub step_color: String,
    /// Color for pairs elements.
    pub pairs_color: String,
    /// Color for paths/traces.
    pub path_color: String,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            padding: 20.0,
            rink: RinkDimensions::OLYMPIC,
            show_labels: true,
            show_paths: true,
            show_markings: true,
            jump_color: "#e74c3c".to_string(),    // Red
            spin_color: "#3498db".to_string(),    // Blue
            step_color: "#2ecc71".to_string(),    // Green
            pairs_color: "#9b59b6".to_string(),   // Purple
            path_color: "#95a5a6".to_string(),    // Gray
        }
    }
}

/// Renders skating programs as SVG rink diagrams.
pub struct RinkRenderer {
    options: SvgOptions,
}

impl RinkRenderer {
    /// Create a new renderer with the given options.
    pub fn new(options: SvgOptions) -> Self {
        Self { options }
    }

    /// Create a renderer with default options.
    pub fn default_renderer() -> Self {
        Self::new(SvgOptions::default())
    }

    /// Render a timeline to SVG.
    pub fn render(&self, timeline: &Timeline) -> String {
        let mut svg = String::new();

        // SVG header
        writeln!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            self.options.width,
            self.options.height,
            self.options.width,
            self.options.height
        )
        .unwrap();

        // Add styles
        self.write_styles(&mut svg);

        // Background
        writeln!(
            &mut svg,
            r##"  <rect width="100%" height="100%" fill="#f8f9fa"/>"##
        )
        .unwrap();

        // Draw rink
        self.draw_rink(&mut svg);

        // Draw ice markings
        if self.options.show_markings {
            self.draw_markings(&mut svg);
        }

        // Draw element paths
        if self.options.show_paths && !timeline.events.is_empty() {
            self.draw_paths(&mut svg, timeline);
        }

        // Draw elements
        self.draw_elements(&mut svg, timeline);

        // Title
        writeln!(
            &mut svg,
            r#"  <text x="{}" y="15" class="title">{}</text>"#,
            self.options.width / 2.0,
            timeline.name
        )
        .unwrap();

        // Close SVG
        writeln!(&mut svg, "</svg>").unwrap();

        svg
    }

    fn write_styles(&self, svg: &mut String) {
        writeln!(svg, "  <style>").unwrap();
        writeln!(svg, "    .title {{ font-family: sans-serif; font-size: 14px; font-weight: bold; text-anchor: middle; }}").unwrap();
        writeln!(svg, "    .label {{ font-family: monospace; font-size: 8px; text-anchor: middle; fill: #333; }}").unwrap();
        writeln!(svg, "    .rink {{ fill: #e8f4f8; stroke: #2980b9; stroke-width: 2; }}").unwrap();
        writeln!(svg, "    .marking {{ fill: none; stroke: #3498db; stroke-width: 1; stroke-dasharray: 5,5; }}").unwrap();
        writeln!(svg, "    .path {{ fill: none; stroke: {}; stroke-width: 1; opacity: 0.5; }}", self.options.path_color).unwrap();
        writeln!(svg, "    .jump {{ fill: {}; }}", self.options.jump_color).unwrap();
        writeln!(svg, "    .spin {{ fill: {}; }}", self.options.spin_color).unwrap();
        writeln!(svg, "    .step {{ fill: {}; }}", self.options.step_color).unwrap();
        writeln!(svg, "    .pairs {{ fill: {}; }}", self.options.pairs_color).unwrap();
        writeln!(svg, "  </style>").unwrap();
    }

    fn draw_rink(&self, svg: &mut String) {
        let scale = self.calculate_scale();
        let cx = self.options.width / 2.0;
        let cy = self.options.height / 2.0;

        let half_length = self.options.rink.length / 2.0 * scale;
        let half_width = self.options.rink.width / 2.0 * scale;
        let corner = self.options.rink.corner_radius * scale;

        // Draw rink outline with rounded corners
        writeln!(
            svg,
            r#"  <rect x="{}" y="{}" width="{}" height="{}" rx="{}" ry="{}" class="rink"/>"#,
            cx - half_length,
            cy - half_width,
            half_length * 2.0,
            half_width * 2.0,
            corner,
            corner
        )
        .unwrap();
    }

    fn draw_markings(&self, svg: &mut String) {
        let scale = self.calculate_scale();
        let cx = self.options.width / 2.0;
        let cy = self.options.height / 2.0;
        let half_length = self.options.rink.length / 2.0 * scale;

        // Center line
        writeln!(
            svg,
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="marking"/>"#,
            cx,
            cy - self.options.rink.width / 2.0 * scale,
            cx,
            cy + self.options.rink.width / 2.0 * scale
        )
        .unwrap();

        // Center circle
        writeln!(
            svg,
            r#"  <circle cx="{}" cy="{}" r="{}" class="marking"/>"#,
            cx,
            cy,
            4.0 * scale // ~4m radius center circle
        )
        .unwrap();

        // Center dot
        writeln!(
            svg,
            r##"  <circle cx="{}" cy="{}" r="3" fill="#3498db"/>"##,
            cx, cy
        )
        .unwrap();

        // Goal lines (approximation)
        let goal_line_offset = 23.0 * scale; // ~23m from center
        for offset in [-goal_line_offset, goal_line_offset] {
            writeln!(
                svg,
                r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="marking"/>"#,
                cx + offset,
                cy - self.options.rink.width / 2.0 * scale,
                cx + offset,
                cy + self.options.rink.width / 2.0 * scale
            )
            .unwrap();
        }

        // Judges label
        writeln!(
            svg,
            r#"  <text x="{}" y="{}" class="label">JUDGES</text>"#,
            cx + half_length - 30.0,
            cy
        )
        .unwrap();
    }

    fn draw_paths(&self, svg: &mut String, timeline: &Timeline) {
        let scale = self.calculate_scale();
        let cx = self.options.width / 2.0;
        let cy = self.options.height / 2.0;

        let mut path = String::from("M ");
        let mut first = true;

        for event in &timeline.events {
            if !event.kind.is_element() {
                continue;
            }

            let x = cx + event.position.x() * scale;
            let y = cy - event.position.y() * scale; // Flip Y for SVG coordinates

            if first {
                write!(&mut path, "{:.1} {:.1}", x, y).unwrap();
                first = false;
            } else {
                write!(&mut path, " L {:.1} {:.1}", x, y).unwrap();
            }
        }

        if !first {
            writeln!(svg, r#"  <path d="{}" class="path"/>"#, path).unwrap();
        }
    }

    fn draw_elements(&self, svg: &mut String, timeline: &Timeline) {
        let scale = self.calculate_scale();
        let cx = self.options.width / 2.0;
        let cy = self.options.height / 2.0;

        for event in &timeline.events {
            if !event.kind.is_element() {
                continue;
            }

            let x = cx + event.position.x() * scale;
            let y = cy - event.position.y() * scale;

            let (class, radius) = self.element_style(&event.kind);

            // Draw element marker
            writeln!(
                svg,
                r#"  <circle cx="{:.1}" cy="{:.1}" r="{}" class="{}"/>"#,
                x, y, radius, class
            )
            .unwrap();

            // Draw label
            if self.options.show_labels {
                if let Some(code) = &event.isu_code {
                    writeln!(
                        svg,
                        r#"  <text x="{:.1}" y="{:.1}" class="label">{}</text>"#,
                        x,
                        y - radius - 3.0,
                        code
                    )
                    .unwrap();
                }
            }
        }
    }

    fn element_style(&self, kind: &EventKind) -> (&str, f64) {
        match kind {
            EventKind::Jump { .. } | EventKind::JumpCombination { .. } => ("jump", 6.0),
            EventKind::Spin { .. } => ("spin", 8.0),
            EventKind::StepSequence { .. } | EventKind::ChoreographicSequence => ("step", 5.0),
            EventKind::Lift { .. }
            | EventKind::Throw { .. }
            | EventKind::Twist { .. }
            | EventKind::DeathSpiral { .. } => ("pairs", 7.0),
            _ => ("step", 4.0),
        }
    }

    fn calculate_scale(&self) -> f64 {
        let available_width = self.options.width - 2.0 * self.options.padding;
        let available_height = self.options.height - 2.0 * self.options.padding;

        let scale_x = available_width / self.options.rink.length;
        let scale_y = available_height / self.options.rink.width;

        scale_x.min(scale_y)
    }
}

/// Render a timeline directly to SVG string.
pub fn render_to_svg(timeline: &Timeline) -> String {
    RinkRenderer::default_renderer().render(timeline)
}

/// Render a timeline to an SVG file.
pub fn render_to_file(timeline: &Timeline, path: &std::path::Path) -> std::io::Result<()> {
    let svg = render_to_svg(timeline);
    std::fs::write(path, svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_ir::timeline::Event;
    use anv_ir::types::{Duration, TimeCode};
    use anv_core::skating::{JumpKind, Rotations};
    use anv_ir::rink::Position;

    #[test]
    fn test_render_empty_timeline() {
        let timeline = Timeline::new("empty");
        let svg = render_to_svg(&timeline);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("empty")); // Title
    }

    #[test]
    fn test_render_with_elements() {
        let mut timeline = Timeline::new("test_program");

        timeline.push_event(
            Event::new(1, EventKind::Jump {
                rotations: Rotations::Triple,
                kind: JumpKind::Axel,
            }, TimeCode::from_secs(10.0))
            .with_duration(Duration::from_secs(3.0))
            .with_position(Position::new(10.0, 5.0))
            .with_isu_code("3A")
        );

        let svg = render_to_svg(&timeline);

        assert!(svg.contains("3A")); // ISU code label
        assert!(svg.contains("class=\"jump\"")); // Jump element class
    }

    #[test]
    fn test_svg_options() {
        let options = SvgOptions {
            width: 1000.0,
            height: 500.0,
            show_labels: false,
            ..Default::default()
        };

        let renderer = RinkRenderer::new(options);
        let timeline = Timeline::new("custom");
        let svg = renderer.render(&timeline);

        assert!(svg.contains("width=\"1000\""));
        assert!(svg.contains("height=\"500\""));
    }
}
