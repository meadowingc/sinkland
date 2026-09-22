use super::{Experiment, Figure};
use plotters::prelude::*;

pub const NAMES: [&str; 8] = [
    "Multi-series response curves",
    "Observation scatterplot",
    "Group mean comparison",
    "Response distribution",
    "Box-and-whisker comparison",
    "Measurement heatmap",
    "Mean and uncertainty intervals",
    "Stacked response areas",
];
const COLORS: [RGBColor; 3] = [
    RGBColor(26, 91, 140),
    RGBColor(184, 72, 46),
    RGBColor(51, 126, 82),
];

pub fn render(data: &Experiment, figure: &Figure) -> Result<String, String> {
    if figure.kind >= NAMES.len()
        || data.series.len() != 3
        || data
            .series
            .iter()
            .any(|series| series.values.is_empty() || series.values.iter().any(|v| !v.is_finite()))
        || data
            .series
            .iter()
            .any(|series| series.values.len() != data.series[0].values.len())
    {
        return Err("Invalid figure data".to_owned());
    }
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (900, 500)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| e.to_string())?;
        let n = data.series[0].values.len();
        let min = data
            .series
            .iter()
            .map(|s| s.summary.min.min(s.summary.low))
            .fold(f64::INFINITY, f64::min);
        let max = data
            .series
            .iter()
            .map(|s| s.summary.max.max(s.summary.high))
            .fold(f64::NEG_INFINITY, f64::max);
        let padding = ((max - min) * 0.15).max(1.0);
        let grouped = matches!(figure.kind, 2 | 4 | 6);
        let x_max = if grouped { 3.0 } else { n as f64 };
        let y_max = if figure.kind == 7 {
            max.max(1.0) * 3.3
        } else {
            max + padding
        };
        if figure.kind == 3 {
            let bins = 12;
            let width = ((max - min) / bins as f64).max(0.01);
            let mut histogram = vec![0_u32; bins];
            for value in &data.series[1].values {
                let index = (((value - min) / width).floor() as usize).min(bins - 1);
                histogram[index] += 1;
            }
            let peak = *histogram.iter().max().unwrap_or(&1);
            let mut chart = ChartBuilder::on(&root)
                .caption(
                    format!("{} — proposed method", figure.name),
                    ("sans-serif", 22),
                )
                .margin(20)
                .x_label_area_size(45)
                .y_label_area_size(65)
                .build_cartesian_2d(min..min + bins as f64 * width, 0_u32..peak + 2)
                .map_err(|e| e.to_string())?;
            chart
                .configure_mesh()
                .x_desc(&data.unit)
                .y_desc("Observation count")
                .draw()
                .map_err(|e| e.to_string())?;
            chart
                .draw_series(histogram.iter().enumerate().map(|(index, count)| {
                    Rectangle::new(
                        [
                            (min + index as f64 * width, 0),
                            (min + (index + 1) as f64 * width, *count),
                        ],
                        COLORS[1].mix(0.7).filled(),
                    )
                }))
                .map_err(|e| e.to_string())?;
        } else if figure.kind == 5 {
            let mut chart = ChartBuilder::on(&root)
                .caption(&figure.name, ("sans-serif", 24))
                .margin(20)
                .x_label_area_size(45)
                .y_label_area_size(65)
                .build_cartesian_2d(0..n as i32, 0..3_i32)
                .map_err(|e| e.to_string())?;
            chart
                .configure_mesh()
                .disable_mesh()
                .x_desc("Observation index")
                .y_desc("Group index")
                .draw()
                .map_err(|e| e.to_string())?;
            for (group, series) in data.series.iter().enumerate() {
                chart
                    .draw_series(series.values.iter().enumerate().map(|(index, value)| {
                        let normalized = ((value - min) / (max - min).max(0.01)).clamp(0.0, 1.0);
                        Rectangle::new(
                            [
                                (index as i32, group as i32),
                                (index as i32 + 1, group as i32 + 1),
                            ],
                            HSLColor(0.66 - normalized * 0.66, 0.65, 0.5).filled(),
                        )
                    }))
                    .map_err(|e| e.to_string())?;
            }
            root.draw(&Text::new(
                format!(
                    "Groups: 0 baseline, 1 proposed, 2 control. Blue {min:.1} → red {max:.1} {}",
                    data.unit
                ),
                (60, 485),
                ("sans-serif", 14),
            ))
            .map_err(|e| e.to_string())?;
        } else {
            let mut chart = ChartBuilder::on(&root)
                .caption(&figure.name, ("sans-serif", 24))
                .margin(20)
                .x_label_area_size(50)
                .y_label_area_size(65)
                .build_cartesian_2d(
                    0.0..x_max,
                    if grouped || figure.kind == 7 {
                        0.0_f64.min(min - padding)
                    } else {
                        min - padding
                    }..y_max,
                )
                .map_err(|e| e.to_string())?;
            chart
                .configure_mesh()
                .x_desc(if grouped {
                    "Groups: 0.5 baseline, 1.5 proposed, 2.5 control"
                } else {
                    "Observation index"
                })
                .y_desc(&data.unit)
                .draw()
                .map_err(|e| e.to_string())?;
            for (group, series) in data.series.iter().enumerate() {
                let color = COLORS[group];
                let x = group as f64 + 0.5;
                let s = &series.summary;
                match figure.kind {
                    0 => {
                        chart
                            .draw_series(LineSeries::new(
                                series
                                    .values
                                    .iter()
                                    .enumerate()
                                    .map(|(i, y)| (i as f64, *y)),
                                color.stroke_width(2),
                            ))
                            .map_err(|e| e.to_string())?
                            .label(&series.name)
                            .legend(move |(x, y)| {
                                PathElement::new(vec![(x, y), (x + 20, y)], color)
                            });
                    }
                    1 => {
                        chart
                            .draw_series(series.values.iter().enumerate().map(|(i, y)| {
                                Circle::new((i as f64, *y), 3, color.mix(0.7).filled())
                            }))
                            .map_err(|e| e.to_string())?
                            .label(&series.name)
                            .legend(move |(x, y)| Circle::new((x + 10, y), 4, color.filled()));
                    }
                    2 => {
                        chart
                            .draw_series(std::iter::once(Rectangle::new(
                                [(x - 0.3, 0.0), (x + 0.3, s.mean)],
                                color.mix(0.75).filled(),
                            )))
                            .map_err(|e| e.to_string())?;
                    }
                    4 => {
                        chart
                            .draw_series(std::iter::once(Rectangle::new(
                                [(x - 0.25, s.q1), (x + 0.25, s.q3)],
                                color.mix(0.35).filled(),
                            )))
                            .map_err(|e| e.to_string())?;
                        chart
                            .draw_series([
                                PathElement::new(vec![(x, s.min), (x, s.max)], color),
                                PathElement::new(
                                    vec![(x - 0.25, s.median), (x + 0.25, s.median)],
                                    BLACK,
                                ),
                                PathElement::new(vec![(x - 0.15, s.min), (x + 0.15, s.min)], color),
                                PathElement::new(vec![(x - 0.15, s.max), (x + 0.15, s.max)], color),
                            ])
                            .map_err(|e| e.to_string())?;
                    }
                    6 => {
                        chart
                            .draw_series([
                                PathElement::new(vec![(x, s.low), (x, s.high)], color),
                                PathElement::new(vec![(x - 0.2, s.low), (x + 0.2, s.low)], color),
                                PathElement::new(vec![(x - 0.2, s.high), (x + 0.2, s.high)], color),
                            ])
                            .map_err(|e| e.to_string())?;
                        chart
                            .draw_series(std::iter::once(Circle::new(
                                (x, s.mean),
                                5,
                                color.filled(),
                            )))
                            .map_err(|e| e.to_string())?;
                    }
                    7 => {
                        let lower: Vec<f64> = (0..n)
                            .map(|i| data.series[..group].iter().map(|s| s.values[i]).sum())
                            .collect();
                        let mut points: Vec<_> = series
                            .values
                            .iter()
                            .enumerate()
                            .map(|(i, y)| (i as f64, lower[i] + *y))
                            .collect();
                        points.extend(lower.iter().enumerate().rev().map(|(i, y)| (i as f64, *y)));
                        chart
                            .draw_series(std::iter::once(Polygon::new(
                                points,
                                color.mix(0.65).filled(),
                            )))
                            .map_err(|e| e.to_string())?
                            .label(&series.name)
                            .legend(move |(x, y)| {
                                Rectangle::new([(x, y - 4), (x + 20, y + 4)], color.filled())
                            });
                    }
                    _ => return Err("Unsupported chart family".to_owned()),
                }
            }
            if matches!(figure.kind, 0 | 1 | 7) {
                chart
                    .configure_series_labels()
                    .background_style(WHITE.mix(0.85))
                    .border_style(BLACK)
                    .draw()
                    .map_err(|e| e.to_string())?;
            }
        }
        root.present().map_err(|e| e.to_string())?;
    }
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::papers::{PaperId, experiment, figure, summarize};

    #[test]
    fn every_family_renders_svg_without_native_fonts() {
        let id = PaperId {
            category: 0,
            author: 0,
            topic: 0,
            nonce: 1,
        };
        for kind in 0..NAMES.len() {
            let mut figure = figure(&id, 0);
            figure.kind = kind;
            figure.name = NAMES[kind].to_owned();
            let data = experiment(&id, 0);
            let svg = render(&data, &figure).unwrap();
            assert!(svg.contains("<svg") && svg.contains("</svg>"));
            assert!(!svg.contains("NaN") && !svg.contains("<script"));
            assert_eq!(svg, render(&data, &figure).unwrap());
            let mut escaped = figure.clone();
            escaped.name = "<script>alert('x')</script> & labels".to_owned();
            let escaped_svg = render(&data, &escaped).unwrap();
            assert!(!escaped_svg.contains("<script>"));
            assert!(escaped_svg.contains("&lt;script&gt;"));
            let mut constant = data.clone();
            for series in &mut constant.series {
                series.values.fill(3.0);
                series.summary = summarize(&series.values);
            }
            assert!(render(&constant, &figure).is_ok());
        }
    }
}
