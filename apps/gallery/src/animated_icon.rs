use slint::{Model, ModelRc, SharedString, VecModel};

use crate::{AnimationMarker, AnimationSegment, ChevronPose, chevron_tracks as tracks};

pub(super) fn markers() -> ModelRc<AnimationMarker> {
    ModelRc::new(VecModel::from(
        tracks::MARKERS
            .into_iter()
            .map(|(name, progress)| AnimationMarker {
                name: name.into(),
                progress: progress as f32,
            })
            .collect::<Vec<_>>(),
    ))
}

pub(super) const DURATION_MS: u64 = tracks::DURATION_MS as u64;

pub(super) fn segment(
    markers: ModelRc<AnimationMarker>,
    from: SharedString,
    to: SharedString,
) -> AnimationSegment {
    let lookup = |name: &str| {
        markers
            .iter()
            .find(|marker| marker.name.as_str() == name)
            .map(|marker| marker.progress)
    };
    let transition = format!("{from}To{to}");
    match (
        lookup(&format!("{transition}_Start")),
        lookup(&format!("{transition}_End")),
    ) {
        (Some(start), Some(end)) => AnimationSegment { start, end },
        (start, end) => {
            let suffix = format!("To{to}_End");
            let frame = end
                .or(start)
                .or_else(|| lookup(&transition))
                .or_else(|| lookup(to.as_str()))
                .or_else(|| {
                    markers
                        .iter()
                        .find(|marker| marker.name.ends_with(&suffix))
                        .map(|marker| marker.progress)
                })
                .or_else(|| to.parse::<f32>().ok())
                .filter(|frame| frame.is_finite())
                .unwrap_or(0.)
                .clamp(0., 1.);
            AnimationSegment {
                start: frame,
                end: frame,
            }
        }
    }
}

pub(super) fn queue(states: ModelRc<SharedString>, state: SharedString) -> ModelRc<SharedString> {
    let mut items: Vec<_> = states.iter().collect();
    items.push(state);
    ModelRc::new(VecModel::from(items))
}

pub(super) fn skip_first(states: ModelRc<SharedString>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(states.iter().skip(1).collect::<Vec<_>>()))
}

pub(super) fn chevron_pose(progress: f32) -> ChevronPose {
    let progress = f64::from(progress);
    let [rotation, _] = sample(tracks::ROTATION, progress);
    let [x, y] = sample(tracks::OFFSET, progress);
    ChevronPose {
        rotation: rotation as f32,
        x: x as f32,
        y: y as f32,
    }
}

fn sample(track: &[(f64, [f64; 2], usize)], progress: f64) -> [f64; 2] {
    let next = track.partition_point(|(position, _, _)| *position <= progress);
    let previous = track[next.saturating_sub(1)];
    let Some(&(end, to, curve)) = track.get(next) else {
        return previous.1;
    };
    if curve == tracks::CURVES.len() || next == 0 {
        return previous.1;
    }
    let weight = bezier(
        (progress - previous.0) / (end - previous.0),
        tracks::CURVES[curve],
    );
    [
        previous.1[0] + (to[0] - previous.1[0]) * weight,
        previous.1[1] + (to[1] - previous.1[1]) * weight,
    ]
}

/// WinUI's small down-chevron offsets, in units of the 48-unit artboard.
pub(super) fn split_chevron_offset(progress: f32) -> f32 {
    let frame = f64::from(progress) * 29.;
    let interpolate = |start: f64, end: f64, from: f64, to: f64, curve| {
        from + (to - from) * bezier(((frame - start) / (end - start)).clamp(0., 1.), curve)
    };
    let offset = if frame < 10. {
        interpolate(0., 9., 0., 7.5, [0.167, 0.167, 0.65, 1.])
    } else if frame <= 15. {
        interpolate(10., 15., 7.5, -3., [0.55, 0., 0.75, 1.])
    } else {
        interpolate(15., 29., -3., 0., [0.35, 0., 0., 1.])
    };
    (offset / 48.) as f32
}

fn bezier(progress: f64, [x1, y1, x2, y2]: [f64; 4]) -> f64 {
    let axis = |t: f64, a: f64, b: f64| {
        let inverse = 1. - t;
        3. * inverse * inverse * t * a + 3. * inverse * t * t * b + t * t * t
    };
    let (mut lower, mut upper) = (0., 1.);
    for _ in 0..20 {
        let mid = (lower + upper) / 2.;
        if axis(mid, x1, x2) < progress {
            lower = mid;
        } else {
            upper = mid;
        }
    }
    axis((lower + upper) / 2., y1, y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_markers_resolve_before_stationary_fallbacks() {
        let open = segment(markers(), "NormalOff".into(), "NormalOn".into());
        assert!((open.start - 0.1925).abs() < 1e-6);
        assert!(((open.end - open.start) * tracks::DURATION_MS - 483.33).abs() < 0.02);
        assert_eq!(chevron_pose(open.start).rotation, 0.);
        assert_eq!(chevron_pose(open.end).rotation, 180.);
        let turns = chevron_pose(0.27);
        assert!(turns.rotation > 0. && turns.rotation < 180. && turns.y > 24.);
        let stationary = segment(markers(), "unregistered".into(), "NormalOff".into());
        assert_eq!(stationary.start, stationary.end);
        assert_eq!(chevron_pose(stationary.end).rotation, 0.);
    }
}
