// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

//! SVG timeline chart renderer.
//!
//! Generates timeline visualizations showing element timing and duration.

use anv_ir::timeline::{EventKind, Timeline};
use std::fmt::Write as FmtWrite;

/// Timeline rendering options.
#[derive(Debug, Clone)]
pub struct TimelineOptions {
    /// Width of the SVG in pixels.
    pub width: f64,
    /// Height of the SVG in pixels.
    pub height: f64,
    /// Padding around the chart.
    pub padding: f64,
    /// Height of element bars.
    pub bar_height: f64,
    /// Gap between bars.
    pub bar_gap: f64,
    /// Show time labels.
    pub show_time_labels: bool,
    /// Show element labels.
    pub show_element_labels: bool,
}

impl Default for TimelineOptions {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 300.0,
            padding: 40.0,
            bar_height: 20.0,
            bar_gap: 5.0,
            show_time_labels: true,
            show_element_labels: true,
        }
    }
}

/// Renders skating programs as SVG timeline charts.
pub struct TimelineRenderer {
    options: TimelineOptions,
}

impl TimelineRenderer {
    /// Create a new renderer with the given options.
    pub fn new(options: TimelineOptions) -> Self {
        Self { options }
    }

    /// Create a renderer with default options.
    pub fn default_renderer() -> Self {
        Self::new(TimelineOptions::default())
    }

    /// Render a timeline to SVG.
    pub fn render(&self, timeline: &Timeline) -> String {
        let mut svg = String::new();

        let chart_width = self.options.width - 2.0 * self.options.padding;
        let chart_height = self.options.height - 2.0 * self.options.padding;
        let total_duration = timeline.duration.as_secs().max(1.0);

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

        // Styles
        self.write_styles(&mut svg);

        // Background
        writeln!(&mut svg, r##"  <rect width="100%" height="100%" fill="#fff"/>"##).unwrap();

        // Title
        writeln!(
            &mut svg,
            r#"  <text x="{}" y="20" class="title">{} - Timeline</text>"#,
            self.options.width / 2.0,
            timeline.name
        )
        .unwrap();

        // Time axis
        self.draw_time_axis(&mut svg, total_duration, chart_width);

        // Segment markers
        for segment in &timeline.segments {
            let x = self.options.padding + (segment.start.as_secs() / total_duration) * chart_width;
            let width = ((segment.end.as_secs() - segment.start.as_secs()) / total_duration) * chart_width;

            writeln!(
                &mut svg,
                r#"  <rect x="{:.1}" y="{}" width="{:.1}" height="{}" class="segment"/>"#,
                x,
                self.options.padding + 20.0,
                width,
                chart_height - 40.0
            )
            .unwrap();

            writeln!(
                &mut svg,
                r#"  <text x="{:.1}" y="{}" class="segment-label">{}: {}</text>"#,
                x + 5.0,
                self.options.padding + 35.0,
                segment.name,
                segment.kind
            )
            .unwrap();
        }

        // Element bars
        let mut y = self.options.padding + 50.0;
        for event in &timeline.events {
            if !event.kind.is_element() {
                continue;
            }

            let x = self.options.padding + (event.start.as_secs() / total_duration) * chart_width;
            let width = (event.duration.as_secs() / total_duration) * chart_width;
            let class = self.element_class(&event.kind);

            writeln!(
                &mut svg,
                r#"  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{}" rx="3" class="{}"/>"#,
                x.max(self.options.padding),
                y,
                width.max(5.0),
                self.options.bar_height,
                class
            )
            .unwrap();

            if self.options.show_element_labels {
                if let Some(code) = &event.isu_code {
                    writeln!(
                        &mut svg,
                        r#"  <text x="{:.1}" y="{:.1}" class="bar-label">{}</text>"#,
                        x + 3.0,
                        y + 14.0,
                        code
                    )
                    .unwrap();
                }
            }

            y += self.options.bar_height + self.options.bar_gap;

            // Wrap to next row if needed
            if y > self.options.height - self.options.padding - 30.0 {
                y = self.options.padding + 50.0;
            }
        }

        // Close SVG
        writeln!(&mut svg, "</svg>").unwrap();

        svg
    }

    fn write_styles(&self, svg: &mut String) {
        writeln!(svg, "  <style>").unwrap();
        writeln!(svg, "    .title {{ font-family: sans-serif; font-size: 16px; font-weight: bold; text-anchor: middle; }}").unwrap();
        writeln!(svg, "    .time-label {{ font-family: monospace; font-size: 10px; text-anchor: middle; fill: #666; }}").unwrap();
        writeln!(svg, "    .segment {{ fill: #ecf0f1; stroke: #bdc3c7; stroke-width: 1; }}").unwrap();
        writeln!(svg, "    .segment-label {{ font-family: sans-serif; font-size: 10px; fill: #7f8c8d; }}").unwrap();
        writeln!(svg, "    .bar-label {{ font-family: monospace; font-size: 9px; fill: white; }}").unwrap();
        writeln!(svg, "    .axis {{ stroke: #333; stroke-width: 1; }}").unwrap();
        writeln!(svg, "    .tick {{ stroke: #666; stroke-width: 1; }}").unwrap();
        writeln!(svg, "    .jump {{ fill: #e74c3c; }}").unwrap();
        writeln!(svg, "    .spin {{ fill: #3498db; }}").unwrap();
        writeln!(svg, "    .step {{ fill: #2ecc71; }}").unwrap();
        writeln!(svg, "    .pairs {{ fill: #9b59b6; }}").unwrap();
        writeln!(svg, "    .choreo {{ fill: #f39c12; }}").unwrap();
        writeln!(svg, "  </style>").unwrap();
    }

    fn draw_time_axis(&self, svg: &mut String, total_duration: f64, chart_width: f64) {
        let y = self.options.height - self.options.padding;

        // Axis line
        writeln!(
            svg,
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>"#,
            self.options.padding,
            y,
            self.options.padding + chart_width,
            y
        )
        .unwrap();

        // Time ticks (every 30 seconds)
        let tick_interval = 30.0;
        let mut t = 0.0;
        while t <= total_duration {
            let x = self.options.padding + (t / total_duration) * chart_width;

            writeln!(
                svg,
                r#"  <line x1="{:.1}" y1="{}" x2="{:.1}" y2="{}" class="tick"/>"#,
                x,
                y,
                x,
                y + 5.0
            )
            .unwrap();

            if self.options.show_time_labels {
                let mins = (t / 60.0) as u32;
                let secs = (t % 60.0) as u32;
                writeln!(
                    svg,
                    r#"  <text x="{:.1}" y="{}" class="time-label">{}:{:02}</text>"#,
                    x,
                    y + 18.0,
                    mins,
                    secs
                )
                .unwrap();
            }

            t += tick_interval;
        }
    }

    fn element_class(&self, kind: &EventKind) -> &'static str {
        match kind {
            EventKind::Jump { .. } | EventKind::JumpCombination { .. } => "jump",
            EventKind::Spin { .. } => "spin",
            EventKind::StepSequence { .. } => "step",
            EventKind::Choreographic { .. } | EventKind::ChoreographicSequence => "choreo",
            EventKind::Lift { .. }
            | EventKind::Throw { .. }
            | EventKind::Twist { .. }
            | EventKind::DeathSpiral { .. }
            | EventKind::PatternDance { .. }
            | EventKind::Twizzle { .. } => "pairs",
            _ => "step",
        }
    }
}

/// Render a timeline chart directly to SVG string.
pub fn render_timeline_to_svg(timeline: &Timeline) -> String {
    TimelineRenderer::default_renderer().render(timeline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anv_ir::timeline::{Event, SegmentMarker};
    use anv_ir::types::{Duration, TimeCode};
    use anv_core::skating::{JumpKind, Rotations, Level, SpinPosition};
    use anv_ir::rink::Position;

    #[test]
    fn test_render_timeline_chart() {
        let mut timeline = Timeline::new("test_timeline");

        timeline.push_segment(SegmentMarker {
            name: "sp".to_string(),
            kind: "short".to_string(),
            start: TimeCode::ZERO,
            end: TimeCode::from_secs(160.0),
        });

        timeline.push_event(
            Event::new(1, EventKind::Jump {
                rotations: Rotations::Triple,
                kind: JumpKind::Axel,
            }, TimeCode::from_secs(15.0))
            .with_duration(Duration::from_secs(3.0))
            .with_position(Position::CENTER)
            .with_isu_code("3A")
        );

        timeline.push_event(
            Event::new(2, EventKind::Spin {
                positions: vec![SpinPosition::Camel],
                level: Level::L4,
                flying: false,
                change_foot: false,
            }, TimeCode::from_secs(30.0))
            .with_duration(Duration::from_secs(10.0))
            .with_position(Position::CENTER)
            .with_isu_code("CSp4")
        );

        let svg = render_timeline_to_svg(&timeline);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Timeline"));
        assert!(svg.contains("3A"));
        assert!(svg.contains("CSp4"));
        assert!(svg.contains("class=\"jump\""));
        assert!(svg.contains("class=\"spin\""));
    }
}
