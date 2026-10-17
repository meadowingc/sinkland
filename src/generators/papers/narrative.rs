use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lens {
    Location,
    Dispersion,
    Quartiles,
    Overlap,
    Windows,
    Ranking,
    Trimming,
    Extent,
    Intervals,
    Occupancy,
    Crossings,
    Balance,
    QuantileRanks,
    PairOrder,
    Roughness,
    Detrending,
    ReferenceBand,
    PointInfluence,
}

impl Lens {
    const ALL: [Self; 18] = [
        Self::Location,
        Self::Dispersion,
        Self::Quartiles,
        Self::Overlap,
        Self::Windows,
        Self::Ranking,
        Self::Trimming,
        Self::Extent,
        Self::Intervals,
        Self::Occupancy,
        Self::Crossings,
        Self::Balance,
        Self::QuantileRanks,
        Self::PairOrder,
        Self::Roughness,
        Self::Detrending,
        Self::ReferenceBand,
        Self::PointInfluence,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Location => "Separation at the center",
            Self::Dispersion => "Level versus dispersion",
            Self::Quartiles => "The shape of the middle",
            Self::Overlap => "Where distributions meet",
            Self::Windows => "Dependence on the observation window",
            Self::Ranking => "Changing the ranking criterion",
            Self::Trimming => "Sensitivity to endpoint removal",
            Self::Extent => "Extremes and typical variation",
            Self::Intervals => "Reading the mean intervals",
            Self::Occupancy => "Concentration around a local center",
            Self::Crossings => "Order along the sequence",
            Self::Balance => "The geometry of the comparison",
            Self::QuantileRanks => "Ordering at different quantiles",
            Self::PairOrder => "Comparing all response pairs",
            Self::Roughness => "Local roughness versus total spread",
            Self::Detrending => "What remains after a linear fit",
            Self::ReferenceBand => "Reciprocal central-band occupancy",
            Self::PointInfluence => "Influence of a single observation",
        }
    }

    fn is_additional(self) -> bool {
        matches!(
            self,
            Self::QuantileRanks
                | Self::PairOrder
                | Self::Roughness
                | Self::Detrending
                | Self::ReferenceBand
                | Self::PointInfluence
        )
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn ranked(data: &Experiment, value: impl Fn(&Series) -> f64) -> Vec<usize> {
    let mut order = (0..data.series.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| value(&data.series[b]).total_cmp(&value(&data.series[a])));
    order
}

fn halves(series: &Series) -> (f64, f64) {
    let split = series.values.len() / 2;
    (mean(&series.values[..split]), mean(&series.values[split..]))
}

fn trimmed(series: &Series) -> f64 {
    (series.values.iter().sum::<f64>() - series.summary.min - series.summary.max)
        / (series.values.len() - 2) as f64
}

fn direction(delta: f64) -> &'static str {
    if delta > 0.0 {
        "higher"
    } else if delta < 0.0 {
        "lower"
    } else {
        "equal"
    }
}

fn relative_position(delta: f64) -> &'static str {
    if delta == 0.0 {
        "equal to"
    } else if delta > 0.0 {
        "higher than"
    } else {
        "lower than"
    }
}

fn pair_reversals(data: &Experiment, score: impl Fn(&Series) -> f64) -> usize {
    let mut reversals = 0;
    for (i, a) in data.series.iter().enumerate() {
        for b in &data.series[i + 1..] {
            reversals +=
                usize::from((a.summary.mean - b.summary.mean) * (score(a) - score(b)) < 0.0);
        }
    }
    reversals
}

fn result_paragraphs(data: &Experiment, lens: Lens, voice: usize) -> Vec<String> {
    if lens.is_additional() {
        return diagnostic_paragraphs(data, lens, voice);
    }
    let order = ranked(data, |s| s.summary.mean);
    let top = &data.series[order[0]];
    let next = &data.series[order[1]];
    let bottom = &data.series[*order.last().unwrap()];
    let unit = &data.unit;
    let n = top.summary.n;
    let (fact, reading) = match lens {
        Lens::Location => {
            let gap = top.summary.mean - next.summary.mean;
            let span = top.summary.mean - bottom.summary.mean;
            let fact = match voice {
                0 => format!(
                    "The largest mean belongs to {}: {:.2} {unit}. The nearest competitor by mean, {}, records {:.2}, leaving a gap of {gap:.2}. At the other end, {} averages {:.2}; the distance between the two extreme means is {span:.2}. These two distances answer different questions about the ordering.",
                    top.name,
                    top.summary.mean,
                    next.name,
                    next.summary.mean,
                    bottom.name,
                    bottom.summary.mean
                ),
                1 => format!(
                    "Sorting the configuration means places {} first and {} second, at {:.2} and {:.2} {unit}. Their separation is {gap:.2}, compared with {span:.2} across the complete set. {} supplies the lowest mean ({:.2}), not a zero-response baseline.",
                    top.name,
                    next.name,
                    top.summary.mean,
                    next.summary.mean,
                    bottom.name,
                    bottom.summary.mean
                ),
                2 => format!(
                    "A comparison with the lowest mean would put {} {:.2} {unit} above {}. That is not its margin over the next-ranked configuration: {} lies only {gap:.2} away. The corresponding means are {:.2} for the leader, {:.2} for the runner-up, and {:.2} for the lowest entry.",
                    top.name,
                    span,
                    bottom.name,
                    next.name,
                    top.summary.mean,
                    next.summary.mean,
                    bottom.summary.mean
                ),
                _ => format!(
                    "The mean response occupies a {span:.2}-{unit} span, bounded by {} ({:.2}) and {} ({:.2}). Within this span, {} sits immediately below the largest mean at {:.2}. Thus the top-two contrast, {gap:.2}, uses a different reference point from the full-range contrast.",
                    bottom.name,
                    bottom.summary.mean,
                    top.name,
                    top.summary.mean,
                    next.name,
                    next.summary.mean
                ),
            };
            let reading = if gap < span / 3.0 {
                format!(
                    "Less than a third of the between-mean span separates the first two entries. Describing the leader solely against {} would hide that proximity.",
                    bottom.name
                )
            } else {
                format!(
                    "At least a third of the between-mean span falls between the first two entries. The ordering is therefore not explained only by an isolated low mean for {}.",
                    bottom.name
                )
            };
            (fact, reading)
        }
        Lens::Dispersion => {
            let sd_order = ranked(data, |s| s.summary.sd);
            let wide = &data.series[sd_order[0]];
            let narrow = &data.series[*sd_order.last().unwrap()];
            let fact = match voice {
                0 => format!(
                    "Dispersion changes the comparison: {} has the largest sample SD ({:.2} {unit}), while {} has the smallest ({:.2}). Their means are {:.2} and {:.2}. The mean leader, {}, has SD {:.2}; a location ranking therefore omits a measurable part of the response profile.",
                    wide.name,
                    wide.summary.sd,
                    narrow.name,
                    narrow.summary.sd,
                    wide.summary.mean,
                    narrow.summary.mean,
                    top.name,
                    top.summary.sd
                ),
                1 => format!(
                    "The tightest sequence by sample SD is {}, with {:.2} {unit}, rather than an unspecified 'stable' baseline. {} reaches SD {:.2}. For context, the highest mean is {:.2} for {}, whose own SD is {:.2}. A small spread and a large center are separate criteria.",
                    narrow.name,
                    narrow.summary.sd,
                    wide.name,
                    wide.summary.sd,
                    top.summary.mean,
                    top.name,
                    top.summary.sd
                ),
                2 => format!(
                    "Looking away from the means, the sample SD spans {:.2}–{:.2} {unit}. The endpoints belong to {} and {}, respectively. Their central responses, {:.2} and {:.2}, show what a spread-only summary would leave out; {} still has the highest mean at {:.2}.",
                    narrow.summary.sd,
                    wide.summary.sd,
                    narrow.name,
                    wide.name,
                    narrow.summary.mean,
                    wide.summary.mean,
                    top.name,
                    top.summary.mean
                ),
                _ => format!(
                    "{} combines a mean of {:.2} with an SD of {:.2} {unit}. It is the mean leader, but the spread comparison runs from {} (SD {:.2}) to {} (SD {:.2}). Both rankings use all {n} recorded values per configuration, not selected portions of a curve.",
                    top.name,
                    top.summary.mean,
                    top.summary.sd,
                    narrow.name,
                    narrow.summary.sd,
                    wide.name,
                    wide.summary.sd
                ),
            };
            let reading = if order[0] == *sd_order.last().unwrap() {
                "Here the largest mean and the smallest SD occur in the same configuration. There is no center–spread conflict between those two criteria in this window."
            } else if order[0] == sd_order[0] {
                "The configuration with the highest mean also has the widest SD. A reader valuing concentration would not reach the same conclusion as one sorting by response level."
            } else {
                "The mean leader occupies neither end of the SD ordering. Response level and concentration consequently cannot be summarized by naming one extreme configuration."
            };
            (fact, reading.to_owned())
        }
        Lens::Quartiles => {
            let selected = ranked(data, |s| (s.summary.mean - s.summary.median).abs())[0];
            let series = &data.series[selected];
            let s = &series.summary;
            let left = s.median - s.q1;
            let right = s.q3 - s.median;
            let fact = match voice {
                0 => format!(
                    "{} has the largest absolute mean–median displacement in this setting. Its mean is {:.2} and its median {:.2} {unit}; the central half extends from {:.2} to {:.2}. The lower half of that interval is {left:.2} wide and the upper half {right:.2}, permitting a shape check independent of the mean ranking.",
                    series.name, s.mean, s.median, s.q1, s.q3
                ),
                1 => format!(
                    "The quartiles make {} worth inspecting: Q1 = {:.2}, median = {:.2}, Q3 = {:.2} {unit}. Its mean, {:.2}, is the furthest from its own median among the configurations. Distances on either side of the median are {left:.2} below and {right:.2} above; equal halves should not be assumed from a box's total width.",
                    series.name, s.q1, s.median, s.q3, s.mean
                ),
                2 => format!(
                    "Replacing an average by a median matters most, in absolute terms, for {}. The two summaries are {:.2} and {:.2} {unit}, respectively. Its interquartile endpoints, {:.2} and {:.2}, split into widths {left:.2} and {right:.2} around the median rather than prescribing a symmetric distribution.",
                    series.name, s.mean, s.median, s.q1, s.q3
                ),
                _ => format!(
                    "For {}, the central 50% interval is {:.2}–{:.2} {unit}, with a median of {:.2}. The mean is {:.2}. This is the configuration where mean and median disagree most in absolute distance; the unequal roles of tails and central observations are therefore directly visible in the tabulated summaries.",
                    series.name, s.q1, s.q3, s.median, s.mean
                ),
            };
            let reading = format!(
                "The mean is {} the median; {}.",
                relative_position(s.mean - s.median),
                if right > left {
                    "the upper half of the interquartile interval is longer"
                } else if right < left {
                    "the lower half of the interquartile interval is longer"
                } else {
                    "the two halves of the interquartile interval have equal width"
                }
            );
            (fact, reading)
        }
        Lens::Overlap => {
            let pair = order
                .windows(2)
                .min_by(|a, b| {
                    let gap = |p: &[usize]| {
                        data.series[p[0]].summary.mean - data.series[p[1]].summary.mean
                    };
                    gap(a).total_cmp(&gap(b))
                })
                .unwrap();
            let a = &data.series[pair[0]];
            let b = &data.series[pair[1]];
            let low = a.summary.q1.max(b.summary.q1);
            let high = a.summary.q3.min(b.summary.q3);
            let fact = match voice {
                0 => format!(
                    "The closest neighboring means are {:.2} for {} and {:.2} for {} {unit}. Their interquartile intervals are {:.2}–{:.2} and {:.2}–{:.2}. Choosing this pair by proximity, rather than contrasting the most separated means, asks whether the central portions can be distinguished by their observed extent.",
                    a.summary.mean,
                    a.name,
                    b.summary.mean,
                    b.name,
                    a.summary.q1,
                    a.summary.q3,
                    b.summary.q1,
                    b.summary.q3
                ),
                1 => format!(
                    "{} and {} form the tightest pair in the mean ordering: their gap is {:.2} {unit}. The respective central-half intervals, {:.2}–{:.2} and {:.2}–{:.2}, provide a different comparison from those means. Interval intersection describes where those summaries occupy common response values, not matched observations.",
                    a.name,
                    b.name,
                    a.summary.mean - b.summary.mean,
                    a.summary.q1,
                    a.summary.q3,
                    b.summary.q1,
                    b.summary.q3
                ),
                2 => format!(
                    "Rather than start with the extremes, consider {} against {}. No other adjacent mean pair is closer. They average {:.2} and {:.2} {unit}, with Q1/Q3 endpoints {:.2}/{:.2} and {:.2}/{:.2}. A small mean gap alone does not say whether their central halves intersect.",
                    a.name,
                    b.name,
                    a.summary.mean,
                    b.summary.mean,
                    a.summary.q1,
                    a.summary.q3,
                    b.summary.q1,
                    b.summary.q3
                ),
                _ => format!(
                    "A local ambiguity in the ranking is captured by {} versus {}: {:.2} {unit} separates their means, the smallest adjacent gap. For the same pair the middle-half ranges are {:.2}–{:.2} and {:.2}–{:.2}. The comparison retains distributional extent instead of converting a close ordering into a winner.",
                    a.name,
                    b.name,
                    a.summary.mean - b.summary.mean,
                    a.summary.q1,
                    a.summary.q3,
                    b.summary.q1,
                    b.summary.q3
                ),
            };
            let mut reading = if high >= low {
                format!(
                    "The two interquartile intervals intersect over {:.2} {unit}, from {low:.2} to {high:.2}.",
                    high - low
                )
            } else {
                format!(
                    "Their interquartile intervals are disjoint, with {:.2} {unit} between the nearer endpoints.",
                    low - high
                )
            };
            let range_overlap = a.summary.max.min(b.summary.max) - a.summary.min.max(b.summary.min);
            if range_overlap >= 0.0 {
                reading.push_str(&format!(" Their full observed ranges intersect across {range_overlap:.2} {unit}, so the endpoints of these samples do not separate completely."));
            } else {
                reading.push_str(&format!(" Even their full observed ranges are separated by {:.2} {unit}: none of the recorded responses crosses that gap.", -range_overlap));
            }
            (fact, reading)
        }
        Lens::Windows => {
            let index = ranked(data, |s| {
                let (early, late) = halves(s);
                (late - early).abs()
            })[0];
            let series = &data.series[index];
            let (early, late) = halves(series);
            let split = n / 2;
            let delta = late - early;
            let fact = match voice {
                0 => format!(
                    "Splitting the recorded order after observation {split} produces the largest absolute half-window change for {}. Its first {split} values average {early:.2} {unit}; the remaining {} average {late:.2}. The later-minus-earlier difference is {delta:.2}, whereas its all-window mean is {:.2}.",
                    series.name,
                    n - split,
                    series.summary.mean
                ),
                1 => format!(
                    "A window check singles out {} by block displacement rather than response level. It changes from {early:.2} {unit} in the first {split} observations to {late:.2} in the last {}. Among the named alternatives this is the largest absolute change between half-window means; the signed difference is {delta:.2}.",
                    series.name,
                    n - split
                ),
                2 => format!(
                    "The strongest half-to-half displacement belongs to {}. With the split fixed at index {split}, the early and late means are {early:.2} and {late:.2} {unit}, giving {delta:.2} on subtraction. The two blocks contain {split} and {} observations, so the full mean ({:.2}) weights them by their actual sizes.",
                    series.name,
                    n - split,
                    series.summary.mean
                ),
                _ => format!(
                    "An average over all {n} steps hides the most window-sensitive sequence: {}. Its first-block mean is {early:.2}, its second-block mean {late:.2} {unit}, and its overall mean {:.2}. Blocks contain {split} and {} values. The difference {delta:.2} has the greatest absolute magnitude among these half-window contrasts.",
                    series.name,
                    series.summary.mean,
                    n - split
                ),
            };
            let increases = series
                .values
                .windows(2)
                .filter(|pair| pair[1] > pair[0])
                .count();
            let decreases = series
                .values
                .windows(2)
                .filter(|pair| pair[1] < pair[0])
                .count();
            let reading = format!(
                "The later block is {} on average. Within the trace, {increases} adjacent steps rise and {decreases} fall. {}",
                direction(delta),
                if increases > 0 && decreases > 0 {
                    "Thus the block displacement coexists with movement in both directions, not a monotonic trajectory."
                } else if increases == 0 && decreases == 0 {
                    "All adjacent values are tied."
                } else {
                    "The observed steps are monotonic in this particular window; no breakpoint is inferred."
                }
            );
            (fact, reading)
        }
        Lens::Ranking => {
            let medians = ranked(data, |s| s.summary.median);
            let median_top = &data.series[medians[0]];
            let reversals = pair_reversals(data, |s| s.summary.median);
            let pairs = data.series.len() * (data.series.len() - 1) / 2;
            let fact = match voice {
                0 => format!(
                    "Ranking by mean puts {} at the top ({:.2} {unit}); ranking by median puts {} first ({:.2}). Across all {pairs} unordered configuration pairs, {reversals} reverse their strict order when the criterion changes. Ties are not counted as reversals, and both rankings are calculated within this setting.",
                    top.name, top.summary.mean, median_top.name, median_top.summary.median
                ),
                1 => format!(
                    "Does the ordering depend on using an average? The mean maximum is {:.2} {unit} for {}, while the median maximum is {:.2} for {}. Comparing every pair yields {reversals} mean-to-median reversals out of {pairs} possible pairs; this checks the full ordering rather than only the top label.",
                    top.summary.mean, top.name, median_top.summary.median, median_top.name
                ),
                2 => format!(
                    "Mean and median do not summarize the same feature. Here their leaders are {} (mean {:.2} {unit}) and {} (median {:.2}). There are {reversals} strict pairwise reversals among {pairs} pairs when observations are summarized by median instead of mean; equal scores do not create a reversal.",
                    top.name, top.summary.mean, median_top.name, median_top.summary.median
                ),
                _ => format!(
                    "The ordering audit compares {pairs} pairs using two location criteria. It finds {reversals} reversals. {} has the greatest mean, {:.2} {unit}, and {} the greatest median, {:.2}. The audit does not reorder the table rows, whose configuration identities remain fixed for comparison with the other settings.",
                    top.name, top.summary.mean, median_top.name, median_top.summary.median
                ),
            };
            let reading = match (order[0] == medians[0], reversals == 0) {
                (true, true) => {
                    "No strict pair reverses, and the leading label is retained. Agreement extends below the top of the ordering."
                }
                (true, false) => {
                    "The top label survives the change of criterion, but the rest of the ordering is not fully preserved. Reporting only the leader would conceal this sensitivity."
                }
                _ => {
                    "The leading configuration changes when the median replaces the mean. The criterion affects the top of the ordering, not just the relative positions of lower-ranked entries."
                }
            };
            (fact, reading.to_owned())
        }
        Lens::Trimming => {
            let index = ranked(data, |s| (trimmed(s) - s.summary.mean).abs())[0];
            let series = &data.series[index];
            let reduced = trimmed(series);
            let delta = reduced - series.summary.mean;
            let trim_order = ranked(data, trimmed);
            let changed = pair_reversals(data, trimmed);
            let fact = match voice {
                0 => format!(
                    "Removing exactly one smallest and one largest observation affects {} most in absolute mean change. Its original mean is {:.2} {unit}; the remaining {} observations average {reduced:.2}, a signed shift of {delta:.2}. This is an endpoint-removal diagnostic, not a fitted robust estimator or a replacement dataset.",
                    series.name,
                    series.summary.mean,
                    n - 2
                ),
                1 => format!(
                    "The endpoint check retains {} of {n} observations in each sequence. {} shows the largest absolute effect: its mean changes from {:.2} to {reduced:.2} {unit} after one minimum and one maximum are removed. The difference {delta:.2} isolates this particular deletion rule, without assuming that those values were errors.",
                    n - 2,
                    series.name,
                    series.summary.mean
                ),
                2 => format!(
                    "How much do the two extreme observations move an average? For {}, removing one value at each extreme leaves a mean of {reduced:.2} {unit}, compared with {:.2} before removal. The shift is {delta:.2}, the largest absolute deletion response in the setting; {} values remain in that calculation.",
                    series.name,
                    series.summary.mean,
                    n - 2
                ),
                _ => format!(
                    "{} is most sensitive to symmetric endpoint deletion. Using all {n} observations gives {:.2} {unit}; discarding just one minimum and one maximum gives {reduced:.2} over {} values. Calling the resulting change ({delta:.2}) an outlier correction would be unwarranted, since no contamination mechanism has been identified.",
                    series.name,
                    series.summary.mean,
                    n - 2
                ),
            };
            let reading = format!(
                "The mean leader {} after the deletion rule is applied to every configuration; {changed} strict pairwise orders reverse. The figures and tables retain the untrimmed observations.",
                if order[0] == trim_order[0] {
                    format!("remains {}", top.name)
                } else {
                    format!(
                        "changes from {} to {}",
                        top.name, data.series[trim_order[0]].name
                    )
                }
            );
            (fact, reading)
        }
        Lens::Extent => {
            let index = ranked(data, |s| s.summary.max - s.summary.min)[0];
            let series = &data.series[index];
            let s = &series.summary;
            let range = s.max - s.min;
            let iqr = s.q3 - s.q1;
            let low_tail = s.q1 - s.min;
            let high_tail = s.max - s.q3;
            let fact = match voice {
                0 => format!(
                    "The widest observed range belongs to {}: {:.2}–{:.2} {unit}, a span of {range:.2}. Its central-half width is {iqr:.2}. Below Q1 the remaining extent is {low_tail:.2}, and above Q3 it is {high_tail:.2}; the full span thus contains substantial information not represented by a single SD.",
                    series.name, s.min, s.max
                ),
                1 => format!(
                    "{} covers more response values by range than any other configuration, from {:.2} to {:.2} {unit}. Of the total width {range:.2}, {iqr:.2} lies between its quartiles. The lower and upper outer stretches occupy {low_tail:.2} and {high_tail:.2}, respectively; these lengths are not counts of tail events.",
                    series.name, s.min, s.max
                ),
                2 => format!(
                    "An extent-based reading selects {}. Its minimum ({:.2}) and maximum ({:.2} {unit}) are {range:.2} apart, while Q1 and Q3 are only {iqr:.2} apart. The two outer portions have widths {low_tail:.2} and {high_tail:.2}. These quantities distinguish a broad middle from long distances to endpoints.",
                    series.name, s.min, s.max
                ),
                _ => format!(
                    "The endpoints for {} bracket the largest within-configuration span: {range:.2} {unit}. The actual bounds are {:.2} and {:.2}, with central-half width {iqr:.2}. Its lower outer extent is {low_tail:.2}, compared with {high_tail:.2} above the upper quartile, making the geometry of this range explicit.",
                    series.name, s.min, s.max
                ),
            };
            let reading = if iqr > range / 2.0 {
                "More than half of the observed range lies inside the interquartile interval. The wide extent cannot be attributed solely to distances beyond the middle half."
            } else {
                "At least half of the observed range lies outside the interquartile interval. A range-only description would therefore emphasize the outer stretches more than the middle half."
            };
            (fact, reading.to_owned())
        }
        Lens::Intervals => {
            let a = &top.summary;
            let b = &next.summary;
            let separation = a.low - b.high;
            let fact = match voice {
                0 => format!(
                    "The two largest means have illustrative intervals {:.2}–{:.2} for {} and {:.2}–{:.2} for {} {unit}. Their centers are {:.2} and {:.2}. Using the stated formula gives half-widths {:.2} and {:.2}; these widths should not be confused with the spread of individual observations.",
                    a.low,
                    a.high,
                    top.name,
                    b.low,
                    b.high,
                    next.name,
                    a.mean,
                    b.mean,
                    (a.high - a.low) / 2.0,
                    (b.high - b.low) / 2.0
                ),
                1 => format!(
                    "For {}, mean {:.2} {unit} is bracketed by {:.2} and {:.2}. The next mean, {:.2} for {}, has endpoints {:.2} and {:.2}. Both calculations divide the sample SD by √{n} before scaling by 1.96; a narrow interval can therefore coexist with considerable observation-level variation.",
                    top.name, a.mean, a.low, a.high, b.mean, next.name, b.low, b.high
                ),
                2 => format!(
                    "Interval geometry offers a check on the top-two comparison. {} has endpoints {:.2} and {:.2} {unit}; {} has {:.2} and {:.2}. The respective means, {:.2} and {:.2}, use {n} observations each. These are the intervals in the distribution table, not newly fitted uncertainty bands.",
                    top.name, a.low, a.high, next.name, b.low, b.high, a.mean, b.mean
                ),
                _ => format!(
                    "The mean intervals around the two leading averages have half-widths {:.2} for {} and {:.2} for {} {unit}. In endpoint form they are {:.2}–{:.2} and {:.2}–{:.2}. Reading only the centers ({:.2}, {:.2}) would omit this arithmetic summary of SD and sample count.",
                    (a.high - a.low) / 2.0,
                    top.name,
                    (b.high - b.low) / 2.0,
                    next.name,
                    a.low,
                    a.high,
                    b.low,
                    b.high,
                    a.mean,
                    b.mean
                ),
            };
            let reading = if separation > 0.0 {
                format!(
                    "The leader's lower endpoint sits {separation:.2} {unit} above the runner-up's upper endpoint. These illustrative bounds remain subject to the dependence limitation in the analysis protocol."
                )
            } else {
                format!(
                    "The two intervals intersect: the leader's lower endpoint is {:.2} {unit} below or equal to the runner-up's upper endpoint. Overlap does not establish equality; these are illustrative bounds.",
                    -separation
                )
            };
            (fact, reading)
        }
        Lens::Occupancy => {
            let selected = &data.series[0];
            let s = &selected.summary;
            let low = s.mean - s.sd;
            let high = s.mean + s.sd;
            let inside = selected
                .values
                .iter()
                .filter(|&&v| v >= low && v <= high)
                .count();
            let below = selected.values.iter().filter(|&&v| v < low).count();
            let above = n - inside - below;
            let fact = match voice {
                0 => format!(
                    "Using the first listed configuration, {}, avoids selecting a favorable concentration after inspection. Its mean ± one sample SD runs from {low:.2} to {high:.2} {unit}. Exactly {inside} of {n} observations lie inside these inclusive bounds; {below} fall below and {above} above.",
                    selected.name
                ),
                1 => format!(
                    "A direct count checks how much of {} is close to its own average. The one-SD neighborhood is {low:.2}–{high:.2} {unit}: {inside} values are inside, {below} are lower, and {above} are higher, totaling {n}. The configuration was chosen by its fixed row position, not by the count obtained.",
                    selected.name
                ),
                2 => format!(
                    "For the first table row ({}), the band between mean minus SD and mean plus SD contains {inside}/{n} recorded values. Its limits are {low:.2} and {high:.2} {unit}. Outside the band, {below} observations occupy the lower side and {above} the upper, distinguishing concentration from an average alone.",
                    selected.name
                ),
                _ => format!(
                    "The local-center check keeps {} as a fixed reference. Of its {n} observations, {inside} are within one sample SD of the sample mean, including the endpoints {low:.2} and {high:.2} {unit}. The remaining counts split into {below} below the band and {above} above it.",
                    selected.name
                ),
            };
            let other = &data.series[1];
            let other_inside = other
                .values
                .iter()
                .filter(|&&v| (v - other.summary.mean).abs() <= other.summary.sd)
                .count();
            let reading = format!(
                "The observed fraction inside is {:.1}%. For {}, a band around its own mean with its own SD contains {other_inside}/{n} values ({:.1}%). {} has the {} observed fraction; neither band is a future-coverage interval.",
                100.0 * inside as f64 / n as f64,
                other.name,
                100.0 * other_inside as f64 / n as f64,
                selected.name,
                if inside > other_inside {
                    "larger"
                } else if inside < other_inside {
                    "smaller"
                } else {
                    "same"
                }
            );
            (fact, reading)
        }
        Lens::Crossings => {
            let a = &data.series[0];
            let b = &data.series[1];
            let signs = a
                .values
                .iter()
                .zip(&b.values)
                .map(|(a, b)| (a - b).total_cmp(&0.0))
                .collect::<Vec<_>>();
            let above = a
                .values
                .iter()
                .zip(&b.values)
                .filter(|(a, b)| a > b)
                .count();
            let ties = a
                .values
                .iter()
                .zip(&b.values)
                .filter(|(a, b)| a == b)
                .count();
            let switches = signs
                .windows(2)
                .filter(|pair| {
                    use std::cmp::Ordering::{Greater, Less};
                    matches!((pair[0], pair[1]), (Greater, Less) | (Less, Greater))
                })
                .count();
            let fact = match voice {
                0 => format!(
                    "Keeping the first two table labels as a fixed pair, {} exceeds {} at {above} of {n} aligned indices, with {ties} ties. Adjacent indices switch strict order {switches} times. Their means are {:.2} and {:.2} {unit}, so the sequence-wise comparison asks more than which mean is larger.",
                    a.name, b.name, a.summary.mean, b.summary.mean
                ),
                1 => format!(
                    "An index-by-index comparison of {} with {} counts {above} higher responses for the first label among {n} positions. There are {ties} exact ties and {switches} strict sign switches between neighboring positions. This is a property of the recorded ordering, alongside means {:.2} and {:.2} {unit}.",
                    a.name, b.name, a.summary.mean, b.summary.mean
                ),
                2 => format!(
                    "The first two traces are not reduced to their averages in this check. {} is above {} at {above}/{n} positions; {ties} positions tie. Moving through consecutive indices gives {switches} strict crossings. Their full-window means remain {:.2} and {:.2} {unit}, calculated independently of that crossing count.",
                    a.name, b.name, a.summary.mean, b.summary.mean
                ),
                _ => format!(
                    "How persistent is the order of {} and {} along the displayed sequence? The first is higher at {above} indices out of {n}, and the pair ties {ties} times. A change from strictly above to strictly below, or the reverse, occurs across {switches} adjacent steps; the means are {:.2} and {:.2} {unit}.",
                    a.name, b.name, a.summary.mean, b.summary.mean
                ),
            };
            let reading = if switches == 0 {
                "No adjacent strict crossing is observed; a transition through a tie is excluded by this definition."
            } else {
                "The traces exchange strict order within the window. The mean ordering therefore does not describe every local comparison along the sequence."
            };
            (fact, reading.to_owned())
        }
        Lens::Balance => {
            let center =
                data.series.iter().map(|s| s.summary.mean).sum::<f64>() / data.series.len() as f64;
            let above = data
                .series
                .iter()
                .filter(|s| s.summary.mean > center)
                .count();
            let upper = top.summary.mean - center;
            let lower = center - bottom.summary.mean;
            let fact = match voice {
                0 => format!(
                    "The equally weighted average of configuration means is {center:.2} {unit}. The count strictly above this center is {above} out of {} configurations. {} is {upper:.2} above this center, whereas {} is {lower:.2} below. Equal observation counts within the setting make this center equal to the pooled observation mean here.",
                    data.series.len(),
                    top.name,
                    bottom.name
                ),
                1 => format!(
                    "Centering the configuration means on their own average gives {center:.2} {unit}. The largest upward displacement, {upper:.2}, belongs to {}; the largest downward displacement, {lower:.2}, belongs to {}. The count above the center is {above} among {} entries, revealing how the set is balanced around that summary.",
                    top.name,
                    bottom.name,
                    data.series.len()
                ),
                2 => format!(
                    "Instead of choosing a baseline label, give all {} configuration means equal weight. Their center is {center:.2} {unit}; the above-center count is {above}. The extreme offsets are +{upper:.2} for {} and −{lower:.2} for {}. Because every row contains {n} observations, weighting by counts would yield the same center within this setting.",
                    data.series.len(),
                    top.name,
                    bottom.name
                ),
                _ => format!(
                    "The comparison's internal reference is {center:.2} {unit}, the average of its {} means. The mean leader, {}, sits {upper:.2} above it, and {}, the lowest entry, {lower:.2} below it. The strict above-reference count is {above}. This is a center of the reported alternatives, not a reference-population estimate.",
                    data.series.len(),
                    top.name,
                    bottom.name
                ),
            };
            let reading = if upper > lower {
                "The high side extends further from this center than the low side. The configuration means are not arranged symmetrically around their equal-weight center."
            } else {
                "The low side extends at least as far from this center as the high side. This describes the configuration means, not skewness within an individual trace."
            };
            (fact, reading.to_owned())
        }
        _ => unreachable!("additional diagnostics are handled above"),
    };
    let reading = if voice >= 2 {
        companion_check(data, lens, voice)
    } else {
        reading
    };
    vec![fact, reading]
}

fn companion_check(data: &Experiment, lens: Lens, voice: usize) -> String {
    let order = ranked(data, |s| s.summary.mean);
    let top = &data.series[order[0]];
    let bottom = &data.series[*order.last().unwrap()];
    let unit = &data.unit;
    let alternative = voice == 3;
    match lens {
        Lens::Location => {
            let median_gap = top.summary.median - bottom.summary.median;
            if alternative {
                format!(
                    "Keeping the extreme-mean labels fixed but changing the statistic gives a median contrast of {median_gap:.2} {unit}, in the order {} minus {}.",
                    top.name, bottom.name
                )
            } else {
                format!(
                    "For this same high-mean/low-mean pair, the median difference is {median_gap:.2} {unit}. The median of {} is {} that of {}.",
                    top.name,
                    relative_position(median_gap),
                    bottom.name
                )
            }
        }
        Lens::Dispersion => {
            let iqr_order = ranked(data, |s| s.summary.q3 - s.summary.q1);
            let sd_order = ranked(data, |s| s.summary.sd);
            let a = &data.series[iqr_order[0]];
            let agrees = iqr_order[0] == sd_order[0];
            if alternative {
                format!(
                    "Sorting by central-half width instead selects {}, with an IQR of {:.2} {unit}. The widest-SD label {}.",
                    a.name,
                    a.summary.q3 - a.summary.q1,
                    if agrees {
                        "also leads this ordering"
                    } else {
                        "does not lead this ordering"
                    }
                )
            } else {
                format!(
                    "A second spread criterion gives {} the largest IQR ({:.2} {unit}); this {} the SD-based selection.",
                    a.name,
                    a.summary.q3 - a.summary.q1,
                    if agrees {
                        "agrees with"
                    } else {
                        "differs from"
                    }
                )
            }
        }
        Lens::Quartiles => {
            let s = &data.series[ranked(data, |s| (s.summary.mean - s.summary.median).abs())[0]];
            let outside = (s.summary.q1 - s.summary.min) + (s.summary.max - s.summary.q3);
            if alternative {
                format!(
                    "For {}, the distances outside the central interval—from minimum to Q1 and Q3 to maximum—add to {outside:.2} {unit}.",
                    s.name
                )
            } else {
                format!(
                    "The outer stretches for {} together span {outside:.2} {unit}, compared with {:.2} between Q1 and Q3.",
                    s.name,
                    s.summary.q3 - s.summary.q1
                )
            }
        }
        Lens::Overlap => {
            let common_low = data
                .series
                .iter()
                .map(|s| s.summary.min)
                .max_by(f64::total_cmp)
                .unwrap();
            let common_high = data
                .series
                .iter()
                .map(|s| s.summary.max)
                .min_by(f64::total_cmp)
                .unwrap();
            let statement = if common_low <= common_high {
                format!(
                    "All observed ranges have a common intersection from {common_low:.2} to {common_high:.2} {unit}"
                )
            } else {
                format!(
                    "There is no intersection common to all observed ranges: the largest minimum ({common_low:.2}) exceeds the smallest maximum ({common_high:.2} {unit})"
                )
            };
            let low_source = &data.series[ranked(data, |s| s.summary.min)[0]];
            let high_source = &data.series[ranked(data, |s| -s.summary.max)[0]];
            if alternative {
                format!(
                    "{statement}. {} supplies the largest minimum; {} supplies the smallest maximum.",
                    low_source.name, high_source.name
                )
            } else {
                format!(
                    "{statement}. The limiting endpoints come from {} below and {} above.",
                    low_source.name, high_source.name
                )
            }
        }
        Lens::Windows => {
            let early = ranked(data, |s| halves(s).0)[0];
            let late = ranked(data, |s| halves(s).1)[0];
            let statement = if early == late {
                "is retained"
            } else {
                "changes"
            };
            if alternative {
                format!(
                    "The half-window mean leader {statement}: {} leads the early block and {} the later one.",
                    data.series[early].name, data.series[late].name
                )
            } else {
                format!(
                    "Across configurations, {} has the largest early-block mean and {} the largest late-block mean. Thus the leading label {statement} at the fixed split.",
                    data.series[early].name, data.series[late].name
                )
            }
        }
        Lens::Ranking => {
            let reversals = pair_reversals(data, trimmed);
            if alternative {
                format!(
                    "As a different perturbation, removing one minimum and one maximum per sequence yields {reversals} strict reversals relative to the original mean ordering. The median comparison above instead retains every observation."
                )
            } else {
                format!(
                    "Endpoint removal offers an additional check: {reversals} strict pairs reverse after deleting one minimum and one maximum from each sequence and recomputing the means.",
                )
            }
        }
        Lens::Trimming => {
            let s = &data.series[ranked(data, |s| (trimmed(s) - s.summary.mean).abs())[0]];
            let endpoint_center = (s.summary.min + s.summary.max) / 2.0;
            if alternative {
                format!(
                    "For {}, the average of the two removed endpoints is {endpoint_center:.2} {unit}, {} its full mean. This locates the contribution removed by the deletion rule.",
                    s.name,
                    relative_position(endpoint_center - s.summary.mean)
                )
            } else {
                format!(
                    "The removed pair for {} has midpoint {endpoint_center:.2} {unit}, while the original mean is {:.2}. Symmetric deletion in count need not remove equal magnitudes on either side of the mean.",
                    s.name, s.summary.mean
                )
            }
        }
        Lens::Extent => {
            let s = &data.series[ranked(data, |s| s.summary.max - s.summary.min)[0]];
            let count = s
                .values
                .iter()
                .filter(|&&v| v < s.summary.q1 || v > s.summary.q3)
                .count();
            if alternative {
                format!(
                    "There are {count} values strictly outside {}'s interquartile interval out of {} observations. This is an observed count, not the outer distance reported above.",
                    s.name, s.summary.n
                )
            } else {
                format!(
                    "Counting instead of measuring distances places {count}/{} observations strictly outside the interquartile interval for {}. Interpolated quartiles and ties affect that finite-sample count.",
                    s.summary.n, s.name
                )
            }
        }
        Lens::Intervals => {
            let wide = &data.series[ranked(data, |s| s.summary.high - s.summary.low)[0]];
            if alternative {
                format!(
                    "The widest illustrative interval belongs to {}, spanning {:.2} {unit}. Equal n makes interval-width order identical to SD order here.",
                    wide.name,
                    wide.summary.high - wide.summary.low
                )
            } else {
                format!(
                    "For comparison with the leading pair, {} has the greatest illustrative interval width ({:.2} {unit}). Since observation counts match, this also identifies a largest SD.",
                    wide.name,
                    wide.summary.high - wide.summary.low
                )
            }
        }
        Lens::Occupancy => {
            let s = &data.series[0];
            let count = s.values.iter().filter(|&&v| v < s.summary.median).count();
            if alternative {
                format!(
                    "Using the median ({:.2} {unit}) as a different reference, {count} of {} observations fall strictly below it. This counts positions rather than membership in the SD-based band.",
                    s.summary.median, s.summary.n
                )
            } else {
                format!(
                    "A positional check counts {count}/{} values strictly below {}'s median of {:.2} {unit}. Moving an endpoint further away would not change its position relative to the median.",
                    s.summary.n, s.name, s.summary.median
                )
            }
        }
        Lens::Crossings => {
            let a = &data.series[0];
            let b = &data.series[1];
            let first_gap = a.values[0] - b.values[0];
            let last_gap = a.values.last().unwrap() - b.values.last().unwrap();
            if alternative {
                format!(
                    "The same pair starts with a signed gap of {first_gap:.2} {unit} and ends at {last_gap:.2}, using first label minus second. The crossing count above retains the intervening order information."
                )
            } else {
                format!(
                    "At the two endpoints, {} minus {} equals {first_gap:.2} and {last_gap:.2} {unit}. The endpoint signs can be read alongside the intervening switches counted above.",
                    a.name, b.name
                )
            }
        }
        Lens::Balance => {
            let reduced = data
                .series
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != order[0])
                .map(|(_, s)| s.summary.mean)
                .sum::<f64>()
                / (data.series.len() - 1) as f64;
            if alternative {
                format!(
                    "Leaving out the greatest-mean configuration changes the equal-weight center to {reduced:.2} {unit}. This removes a complete configuration, rather than trimming observations within every trace."
                )
            } else {
                format!(
                    "Without {}, the remaining configuration means average {reduced:.2} {unit}. The internal baseline is therefore conditional on which configurations are included.",
                    top.name
                )
            }
        }
        _ => unreachable!("additional diagnostics have their own interpretation"),
    }
}

fn roughness(series: &Series) -> f64 {
    series
        .values
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .sum::<f64>()
        / (series.values.len() - 1) as f64
}

fn response_pairs(a: &Series, b: &Series) -> (usize, usize, usize) {
    let mut sorted = b.values.clone();
    sorted.sort_by(f64::total_cmp);
    let mut greater = 0;
    let mut less = 0;
    for value in &a.values {
        greater += sorted.partition_point(|other| other < value);
        less += sorted.len() - sorted.partition_point(|other| other <= value);
    }
    (
        greater,
        less,
        a.values.len() * b.values.len() - greater - less,
    )
}

fn line_fit(series: &Series) -> (f64, f64, f64) {
    let n = series.values.len();
    let center = (n - 1) as f64 / 2.0;
    let xx = (0..n).map(|i| (i as f64 - center).powi(2)).sum::<f64>();
    let slope = series
        .values
        .iter()
        .enumerate()
        .map(|(i, y)| (i as f64 - center) * (y - series.summary.mean))
        .sum::<f64>()
        / xx;
    let residual_ss = series
        .values
        .iter()
        .enumerate()
        .map(|(i, y)| (y - series.summary.mean - slope * (i as f64 - center)).powi(2))
        .sum::<f64>();
    let total_ss = series
        .values
        .iter()
        .map(|y| (y - series.summary.mean).powi(2))
        .sum::<f64>();
    let explained = if total_ss > 0.0 {
        (1.0 - residual_ss / total_ss).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (slope, (residual_ss / (n - 2) as f64).sqrt(), explained)
}

fn diagnostic_paragraphs(data: &Experiment, lens: Lens, voice: usize) -> Vec<String> {
    let unit = &data.unit;
    let n = data.series[0].values.len();
    let (fact, interpretation) = match lens {
        Lens::QuantileRanks => {
            let lower = ranked(data, |s| s.summary.q1);
            let upper = ranked(data, |s| s.summary.q3);
            let a = &data.series[lower[0]];
            let b = &data.series[upper[0]];
            let mut flips = 0;
            for (i, x) in data.series.iter().enumerate() {
                for y in &data.series[i + 1..] {
                    flips += usize::from(
                        (x.summary.q1 - y.summary.q1) * (x.summary.q3 - y.summary.q3) < 0.0,
                    );
                }
            }
            let fact = match voice {
                0 => format!(
                    "At the lower quartile, {} has the largest response ({:.2} {unit}); at the upper quartile, {} leads ({:.2}). Moving from Q1 to Q3 reverses {flips} strict pair orders.",
                    a.name, a.summary.q1, b.name, b.summary.q3
                ),
                1 => format!(
                    "The quantile comparison starts at Q1 = {:.2} {unit} for {} and ends at a maximum Q3 of {:.2} for {}. There are {flips} pairwise order reversals between these two slices of the distributions.",
                    a.summary.q1, a.name, b.summary.q3, b.name
                ),
                2 => format!(
                    "Comparing low-side and high-side summaries selects {} at Q1 ({:.2}) and {} at Q3 ({:.2} {unit}). The full configuration order contains {flips} strict reversals when the upper quartile replaces the lower.",
                    a.name, a.summary.q1, b.name, b.summary.q3
                ),
                _ => format!(
                    "{} supplies the largest first quartile, {:.2} {unit}, whereas {} supplies the largest third quartile, {:.2}. Across the roster, {flips} pairs change strict order between these quantiles.",
                    a.name, a.summary.q1, b.name, b.summary.q3
                ),
            };
            let interpretation = if lower[0] != upper[0] {
                format!(
                    "The quartile leader changes. {} has IQR {:.2} {unit}, compared with {:.2} for {}; the high-side and low-side summaries therefore select different response profiles rather than a common winner.",
                    a.name,
                    a.summary.q3 - a.summary.q1,
                    b.summary.q3 - b.summary.q1,
                    b.name
                )
            } else if flips > 0 {
                format!(
                    "{} remains first at both quartiles, but that agreement does not extend to every pair. Its own central interval spans {:.2} {unit}; lower-ranked configurations account for the reversals.",
                    a.name,
                    a.summary.q3 - a.summary.q1
                )
            } else {
                let names = lower
                    .iter()
                    .map(|&i| data.series[i].name.as_str())
                    .collect::<Vec<_>>()
                    .join(" → ");
                format!(
                    "No strict pair reverses between quartiles. In descending Q1 order the labels are {names}; any equal quartiles follow table order."
                )
            };
            (fact, interpretation)
        }
        Lens::PairOrder => {
            let a = &data.series[0];
            let b = &data.series[1];
            let (greater, less, ties) = response_pairs(a, b);
            let total = n * n;
            let fraction = 100.0 * greater as f64 / total as f64;
            let fact = match voice {
                0 => format!(
                    "Comparing every value of {} against every value of {} gives {total} cross-sample pairs. The first response is higher in {greater}, lower in {less}, and equal in {ties}: {fraction:.1}% are strict exceedances.",
                    a.name, b.name
                ),
                1 => format!(
                    "The all-pairs comparison of {} with {} contains {greater} strict exceedances, {less} deficits, and {ties} ties among {total} combinations. The exceedance share is {fraction:.1}%, with no alignment by observation index.",
                    a.name, b.name
                ),
                2 => format!(
                    "For {} versus {}, the empirical exceedance count is {greater}/{total} ({fraction:.1}%). Reversing the inequality counts {less} pairs, leaving {ties} equal-value pairs. Every observation enters against all {n} values of the other sample.",
                    a.name, b.name
                ),
                _ => format!(
                    "Ignoring sequence alignment, {} exceeds {} in {fraction:.1}% of the {total} possible response pairs. Counts are {greater} above, {less} below, and {ties} tied; these are combinations of recorded values, not independent trials.",
                    a.name, b.name
                ),
            };
            let mean_gap = a.summary.mean - b.summary.mean;
            let relation = if greater == total {
                "Every cross-sample comparison points in that direction: the observed ranges are strictly separated."
            } else if less == total {
                "Every cross-sample comparison has the opposite direction: the first sample lies strictly below the second."
            } else if (greater as f64 - less as f64) * mean_gap < 0.0 {
                "Pair frequency and mean order disagree; magnitudes and ordinal counts emphasize different observations."
            } else if greater == less {
                "Neither strict direction is more frequent, even though the means may differ in magnitude."
            } else {
                "Both strict directions occur; the mean ordering is not complete separation of the two samples."
            };
            (
                fact,
                format!(
                    "The mean contrast, {} minus {}, is {mean_gap:.2} {unit}. {relation}",
                    a.name, b.name
                ),
            )
        }
        Lens::Roughness => {
            let order = ranked(data, roughness);
            let high = &data.series[order[0]];
            let low = &data.series[*order.last().unwrap()];
            let h = roughness(high);
            let l = roughness(low);
            let fact = match voice {
                0 => format!(
                    "The largest mean absolute adjacent change is {h:.2} {unit} for {}; the smallest is {l:.2} for {}. Each average uses {count} one-step differences, without interpreting index steps as physical time.",
                    high.name,
                    low.name,
                    count = n - 1
                ),
                1 => format!(
                    "Local roughness ranks {} first ({h:.2} {unit} per index step) and {} last ({l:.2}). Here roughness is the average absolute difference between neighboring responses, not their SD around a common center.",
                    high.name, low.name
                ),
                2 => format!(
                    "A stepwise reading gives {} an average absolute increment of {h:.2} {unit}, versus {l:.2} for {}. These are the roughest and smoothest traces by this criterion over {count} adjacent pairs.",
                    high.name,
                    low.name,
                    count = n - 1
                ),
                _ => format!(
                    "{} changes by {h:.2} {unit} per step in mean absolute magnitude, compared with {l:.2} for {}. These endpoints of the local-roughness ordering use all adjacent differences, with their signs removed before averaging.",
                    high.name, low.name
                ),
            };
            let widest = &data.series[ranked(data, |s| s.summary.sd)[0]];
            let position = order
                .iter()
                .position(|&i| data.series[i].name == widest.name)
                .unwrap()
                + 1;
            let interpretation = if widest.name == high.name {
                format!(
                    "{} also has the largest sample SD ({:.2} {unit}). The same configuration is widest around its mean and roughest between adjacent values.",
                    high.name, high.summary.sd
                )
            } else {
                format!(
                    "The widest-SD trace is {} (SD {:.2} {unit}), only rank {position} by adjacent-change magnitude. Its roughness is {:.2}; overall spread and local fluctuation select different configurations.",
                    widest.name,
                    widest.summary.sd,
                    roughness(widest)
                )
            };
            (fact, interpretation)
        }
        Lens::Detrending => {
            let index = ranked(data, |s| line_fit(s).0.abs())[0];
            let s = &data.series[index];
            let (slope, residual_sd, explained) = line_fit(s);
            let percent = 100.0 * explained;
            let fact = match voice {
                0 => format!(
                    "The largest absolute fitted linear slope belongs to {}: {slope:.3} {unit} per index step. An ordinary least-squares line with an intercept accounts for {percent:.1}% of its centered sum of squares, leaving residual SD {residual_sd:.2}.",
                    s.name
                ),
                1 => format!(
                    "Fitting response against index gives {} the strongest slope in absolute magnitude, {slope:.3} {unit} per step. The line explains {percent:.1}% of observed squared variation; residual SD is {residual_sd:.2}, using the n − 2 divisor.",
                    s.name
                ),
                2 => format!(
                    "For {}, an intercept-and-slope fit yields {slope:.3} {unit} per index increment, the largest absolute slope among the traces. Its residual SD is {residual_sd:.2}; {percent:.1}% of the centered response sum of squares is assigned to the fitted line.",
                    s.name
                ),
                _ => format!(
                    "Linear detrending is most pronounced by absolute slope for {} ({slope:.3} {unit} per step). The fitted component accounts for {percent:.1}% of centered squared variation, while the residual SD is {residual_sd:.2}. No trend test is performed.",
                    s.name
                ),
            };
            let other = &data.series[ranked(data, |s| line_fit(s).1)[0]];
            let interpretation = format!(
                "Before detrending, {} has sample SD {:.2} {unit}; afterward its residual SD is {} that value. Residual SD uses n − 2 after fitting two coefficients. The largest residual SD across configurations is {:.2} for {}.",
                s.name,
                s.summary.sd,
                relative_position(residual_sd - s.summary.sd),
                line_fit(other).1,
                other.name
            );
            (fact, interpretation)
        }
        Lens::ReferenceBand => {
            let a = &data.series[0];
            let b = &data.series[1];
            let inside = |values: &Series, band: &Series| {
                values
                    .values
                    .iter()
                    .filter(|&&v| v >= band.summary.q1 && v <= band.summary.q3)
                    .count()
            };
            let ab = inside(a, b);
            let ba = inside(b, a);
            let fact = match voice {
                0 => format!(
                    "{} places {ab}/{n} values inside {}'s central band, {:.2}–{:.2} {unit}. In the reciprocal check, {ba}/{n} values of the second sample fall inside the first sample's band, {:.2}–{:.2}.",
                    a.name, b.name, b.summary.q1, b.summary.q3, a.summary.q1, a.summary.q3
                ),
                1 => format!(
                    "The central interval for {} ({:.2}–{:.2} {unit}) contains {ab} observations from {}. Reversing the reference, the interval {:.2}–{:.2} contains {ba} values from the other sample. Each denominator is {n}.",
                    b.name, b.summary.q1, b.summary.q3, a.name, a.summary.q1, a.summary.q3
                ),
                2 => format!(
                    "Cross-sample occupancy is {ab}/{n} for {} inside {}'s Q1–Q3 band and {ba}/{n} in the reverse direction. The bands are {:.2}–{:.2} and {:.2}–{:.2} {unit}, respectively.",
                    a.name, b.name, b.summary.q1, b.summary.q3, a.summary.q1, a.summary.q3
                ),
                _ => format!(
                    "Taking {} as the reference, its {:.2}–{:.2} {unit} central band includes {ab} values of {}. Taking the first sample's {:.2}–{:.2} band instead includes {ba} values from the second. Both samples contain {n} observations.",
                    b.name, b.summary.q1, b.summary.q3, a.name, a.summary.q1, a.summary.q3
                ),
            };
            let interpretation = if ab == 0 && ba == 0 {
                "Neither sample puts a recorded value inside the other's central interval. This concerns the central bands, not necessarily the full observed ranges.".to_owned()
            } else if ab == ba {
                format!(
                    "The reciprocal counts are equal at {ab}, despite using separately estimated bands. Count agreement does not require the interval widths to match: they are {:.2} for {} and {:.2} for {} {unit}.",
                    a.summary.q3 - a.summary.q1,
                    a.name,
                    b.summary.q3 - b.summary.q1,
                    b.name
                )
            } else {
                let aw = a.summary.q3 - a.summary.q1;
                let bw = b.summary.q3 - b.summary.q1;
                let widths = if aw == bw {
                    format!(
                        "Both bands have width {aw:.2} {unit}, despite their different occupancy counts."
                    )
                } else {
                    format!(
                        "The wider band belongs to {} ({:.2} versus {:.2} {unit}).",
                        if aw > bw { &a.name } else { &b.name },
                        aw.max(bw),
                        aw.min(bw)
                    )
                };
                format!(
                    "The reciprocal counts differ by {} observations. {widths} Occupancy and geometric width therefore remain distinct quantities.",
                    ab.abs_diff(ba)
                )
            };
            (fact, interpretation)
        }
        Lens::PointInfluence => {
            let (group, index, delta) =
                data.series
                    .iter()
                    .enumerate()
                    .flat_map(|(g, s)| {
                        s.values.iter().enumerate().map(move |(i, value)| {
                            (g, i, (s.summary.mean - value) / (n - 1) as f64)
                        })
                    })
                    .max_by(|a, b| a.2.abs().total_cmp(&b.2.abs()))
                    .unwrap();
            let s = &data.series[group];
            let reduced = s.summary.mean + delta;
            let fact = match voice {
                0 => format!(
                    "A single-observation deletion moves the mean most for {} at index {index}. Removing its value {:.2} changes the mean from {:.2} to {reduced:.2} {unit}: a signed shift of {delta:.2} over the remaining {} values.",
                    s.name,
                    s.values[index],
                    s.summary.mean,
                    n - 1
                ),
                1 => format!(
                    "The largest absolute leave-one-value influence is {delta:.2} {unit}, found in {} at index {index}. The omitted response is {:.2}; its deletion replaces mean {:.2} with {reduced:.2}, without changing any other configuration.",
                    s.name, s.values[index], s.summary.mean
                ),
                2 => format!(
                    "Omitting index {index} from {} gives mean {reduced:.2} instead of {:.2} {unit}. The removed value is {:.2}, and the signed change {delta:.2} is largest in absolute magnitude among all single-value deletions in this setting.",
                    s.name, s.summary.mean, s.values[index]
                ),
                _ => format!(
                    "{} contains the most influential individual response for a leave-one-value mean check: {:.2} at index {index}. Deletion changes its mean by {delta:.2} {unit}, from {:.2} to {reduced:.2}. The original table retains this observation.",
                    s.name, s.values[index], s.summary.mean
                ),
            };
            let neighbor = data
                .series
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != group)
                .min_by(|(_, a), (_, b)| {
                    (a.summary.mean - s.summary.mean)
                        .abs()
                        .total_cmp(&(b.summary.mean - s.summary.mean).abs())
                })
                .unwrap()
                .1;
            let old_gap = s.summary.mean - neighbor.summary.mean;
            let new_gap = reduced - neighbor.summary.mean;
            let interpretation = format!(
                "Relative to the nearest mean, {}'s, the signed contrast changes from {old_gap:.2} to {new_gap:.2} {unit}. {} This is a deletion diagnostic, not a claim that the omitted response was erroneous.",
                neighbor.name,
                if old_gap * new_gap < 0.0 {
                    "The strict pair order reverses."
                } else if new_gap == 0.0 {
                    "The adjusted means tie."
                } else {
                    "The original strict direction is not reversed."
                }
            );
            (fact, interpretation)
        }
        _ => unreachable!("original lenses have their own reporting"),
    };
    vec![fact, interpretation]
}

// These questions motivate the diagnostics; the generated series do not simulate
// the domain mechanisms themselves.
const QUESTIONS: [[&str; 3]; 8] = [
    [
        "An algorithm can look attractive by average response while behaving unevenly across steps. A deployment decision would need to distinguish throughput, latency, and failure cost; an unnamed response index cannot resolve that trade-off.",
        "A comparison between computational procedures needs an explicit unit of work. Equal-length observation sequences make the descriptive arithmetic comparable, but they do not equalize messages sent, memory consumed, or processor time.",
        "A systems claim must distinguish repeatable output from guarantees under an adversary. The ordering of simulated configurations is worth documenting precisely because it is a much weaker object than correctness, scalability, or fault tolerance.",
    ],
    [
        "A finite construction can illustrate why two summaries differ without settling a mathematical conjecture. The useful question here is what survives a change of summary, rather than whether the sample proves a property for an entire class of objects.",
        "Location and dispersion encode different aspects of an approximation. Two constructions with similar averages can have different extents, so reducing their comparison to a scalar would discard structure before any limiting argument could begin.",
        "A numerical ordering is conditional on the chosen representation. An eventual theorem would need hypotheses connecting that representation to the mathematical object; this exercise instead makes the finite descriptive contrasts explicit.",
    ],
    [
        "Separating a response level from its fluctuations is essential before interpreting a physical mechanism. A broader distribution might reflect drift or oscillation rather than a new state, and a displaced mean alone cannot decide among those explanations.",
        "An observation window is part of a physical measurement, not merely a bookkeeping choice. Early and late portions can weight an oscillation differently; examining that dependence is more defensible here than interpreting the average as a conserved or equilibrium quantity.",
        "A numerical trace and an instrument reading have different evidential roles. Calibration, dimensions, and a model linking the observable to a physical state would be required before the relative positions of these traces could test a theory.",
    ],
    [
        "The statistic used to order estimators is itself a modeling choice. Means respond to every magnitude, medians emphasize central order, and interval widths combine sample size with dispersion; agreement among them cannot be assumed in advance.",
        "The target of inference must be specified before uncertainty can be interpreted. An interval calculated from a generated sequence has arithmetic meaning even when no population, assignment mechanism, or repeated-sampling design warrants a coverage claim.",
        "A robustness check should say exactly what was changed. Deleting endpoints, moving a window boundary, and changing a location statistic ask different questions; none is equivalent to drawing independent validation data.",
    ],
    [
        "Biological heterogeneity can disappear in an average. A response distributed broadly across observations would motivate a different follow-up from a uniform shift, but simulated values cannot decide whether either pattern has a cellular, organismal, or ecological origin.",
        "A sequence index does not by itself define a biological timescale. Interpreting short-window differences would require information about generations, sampling schedules, or phase; the present calculations deliberately stop at the recorded order.",
        "A configuration-level contrast is not a mechanism. Biological interpretation would require an observation model and independently measured covariates, so distributional descriptions are kept separate from claims about adaptation, regulation, or interaction.",
    ],
    [
        "A large response and a dependable response need not select the same allocation rule. Without prices, positions, transaction costs, and a loss definition, even a clear ranking in a synthetic index cannot establish an investment preference.",
        "Averages can conceal a response concentrated in a few observations. Finance would attach particular importance to which side corresponds to loss; this construction has no such mapping, so tail extent is described without calling it downside risk.",
        "Comparing rules across regimes raises a transport question before a return question. A configuration's position may change when the window changes, but a sequence label supplies neither a trading calendar nor an independently specified stress scenario.",
    ],
    [
        "An aggregate response does not reveal who gains or loses. A welfare interpretation would require a mapping from the index to agents and outcomes; the present comparison instead asks how strongly the summary depends on configuration and window.",
        "A policy label does not create a counterfactual. Assignment, behavior, and resource constraints would need explicit models before differences could identify policy effects; descriptive ordering remains a separate and narrower task.",
        "A stable ranking of alternatives can coexist with a fragile magnitude estimate. Before using either in an economic argument, one would need units and a population of interest, neither of which is recovered from a generated response table.",
    ],
    [
        "Average response is not a substitute for an engineering specification. A controller or estimator may be judged by overshoot, error, power, or reliability, and the synthetic index is not calibrated to any of these requirements.",
        "Sequence structure matters when interpreting a signal-processing comparison. A drift component and an oscillation can alter summaries without representing a sensor fault or a controller instability; the window check separates that arithmetic from diagnosis.",
        "A test-bench description needs a link between stimulus and response. Here configuration labels organize generated traces, but no hardware, plant dynamics, or measurement chain supports a claim about deployment performance.",
    ],
];

fn pick<'a>(choices: &'a [&'a str], rng: &mut ChaCha8Rng) -> &'a str {
    choices.choose(rng).unwrap()
}

fn introduction(id: &PaperId, data: &[Experiment], lens: Lens) -> Vec<String> {
    let mut rng = rng(&format!("{id}:narrative-introduction-v3"));
    let topic = TOPICS[id.category][id.topic];
    let frame = &TOPIC_FRAMES[id.category][id.topic];
    let count = data.len();
    let labels = data[0]
        .series
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let agenda = QUESTIONS[id.category][rng.gen_range(0..3)];
    let question = match lens {
        Lens::Location | Lens::Balance => {
            "how the apparent lead depends on the reference used to describe it"
        }
        Lens::Dispersion | Lens::Occupancy => {
            "whether response level and concentration point to the same configuration"
        }
        Lens::Quartiles | Lens::Extent => {
            "what the central and outer portions of the distributions contribute to the comparison"
        }
        Lens::Overlap | Lens::Intervals => {
            "what separation of descriptive summaries does, and does not, establish"
        }
        Lens::Windows | Lens::Crossings => {
            "how much of the comparison survives attention to the recorded sequence"
        }
        Lens::Ranking | Lens::Trimming => {
            "whether a change in the summary rule changes the apparent ordering"
        }
        Lens::QuantileRanks => {
            "whether low-side and high-side summaries preserve a common ordering"
        }
        Lens::PairOrder => {
            "whether a mean contrast agrees with ordering individual responses across samples"
        }
        Lens::Roughness => {
            "whether a wide distribution also fluctuates strongly between adjacent observations"
        }
        Lens::Detrending => {
            "how much recorded variation remains after removing a fitted linear index component"
        }
        Lens::ReferenceBand => {
            "whether central-band occupancy is symmetric when the reference sample changes"
        }
        Lens::PointInfluence => {
            "whether a single observation can change the configuration comparison"
        }
    };
    let opening = match rng.gen_range(0..6) {
        0 => format!(
            "{frame_premise} The immediate question for {topic} is {question}. We address that narrower question using {count} generated settings, rather than presenting the configuration labels as implemented scientific methods.",
            frame_premise = frame.premise
        ),
        1 => format!(
            "What can a finite response table tell us about {topic}? {frame_premise} Our starting point is {question}; the {count} settings offer multiple descriptive windows, not external replications of the domain mechanism.",
            frame_premise = frame.premise
        ),
        2 => format!(
            "This account of {topic} begins with a limitation on interpretation. {agenda} Its positive task is to establish {question} across the supplied synthetic observations."
        ),
        3 => format!(
            "A comparative account of {topic} can start from a distribution rather than a winner. {frame_premise} We use {count} settings to ask {question}, keeping the numerical evidence separate from the mechanism that motivates the question.",
            frame_premise = frame.premise
        ),
        4 => format!(
            "The organizing problem is {question}. For {topic}, that problem matters because the scientific premise is conditional: {premise} The generated response sequences permit inspection of the descriptive consequences, not verification of that premise.",
            premise = frame.premise
        ),
        _ => format!(
            "Rather than assume one summary settles the comparison, we examine {topic} through {count} observation settings. {frame_premise} The first analytical emphasis is {question}. Subsequent windows expose other aspects of the same labeled configuration set.",
            frame_premise = frame.premise
        ),
    };
    let roster = match rng.gen_range(0..4) {
        0 => format!(
            "The alternatives are {labels}. These are persistent labels for response sequences, not evidence that the named algorithms or policies have been run. Their order is kept in the tables even when the prose selects a different pair, an extreme, or a sensitivity check."
        ),
        1 => format!(
            "We retain the labels {labels} throughout. Keeping identities fixed lets a reader follow a configuration from one window to another, but does not make those windows commensurate experiments on an implemented procedure. The labels specify the comparison set, not a performance guarantee."
        ),
        2 => format!(
            "The comparison set comprises {labels}. Choosing the largest mean, the widest spread, or the closest pair is an explicit descriptive selection from this set. None of these selections changes which observations are shown in the tables and figures."
        ),
        _ => format!(
            "All settings use {labels}. A name denotes a synthetic configuration here; it does not certify an implementation, treatment, or fitted scientific model. This distinction matters especially when an apparently familiar method name encourages a stronger reading than the numerical construction supports."
        ),
    };
    let mut paragraphs = vec![opening, roster];
    if !paragraphs[0].contains(frame.premise) {
        paragraphs[0].push_str(&format!(" {}", frame.premise));
    }
    if !paragraphs[0].contains(agenda) {
        paragraphs.insert(1, agenda.to_owned());
    }
    paragraphs
}

fn design(id: &PaperId, data: &[Experiment]) -> (Vec<String>, Vec<String>) {
    let mut rng = rng(&format!("{id}:narrative-design-v3"));
    let frame = &TOPIC_FRAMES[id.category][id.topic];
    let first = &data[0];
    let min_n = data.iter().map(|e| e.series[0].summary.n).min().unwrap();
    let max_n = data.iter().map(|e| e.series[0].summary.n).max().unwrap();
    let count = data.len();
    let groups = first.series.len();
    let sampling = match rng.gen_range(0..6) {
        0 => format!(
            "Each of the {count} windows contains {groups} sequences of equal length; lengths across windows range from {min_n} to {max_n}. The initial window is the {}, with {} observations per label. A fresh setting changes the generated values, not the roster of configurations.",
            first.context, first.series[0].summary.n
        ),
        1 => format!(
            "The sampling unit for the descriptive tables is a value at a sequence index. We generate {count} sets of {groups} sequences. Within a set the lengths match; between sets they vary from {min_n} to {max_n}. In particular, the {} uses {} values per configuration.",
            first.context, first.series[0].summary.n
        ),
        2 => format!(
            "The design is balanced within windows, not necessarily between them. Every configuration in the {} contributes {} values, and the other settings use counts within {min_n}–{max_n}. There are {groups} fixed labels and {count} windows, with no missing entries or unequal within-window weights.",
            first.context, first.series[0].summary.n
        ),
        3 => format!(
            "Observation counts belong to settings: the first ({}) assigns {} values to each configuration. Over all {count} settings, the shortest and longest windows contain {min_n} and {max_n} values. This supplies equal information by count within a setting, not equal effective sample sizes under dependence.",
            first.context, first.series[0].summary.n
        ),
        4 => format!(
            "We keep the roster fixed at {groups} configurations while regenerating their sequences {count} times. Window lengths run from {min_n} to {max_n}; {} values per label appear in the opening {}. Balancing counts removes one source of weighting differences without standardizing the contexts or units.",
            first.series[0].summary.n, first.context
        ),
        _ => format!(
            "The opening {}, with {} observations for each of {groups} configurations, is one of {count} generated windows. Other windows can be shorter or longer; the realized limits are {min_n} and {max_n}. No observations are dropped before the primary summaries, including those at either extreme.",
            first.context, first.series[0].summary.n
        ),
    };
    let generator = "Responses follow offset + slope × index + 2 sin(index / 6) + Uniform[−4, 4). Offsets include a row-position contribution. All contexts redraw under these rules; names denote neither implemented methods nor controlled interventions.";
    let summaries = "Figures and tables reuse the unmodified observations. Means are arithmetic; sample SD uses n − 1; quartiles interpolate at sorted positions p(n − 1). Calculations precede two-decimal display rounding.";
    let intervals = "Mean intervals use mean ± 1.96 × sample SD / √n. This illustrative normal approximation ignores drift and serial structure: it supplies neither calibrated confidence coverage nor a test of significance.";
    (
        vec![sampling, generator.to_owned(), frame.protocol.to_owned()],
        vec![summaries.to_owned(), intervals.to_owned()],
    )
}

fn unit_scope(data: &[Experiment], voice: usize) -> String {
    let first = &data[0];
    let last = data.last().unwrap();
    let units = data.iter().map(|e| &e.unit).collect::<BTreeSet<_>>().len();
    match voice {
        0 => format!(
            "The {units} unit labels begin with {} and end with {}. Comparisons stay within windows; the cross-setting audit carries ranks, not pooled magnitudes.",
            first.unit, last.unit
        ),
        1 => format!(
            "Units attach to windows: {} initially, {} finally, and {units} labels overall. Matching labels do not calibrate separately generated contexts, so raw means are not pooled.",
            first.unit, last.unit
        ),
        2 => format!(
            "There is no conversion rule among the {units} unit labels, including {} and {} at the endpoints. Only within-window ordering enters the cross-setting synthesis.",
            first.unit, last.unit
        ),
        _ => format!(
            "Opening and final responses use {} and {}, respectively ({units} unit labels overall). An identical unit name would not ensure comparable conditions; numerical differences remain local.",
            first.unit, last.unit
        ),
    }
}

fn synthesis(data: &[Experiment], voice: usize) -> Vec<String> {
    let first = &data[0];
    let last = data.last().unwrap();
    let orders = data
        .iter()
        .map(|e| ranked(e, |s| s.summary.mean))
        .collect::<Vec<_>>();
    let winners = orders.iter().map(|o| o[0]).collect::<BTreeSet<_>>();
    let switches = orders.windows(2).filter(|p| p[0][0] != p[1][0]).count();
    let first_top = &first.series[orders[0][0]];
    let last_top = &last.series[orders.last().unwrap()[0]];
    let mut reversals = 0;
    for i in 0..first.series.len() {
        for j in i + 1..first.series.len() {
            reversals += usize::from(
                (first.series[i].summary.mean - first.series[j].summary.mean)
                    * (last.series[i].summary.mean - last.series[j].summary.mean)
                    < 0.0,
            );
        }
    }
    let distinct = winners.len();
    let fact = match voice {
        0 => format!(
            "Across {} settings, {distinct} distinct configurations attain a largest mean. The leading label changes {switches} times between neighboring settings in display order. {} leads the opening window and {} the final window; comparing all pairs between those endpoints yields {reversals} strict order reversals.",
            data.len(),
            first_top.name,
            last_top.name
        ),
        1 => format!(
            "The cross-setting audit follows labels, not numerical magnitudes. It finds {distinct} different mean leaders and {switches} changes of leader over adjacent windows. The first leader is {}, the last {}. Between the opening and final rankings, {reversals} pairs exchange strict order.",
            first_top.name, last_top.name
        ),
        2 => format!(
            "A single ranking would leave out part of the record: {} windows produce {distinct} distinct leaders. In displayed sequence there are {switches} top-label changes. The endpoint comparison starts with {} and ends with {} at the top, with {reversals} strict pairwise reversals between those windows.",
            data.len(),
            first_top.name,
            last_top.name
        ),
        _ => format!(
            "Tracking the greatest mean through the settings identifies {distinct} leading labels, with {switches} transitions from one leader to another. The endpoints are {} in the first setting and {} in the last. The full endpoint rankings, not just their leaders, contain {reversals} strict pair reversals.",
            first_top.name, last_top.name
        ),
    };
    let interpretation = match voice {
        0 => {
            let mut best = (0, 0);
            let mut start = 0;
            for end in 0..orders.len() {
                if end > 0 && orders[end][0] != orders[end - 1][0] {
                    start = end;
                }
                if end - start > best.1 - best.0 {
                    best = (start, end);
                }
            }
            let leader = &first.series[orders[best.0][0]].name;
            if distinct == 1 {
                let tail = if reversals == 0 {
                    "The endpoint ranking also retains every strict pair order."
                } else {
                    "A persistent leader coexists with changes lower in the ranking."
                };
                format!("{leader} leads all {} windows. {tail}", data.len())
            } else if best.0 == best.1 {
                format!(
                    "No leading label repeats in consecutive windows. Even {leader}, selected in window {}, is replaced immediately at the next comparison. Leadership is dispersed in display order rather than sustained in a block.",
                    best.0 + 1
                )
            } else {
                let outside = orders
                    .iter()
                    .enumerate()
                    .filter(|(i, o)| (*i < best.0 || *i > best.1) && o[0] == orders[best.0][0])
                    .count();
                format!(
                    "{leader} has the longest uninterrupted lead, covering windows {}–{}. Outside that block, its count of leading windows is {outside}. This separates a consecutive run from total frequency of leadership; the block comprises {} different generated contexts.",
                    best.0 + 1,
                    best.1 + 1,
                    best.1 - best.0 + 1
                )
            }
        }
        1 => {
            let histories = rank_histories(&orders);
            let (index, history) = histories
                .iter()
                .enumerate()
                .max_by_key(|(_, h)| h.iter().max().unwrap() - h.iter().min().unwrap())
                .unwrap();
            let best = history.iter().min().unwrap();
            let worst = history.iter().max().unwrap();
            let route = history
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" → ");
            let name = &first.series[index].name;
            if best == worst {
                format!(
                    "Every configuration retains its ordinal position. For example, {name} remains at rank {best}: {route}. The changing context titles do not produce a rank reversal in this set of draws."
                )
            } else {
                let extremes = if *best == 1 && *worst == first.series.len() {
                    "It appears at both ends of the ranking."
                } else {
                    "Its movement spans only part of the available ranking."
                };
                format!(
                    "{name} has a largest rank excursion, ranging from {best} to {worst}. Its window-by-window path is {route}, with rank 1 denoting the largest mean. {extremes}"
                )
            }
        }
        2 => {
            let mut selected = (0, 1, 0);
            for a in 0..first.series.len() {
                for b in a + 1..first.series.len() {
                    let flips = data
                        .windows(2)
                        .filter(|pair| {
                            let gap = |e: &Experiment| {
                                e.series[a].summary.mean - e.series[b].summary.mean
                            };
                            gap(&pair[0]) * gap(&pair[1]) < 0.0
                        })
                        .count();
                    if flips > selected.2 {
                        selected = (a, b, flips);
                    }
                }
            }
            let (a, b, flips) = selected;
            let above = data
                .iter()
                .filter(|e| e.series[a].summary.mean > e.series[b].summary.mean)
                .count();
            let below = data
                .iter()
                .filter(|e| e.series[a].summary.mean < e.series[b].summary.mean)
                .count();
            let relation = match flips {
                0 => "No pair strictly reverses between adjacent windows.",
                1 => "This is a single exchange of order, not repeated alternation.",
                _ => "This pair exchanges order repeatedly rather than changing sides only once.",
            };
            format!(
                "{} versus {} has a largest adjacent reversal count: {flips}. The first has the higher mean in {above} windows and the lower mean in {below}, with {} ties. {relation}",
                first.series[a].name,
                first.series[b].name,
                data.len() - above - below
            )
        }
        _ => {
            let joint = data
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    let tight = ranked(e, |s| -s.summary.sd)[0];
                    (orders[i][0] == tight).then_some((i + 1).to_string())
                })
                .collect::<Vec<_>>();
            let tight = &first.series[ranked(first, |s| -s.summary.sd)[0]];
            let opening = format!(
                "In the opening {}, {} leads by mean but has SD {:.2} {}; the minimum SD is {:.2} for {}.",
                first.context,
                first_top.name,
                first_top.summary.sd,
                first.unit,
                tight.summary.sd,
                tight.name
            );
            if joint.is_empty() {
                format!(
                    "No window selects the same label by largest mean and smallest SD. {opening}"
                )
            } else if joint.len() == data.len() {
                format!(
                    "The mean leader also has minimum SD in every window. For the opening {}, both criteria select {}, with mean {:.2} and SD {:.2} {}.",
                    first.context,
                    first_top.name,
                    first_top.summary.mean,
                    first_top.summary.sd,
                    first.unit
                )
            } else {
                format!(
                    "The mean and minimum-SD selections coincide in windows {} only. {opening}",
                    joint.join(", ")
                )
            }
        }
    };
    vec![fact, interpretation]
}

fn rank_histories(orders: &[Vec<usize>]) -> Vec<Vec<usize>> {
    (0..orders[0].len())
        .map(|configuration| {
            orders
                .iter()
                .map(|order| order.iter().position(|&i| i == configuration).unwrap() + 1)
                .collect()
        })
        .collect()
}

fn limitations(id: &PaperId, lens: Lens) -> Vec<String> {
    let mut rng = rng(&format!("{id}:narrative-limitations-v3"));
    let frame = &TOPIC_FRAMES[id.category][id.topic];
    let boundary = frame.reading.to_owned();
    let follow_up = match lens {
        Lens::Location | Lens::Balance | Lens::ReferenceBand => pick(
            &[
                "A useful next experiment would define the cost of a response difference before adding more alternatives. It could then vary configuration composition while holding the measurement scale fixed, checking whether a claimed advantage survives the choice of reference rather than merely a new random seed.",
                "Reference sensitivity calls for a design with an externally justified comparison point. Varying the roster would show how much the internal center moves, but it would still need a common response objective before that movement could guide scientific or practical decisions.",
            ],
            &mut rng,
        ),
        Lens::Dispersion | Lens::Occupancy => pick(
            &[
                "Further work should separate within-window fluctuation from variation between independent replications. A concentration criterion would also need to be fixed before observing the samples; otherwise selecting the narrowest trace and then treating its spread as typical would obscure selection.",
                "The next spread-focused check would hold sequence length fixed while varying drift and random deviation separately. This would distinguish variability caused by the window from variability caused by the perturbation amplitude, without declaring a low SD intrinsically desirable.",
            ],
            &mut rng,
        ),
        Lens::Quartiles | Lens::Extent => pick(
            &[
                "A shape-focused extension would compare empirical distributions under prespecified window lengths and known perturbation laws. Quantiles and extremes would then be interpreted against those laws, rather than inferring a tail class from one finite range or a smoothed visual profile.",
                "To distinguish a broad central distribution from a few extended endpoints, additional independent sequences would be more informative than a finer plot. Their lengths and generating assumptions should be fixed in advance; endpoint stability cannot be judged by redrawing the same observations.",
            ],
            &mut rng,
        ),
        Lens::Overlap | Lens::Intervals | Lens::PairOrder => pick(
            &[
                "An inferential extension needs a target population and a dependence-aware sampling model before revising the interval formula. Merely increasing the multiplier or looking for separated bars would not fix the absence of calibrated coverage for a configuration contrast.",
                "For uncertainty claims, a future design would have to specify what is repeatedly sampled and why that repetition approximates the scientific target. It could then evaluate contrast intervals directly, rather than using overlap of marginal summaries as a substitute for that design.",
            ],
            &mut rng,
        ),
        Lens::Windows | Lens::Crossings | Lens::Roughness | Lens::Detrending => pick(
            &[
                "A temporal extension should vary the observation window and the oscillatory component separately, with independent replications at each combination. Only then could a window effect be distinguished from a change in the signal-generating rule; the current contexts redraw several ingredients at once.",
                "The sequence checks suggest fixing an index-to-time mapping before making a dynamical claim. Testing alternative split points or alignment rules would assess descriptive sensitivity, but would not by itself validate a mechanism behind a local crossing or half-window displacement.",
            ],
            &mut rng,
        ),
        Lens::Ranking | Lens::Trimming | Lens::QuantileRanks | Lens::PointInfluence => pick(
            &[
                "A robustness study should choose its transformation before inspection and repeat it across independent sequences. Endpoint removal is not a contamination model, and changing the location statistic is not a new dataset; both remain diagnostic choices whose effect must be reported explicitly.",
                "An informative extension would predefine the ranking criterion and the perturbations against which it should be stable. If the target is a median rather than a mean, that choice needs a scientific reason, not retrospective selection of whichever summary preserves a preferred label.",
            ],
            &mut rng,
        ),
    };
    vec![boundary, follow_up.to_owned()]
}

fn conclusion(id: &PaperId, data: &[Experiment], voice: usize) -> Vec<String> {
    let first = &data[0];
    let orders = data
        .iter()
        .map(|e| ranked(e, |s| s.summary.mean))
        .collect::<Vec<_>>();
    let winner = orders[0][0];
    let retained = orders.iter().filter(|o| o[0] == winner).count();
    let topic = TOPICS[id.category][id.topic];
    let name = &first.series[winner].name;
    let count = data.len();
    let text = match voice {
        0 => format!(
            "The evidence for {topic} is conditional on the summary and the window. The opening mean leader, {name}, leads {retained} of {count} settings. This count is traceable to the individual tables; it is not a probability of future superiority. A stronger conclusion would require both a meaningful response objective and a model connecting the configurations to the scientific question."
        ),
        1 => format!(
            "For {topic}, the defensible endpoint is a set of documented contrasts rather than a method recommendation. {name} starts as the mean leader and retains that position in {retained}/{count} settings. The distributional and sequence checks explain what an average alone leaves unresolved, while the numerical construction prevents treating the ordering as a validated domain finding."
        ),
        2 => format!(
            "Returning to the comparison of {topic}, {name} holds the largest mean in {retained} of the {count} windows, including the first. Frequency of leadership is descriptive and sensitive to the chosen construction. Its value here is to make persistence inspectable without merging incompatible response magnitudes or extending a synthetic ranking into a real-world claim."
        ),
        _ => format!(
            "This analysis stops short of naming a universally preferable configuration for {topic}. The first-window mean leader, {name}, appears at the top {retained} times across {count} settings. That is an auditable feature of these samples, not an implementation benchmark. The remaining task is scientific calibration: establishing which response, which comparison, and which independent measurements would make the distinction consequential."
        ),
    };
    vec![text]
}

pub(super) fn sections(
    id: &PaperId,
    metadata: &Metadata,
    data: &[Experiment],
    style: Archetype,
) -> Vec<Section> {
    let mut outline_rng = rng(&format!("{id}:narrative-outline-v3"));
    let mut lens_rng = rng(&format!("{id}:narrative-lenses-v3"));
    let mut lenses = Lens::ALL;
    lenses.shuffle(&mut lens_rng);
    let layout = outline_rng.gen_range(0..6);
    let mut sections = Vec::new();
    let opening = pick(
        &[
            "Introduction",
            "The comparison problem",
            "Research context",
            "Questions and scope",
            "Motivation",
        ],
        &mut outline_rng,
    );
    add_section(
        &mut sections,
        2,
        opening,
        introduction(id, data, lenses[0]),
        vec![],
        None,
    );
    let (design, mut definitions) = design(id, data);
    definitions.push(unit_scope(data, outline_rng.gen_range(0..4)));
    let method_title = pick(
        &[
            "Study design",
            "Construction of the response windows",
            "Numerical framework",
            "Observation design",
            "Methods",
        ],
        &mut outline_rng,
    );
    if matches!(layout, 0 | 3) {
        add_section(
            &mut sections,
            2,
            method_title,
            [design, definitions].concat(),
            vec![],
            None,
        );
    } else {
        add_section(&mut sections, 2, method_title, design, vec![], None);
        add_section(
            &mut sections,
            if layout == 2 { 3 } else { 2 },
            pick(
                &[
                    "Summary rules and units",
                    "Measurement and comparison",
                    "Reading the numerical evidence",
                    "Analysis protocol",
                ],
                &mut outline_rng,
            ),
            definitions,
            vec![],
            None,
        );
    }
    if layout == 3 {
        add_section(
            &mut sections,
            2,
            "Interpretive boundaries",
            limitations(id, lenses[0]),
            vec![],
            None,
        );
    }
    let result_title = match style {
        Archetype::Comparative => "Comparative results",
        Archetype::Methods => "Benchmark results",
        Archetype::Observational => "Observed patterns",
    };
    let transition = match outline_rng.gen_range(0..4) {
        0 => format!(
            "The {} settings below receive different analytical checks. Each check names its selection rule—such as the closest mean pair or a fixed row—so a highlighted configuration is not mistaken for a prespecified experimental treatment. Full summaries remain available beside every figure.",
            data.len()
        ),
        1 => format!(
            "We begin with {} and retain the original setting order. The prose changes its emphasis between windows instead of repeating a fixed report for every table. A distributional check, a sequence check, and a rank comparison answer distinct descriptive questions about the same response arrays.",
            lenses[0].title().to_lowercase()
        ),
        2 => format!(
            "For {}, the results separate an observed contrast from its interpretation. No setting title introduces a new causal intervention: the following analyses use the declared construction and keep all configurations in the accompanying tables, including those not singled out in the text.",
            metadata.topic
        ),
        _ => format!(
            "The first result emphasizes {}; later results turn to other properties of the samples. The figures preserve their original numbering and observations. Differences mentioned in the text belong to the unit printed with that setting, rather than a common scale imposed after inspection.",
            lenses[0].title().to_lowercase()
        ),
    };
    add_section(
        &mut sections,
        2,
        result_title,
        vec![transition],
        vec![],
        None,
    );
    let block = match layout {
        1 => data.len().div_ceil(2),
        2 => 3,
        5 => 2,
        _ => data.len(),
    };
    for (index, experiment) in data.iter().enumerate() {
        if index > 0 && index % block == 0 {
            let title = format!(
                "Observation windows {}–{}",
                index + 1,
                (index + block).min(data.len())
            );
            add_section(&mut sections, 2, title, vec![], vec![], None);
        }
        let voice = rng(&format!("{id}:narrative-result-v3:{index}")).gen_range(0..4);
        let title = if layout % 2 == 0 {
            format!(
                "{}: {} ({})",
                lenses[index].title(),
                experiment.context,
                experiment.unit
            )
        } else {
            format!(
                "{} ({})",
                capitalize_title(&experiment.context),
                experiment.unit
            )
        };
        add_section(
            &mut sections,
            if layout == 4 { 2 } else { 3 },
            title,
            result_paragraphs(experiment, lenses[index], voice),
            tables(experiment),
            Some(figure(id, index)),
        );
    }
    add_section(
        &mut sections,
        2,
        "Cross-setting comparison",
        synthesis(data, outline_rng.gen_range(0..4)),
        vec![],
        None,
    );
    if layout != 3 {
        add_section(
            &mut sections,
            if layout == 5 { 3 } else { 2 },
            pick(
                &[
                    "Scope and limitations",
                    "What the comparison leaves open",
                    "Limits of interpretation",
                    "Requirements for validation",
                ],
                &mut outline_rng,
            ),
            limitations(id, lenses[0]),
            vec![],
            None,
        );
    }
    add_section(
        &mut sections,
        2,
        pick(
            &[
                "Conclusion",
                "Concluding remarks",
                "Implications and next questions",
                "Outlook",
            ],
            &mut outline_rng,
        ),
        conclusion(id, data, outline_rng.gen_range(0..4)),
        vec![],
        None,
    );
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Experiment {
        Experiment {
            context: "test window".to_owned(),
            unit: "test units".to_owned(),
            series: [
                ("A", vec![0.0, 1.0, 2.0, 3.0, 4.0, 30.0]),
                ("B", vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0]),
                ("C", vec![0.0, 0.0, 0.0, 0.0, 0.0, 50.0]),
            ]
            .into_iter()
            .map(|(name, values)| Series {
                name: name.to_owned(),
                summary: summarize(&values),
                values,
            })
            .collect(),
        }
    }

    #[test]
    fn analytical_checks_match_known_observations_in_every_voice() {
        let data = fixture();
        assert_eq!(ranked(&data, |s| s.summary.mean), [2, 1, 0]);
        assert_eq!(ranked(&data, |s| s.summary.median), [1, 0, 2]);
        assert_eq!(pair_reversals(&data, |s| s.summary.median), 2);
        assert_eq!(pair_reversals(&data, trimmed), 2);
        assert_eq!(trimmed(&data.series[0]), 2.5);
        assert_eq!(trimmed(&data.series[2]), 0.0);
        assert_eq!(halves(&data.series[2]), (0.0, 50.0 / 3.0));
        for voice in 0..4 {
            for lens in Lens::ALL {
                let prose = result_paragraphs(&data, lens, voice).join(" ");
                assert!(prose.contains("test units"), "{lens:?}/{voice}");
                let expected: &[&str] = match lens {
                    Lens::Location => &["8.33", "7.50", "6.67", "0.83", "1.67"],
                    Lens::Dispersion => &["20.41", "1.87"],
                    Lens::Quartiles => &["8.33", "0.00"],
                    Lens::Overlap => &["1.25", "3.75", "6.25", "8.75"],
                    Lens::Windows => &["0.00", "16.67"],
                    Lens::Ranking => &["8.33", "7.50", "2", "3"],
                    Lens::Trimming => &["0.00", "8.33", "-8.33"],
                    Lens::Extent => &["50.00", "0.00"],
                    Lens::Intervals => &["8.33", "7.50"],
                    Lens::Occupancy => &["5", "6"],
                    Lens::Crossings => &["6.67", "7.50"],
                    Lens::Balance => &["7.50", "0.83"],
                    Lens::QuantileRanks => &["B", "6.25", "8.75"],
                    Lens::PairOrder => &["36", "6", "30", "16.7%", "-0.83"],
                    Lens::Roughness => &["C", "B", "10.00", "1.00", "20.41"],
                    Lens::Detrending => &["C", "7.143", "42.9%", "17.25", "20.41"],
                    Lens::ReferenceBand => &["6.25", "8.75", "1.25", "3.75", "Neither sample"],
                    Lens::PointInfluence => &["C", "50.00", "8.33", "0.00", "-8.33", "reverses"],
                };
                for value in expected {
                    assert!(
                        prose.contains(value),
                        "{lens:?}/{voice}: missing {value}: {prose}"
                    );
                }
                if voice >= 2 && !lens.is_additional() {
                    let companion = companion_check(&data, lens, voice);
                    let extra: &[&str] = match lens {
                        Lens::Location => &["-2.50"],
                        Lens::Dispersion => &["A", "2.50"],
                        Lens::Quartiles => &["50.00"],
                        Lens::Overlap => &["5.00", "10.00"],
                        Lens::Windows => &["B", "C", "changes"],
                        Lens::Ranking => &["2 strict"],
                        Lens::Trimming => &["25.00"],
                        Lens::Extent => &["1", "6"],
                        Lens::Intervals => &["32.67"],
                        Lens::Occupancy => &["3", "2.50"],
                        Lens::Crossings => &["-5.00", "20.00"],
                        Lens::Balance => &["7.08"],
                        _ => unreachable!("additional diagnostics checked separately"),
                    };
                    for value in extra {
                        assert!(companion.contains(value), "{lens:?}/{voice}: {companion}");
                    }
                }
                assert_eq!(
                    result_paragraphs(&data, lens, voice),
                    result_paragraphs(&data, lens, voice)
                );
            }
        }
    }

    #[test]
    fn direction_overlap_ranking_and_sequence_branches_follow_the_data() {
        let mut data = fixture();
        for series in &mut data.series {
            series.values.reverse();
            series.summary = summarize(&series.values);
        }
        let prose = result_paragraphs(&data, Lens::Windows, 0).join(" ");
        assert!(prose.contains("-16.67"));
        assert!(prose.contains("later block is lower"));

        let odd = Series {
            name: "Odd".to_owned(),
            values: vec![1.0, 2.0, 10.0, 20.0, 30.0],
            summary: summarize(&[1.0, 2.0, 10.0, 20.0, 30.0]),
        };
        assert_eq!(halves(&odd), (1.5, 20.0));
        assert_eq!(trimmed(&odd), 32.0 / 3.0);

        for (index, series) in data.series.iter_mut().enumerate() {
            series.values = (0..24).map(|x| x as f64 + index as f64 / 10.0).collect();
            series.summary = summarize(&series.values);
        }
        assert!(result_paragraphs(&data, Lens::Overlap, 0)[1].contains("intervals intersect"));
        assert!(result_paragraphs(&data, Lens::Ranking, 0)[1].contains("No strict pair reverses"));
        assert!(
            result_paragraphs(&data, Lens::Crossings, 0)[1].contains("No adjacent strict crossing")
        );
        assert!(result_paragraphs(&data, Lens::Trimming, 0)[1].contains("leader remains C"));
        for (index, series) in data.series.iter_mut().enumerate() {
            series.values = (0..24).map(|x| x as f64 + index as f64 * 100.0).collect();
            series.summary = summarize(&series.values);
        }
        assert!(result_paragraphs(&data, Lens::Intervals, 0)[1].contains("above the runner-up"));
        assert!(result_paragraphs(&data, Lens::Overlap, 0)[1].contains("are disjoint"));
        for series in &mut data.series {
            series.values = vec![4.0; 24];
            series.summary = summarize(&series.values);
        }
        assert_eq!(pair_reversals(&data, |s| s.summary.median), 0);
        assert!(result_paragraphs(&data, Lens::Quartiles, 0)[1].contains("equal to the median"));
        assert!(result_paragraphs(&data, Lens::Occupancy, 0)[1].contains("100.0%"));
        for lens in Lens::ALL {
            let text = result_paragraphs(&data, lens, 0).join(" ");
            assert!(
                !text
                    .split_whitespace()
                    .any(|word| matches!(word, "NaN" | "inf" | "-inf"))
            );
        }
    }

    #[test]
    fn fitted_trends_pair_counts_and_step_changes_match_direct_calculations() {
        let series = |values: Vec<f64>| Series {
            name: "Check".to_owned(),
            summary: summarize(&values),
            values,
        };
        let ascending = series(vec![2.0, 5.0, 8.0, 11.0, 14.0]);
        assert_eq!(line_fit(&ascending), (3.0, 0.0, 1.0));
        assert_eq!(roughness(&ascending), 3.0);
        let descending = series(vec![14.0, 11.0, 8.0, 5.0, 2.0]);
        assert_eq!(line_fit(&descending), (-3.0, 0.0, 1.0));
        assert_eq!(line_fit(&series(vec![4.0; 6])), (0.0, 0.0, 0.0));
        let data = fixture();
        let (slope, residual, explained) = line_fit(&data.series[2]);
        assert!((slope - 50.0 / 7.0).abs() < 1e-12);
        assert!((residual.powi(2) - 6250.0 / 21.0).abs() < 1e-10);
        assert!((explained - 3.0 / 7.0).abs() < 1e-12);
        assert_eq!(roughness(&data.series[0]), 6.0);
        assert_eq!(response_pairs(&data.series[0], &data.series[1]), (6, 30, 0));
        let a = series(vec![1.0, 2.0, 2.0, 4.0]);
        let b = series(vec![2.0, 3.0, 4.0]);
        assert_eq!(response_pairs(&a, &b), (2, 7, 3));
        let all = [a, b, ascending, descending];
        for a in &all {
            for b in &all {
                let greater = a
                    .values
                    .iter()
                    .flat_map(|x| b.values.iter().map(move |y| x > y))
                    .filter(|v| *v)
                    .count();
                let less = a
                    .values
                    .iter()
                    .flat_map(|x| b.values.iter().map(move |y| x < y))
                    .filter(|v| *v)
                    .count();
                assert_eq!(
                    response_pairs(a, b),
                    (
                        greater,
                        less,
                        a.values.len() * b.values.len() - greater - less
                    )
                );
            }
        }
    }

    #[test]
    fn new_diagnostics_report_reversals_separation_and_ties_without_inference() {
        let mut data = fixture();
        for voice in 0..4 {
            let influence = diagnostic_paragraphs(&data, Lens::PointInfluence, voice).join(" ");
            assert!(influence.contains("0.83") && influence.contains("-7.50"));
            assert!(influence.contains("strict pair order reverses"));
        }
        data.series[0].values = vec![0.0, 1.0, 2.0, 3.0, 100.0, 101.0];
        data.series[0].summary = summarize(&data.series[0].values);
        let quartiles = diagnostic_paragraphs(&data, Lens::QuantileRanks, 0).join(" ");
        assert!(quartiles.contains("6.25") && quartiles.contains("75.75"));
        assert!(quartiles.contains("quartile leader changes"));
        data.series[0].values = vec![0.0, 0.0, 0.0, 0.0, 1.0, 4.0];
        data.series[1].values = vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        for s in &mut data.series {
            s.summary = summarize(&s.values);
        }
        let reciprocal = diagnostic_paragraphs(&data, Lens::ReferenceBand, 0)[1].clone();
        assert!(reciprocal.contains("Both bands have width 0.75"));
        assert!(reciprocal.contains("counts differ by 1"));
        for (index, series) in data.series.iter_mut().enumerate() {
            series.values = (0..6)
                .map(|step| 100.0 * index as f64 + step as f64)
                .collect();
            series.summary = summarize(&series.values);
        }
        assert!(diagnostic_paragraphs(&data, Lens::PairOrder, 0)[1].contains("strictly below"));
        assert!(diagnostic_paragraphs(&data, Lens::PointInfluence, 0)[1].contains("not reversed"));
        data.series.swap(0, 1);
        assert!(diagnostic_paragraphs(&data, Lens::PairOrder, 0)[1].contains("strictly separated"));
        for series in &mut data.series {
            series.values = vec![4.0; 6];
            series.summary = summarize(&series.values);
        }
        assert_eq!(response_pairs(&data.series[0], &data.series[1]), (0, 0, 36));
        assert!(
            diagnostic_paragraphs(&data, Lens::ReferenceBand, 0)[1]
                .contains("counts are equal at 6")
        );
        assert!(diagnostic_paragraphs(&data, Lens::Detrending, 0)[0].contains("0.0%"));
        let balance = result_paragraphs(&fixture(), Lens::Balance, 0)[0].clone();
        assert!(balance.contains("count strictly above this center is 1 out of 3 configurations"));
        assert!(!balance.contains("1 lie"));
    }

    #[test]
    fn cross_setting_interpretation_tracks_runs_excursions_and_pair_flips() {
        let first = fixture();
        let mut changed = first.clone();
        changed.series[0].values = vec![100.0; 6];
        changed.series[0].summary = summarize(&changed.series[0].values);
        let windows = vec![first.clone(), first.clone(), first.clone(), changed.clone()];
        let run = synthesis(&windows, 0)[1].clone();
        assert!(run.contains("C has the longest uninterrupted lead"));
        assert!(run.contains("windows 1–3"));
        assert!(run.contains("count of leading windows is 0"));
        let alternating = vec![first.clone(), changed.clone(), first, changed];
        let pairs = synthesis(&alternating, 2)[1].clone();
        assert!(pairs.contains("A versus B"));
        assert!(pairs.contains("reversal count: 3"));
        assert!(pairs.contains("higher mean in 2 windows and the lower mean in 2"));
        assert_eq!(
            rank_histories(&[vec![2, 1, 0], vec![0, 2, 1]]),
            vec![vec![3, 1], vec![2, 3], vec![1, 2]]
        );
        let excursion = synthesis(&alternating, 1)[1].clone();
        assert!(excursion.contains("A has a largest rank excursion"));
        assert!(excursion.contains("3 → 1 → 3 → 1"));
        let joint = synthesis(&alternating, 3)[1].clone();
        assert!(joint.contains("coincide in windows 2, 4 only"));
    }

    #[test]
    fn shared_scope_is_stated_once_instead_of_repeated_after_each_result() {
        for category in 0..CATEGORIES.len() {
            let id = PaperId {
                category,
                topic: 0,
                author: 12,
                nonce: 19,
            };
            let paper = generate(&id);
            let text = paper
                .sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(
                text.matches("Mean intervals use mean ± 1.96 × sample SD / √n.")
                    .count(),
                1
            );
            assert!(
                text.contains("neither calibrated confidence coverage nor a test of significance")
            );
            assert!(
                text.contains(
                    "names denote neither implemented methods nor controlled interventions"
                )
            );
            assert!(!text.contains(
                "Endpoint reversals and adjacent leader changes answer different questions"
            ));
            assert!(
                !text.contains("Offsets, slopes, lengths, and deviations are redrawn together")
            );
        }
    }

    #[test]
    fn synthesis_distinguishes_leadership_from_full_rank_persistence() {
        let first = fixture();
        for voice in 0..4 {
            let same = synthesis(&[first.clone(), first.clone()], voice).join(" ");
            assert!(same.contains("0"));
            assert!(same.contains("1"));
            let mut last = first.clone();
            last.series[0].values = vec![100.0; 6];
            last.series[0].summary = summarize(&last.series[0].values);
            let changed = synthesis(&[first.clone(), last], voice).join(" ");
            assert!(changed.contains("2"));
            assert!(changed.contains("1"));
            assert!(!changed.contains("same label leads throughout"));
        }
        let mut last = first.clone();
        last.series[0].summary = summarize(&[8.0; 6]);
        last.series[0].values = vec![8.0; 6];
        let stable_lead = synthesis(&[first, last], 0).join(" ");
        assert!(stable_lead.contains("1 strict order reversals"));
        assert!(stable_lead.contains("persistent leader coexists"));
    }

    #[test]
    fn narratives_have_independent_lenses_outlines_and_no_repeated_paragraphs() {
        let mut outlines = BTreeSet::new();
        let mut first_checks = BTreeSet::new();
        let mut introductions = BTreeSet::new();
        let mut saw_top_level_result = false;
        let mut saw_nested_method = false;
        for category in 0..CATEGORIES.len() {
            for nonce in 0..24 {
                let id = PaperId {
                    category,
                    topic: nonce as usize % 6,
                    author: 12,
                    nonce,
                };
                let paper = generate(&id);
                let mut paragraphs = BTreeSet::new();
                let mut anchors = BTreeSet::new();
                for section in &paper.sections {
                    assert!(anchors.insert(&section.anchor), "{id}: duplicate anchor");
                    for paragraph in &section.paragraphs {
                        assert!(
                            paragraphs.insert(paragraph),
                            "{id}: repeated paragraph: {paragraph}"
                        );
                    }
                    saw_top_level_result |= section.figure.is_some() && section.level == 2;
                    saw_nested_method |= section.figure.is_none()
                        && section.level == 3
                        && section.paragraphs.iter().any(|p| p.contains("quartile"));
                }
                assert_eq!(
                    paper
                        .sections
                        .iter()
                        .filter(|s| s.title == "Cross-setting comparison")
                        .count(),
                    1
                );
                let mut lens_rng = rng(&format!("{id}:narrative-lenses-v3"));
                let mut lenses = Lens::ALL;
                lenses.shuffle(&mut lens_rng);
                let results = paper
                    .sections
                    .iter()
                    .filter(|s| s.figure.is_some())
                    .collect::<Vec<_>>();
                for (index, section) in results.iter().enumerate() {
                    let voice = rng(&format!("{id}:narrative-result-v3:{index}")).gen_range(0..4);
                    assert_eq!(
                        section.paragraphs,
                        result_paragraphs(&experiment(&id, index), lenses[index], voice)
                    );
                    assert_eq!(section.figure, Some(figure(&id, index)));
                }
                first_checks.insert(lenses[0].title());
                introductions.insert(paper.sections[0].paragraphs[0].clone());
                outlines.insert(
                    paper
                        .sections
                        .iter()
                        .map(|s| (s.level, s.figure.is_some()))
                        .collect::<Vec<_>>(),
                );
                assert_eq!(paper.metadata, metadata(&id));
                assert_eq!(paper, generate(&id));
                let preview = abstract_preview(&paper.metadata).unwrap();
                assert!(
                    paper
                        .abstract_text
                        .starts_with(preview.trim_end_matches('…'))
                );
                for reference in paper.references.iter().chain(&paper.related) {
                    let target: PaperId = reference.id.parse().unwrap();
                    assert_eq!(reference, &metadata(&target));
                }
            }
        }
        assert_eq!(first_checks.len(), Lens::ALL.len());
        assert!(
            outlines.len() >= 24,
            "{} structural outlines",
            outlines.len()
        );
        assert!(introductions.len() >= 180);
        assert!(saw_top_level_result && saw_nested_method);
    }
}
