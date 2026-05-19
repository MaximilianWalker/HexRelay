use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> MetricLabel<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricSample<'a> {
    pub name: &'a str,
    pub help: &'a str,
    pub labels: Vec<MetricLabel<'a>>,
    pub value: u64,
}

impl<'a> MetricSample<'a> {
    pub fn counter(name: &'a str, help: &'a str, labels: Vec<MetricLabel<'a>>, value: u64) -> Self {
        Self {
            name,
            help,
            labels,
            value,
        }
    }
}

pub fn render_prometheus_counters(samples: &[MetricSample<'_>]) -> String {
    let mut output = String::new();
    let mut emitted_metadata = BTreeSet::new();

    for sample in samples {
        if emitted_metadata.insert(sample.name) {
            output.push_str("# HELP ");
            output.push_str(sample.name);
            output.push(' ');
            output.push_str(&escape_help(sample.help));
            output.push('\n');
            output.push_str("# TYPE ");
            output.push_str(sample.name);
            output.push_str(" counter\n");
        }

        output.push_str(sample.name);
        if !sample.labels.is_empty() {
            output.push('{');
            for (index, label) in sample.labels.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(label.name);
                output.push_str("=\"");
                output.push_str(&escape_label_value(label.value));
                output.push('"');
            }
            output.push('}');
        }
        output.push(' ');
        output.push_str(&sample.value.to_string());
        output.push('\n');
    }

    output
}

fn escape_help(value: &str) -> String {
    value.replace('\\', r"\\").replace('\n', r"\n")
}

fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_prometheus_counter_text_with_escaped_labels() {
        let rendered = render_prometheus_counters(&[
            MetricSample::counter(
                "hexrelay_test_total",
                "Test counter",
                vec![MetricLabel::new("outcome", "accepted")],
                2,
            ),
            MetricSample::counter(
                "hexrelay_test_total",
                "Test counter",
                vec![MetricLabel::new("outcome", "quoted\"value")],
                1,
            ),
        ]);

        assert_eq!(
            rendered,
            "# HELP hexrelay_test_total Test counter\n\
             # TYPE hexrelay_test_total counter\n\
             hexrelay_test_total{outcome=\"accepted\"} 2\n\
             hexrelay_test_total{outcome=\"quoted\\\"value\"} 1\n"
        );
    }
}
