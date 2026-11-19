use std::{
    cell::{Cell, RefCell},
    error::Error,
    process::Command,
    rc::Rc,
    time::{Duration, Instant},
};

use slint::{
    ComponentHandle, LogicalPosition, Timer, TimerMode,
    platform::{Key, WindowEvent},
};

use crate::{FluentTheme, GalleryPage, GalleryWindow, NavigationPaneDisplayMode, smoke::click};

const ROUTES: [i32; 3] = [1, 3, 0];
const WARMUP: usize = 300;
const SWITCHES: usize = 2100;

pub(super) fn profile_command_bar(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    const STEPS: usize = 32;
    if !cfg!(feature = "profile-graphics") {
        return Err("Build with --features profile-graphics to profile Metal frames".into());
    }
    gallery.set_page(GalleryPage::CommandBar);
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduce_transparency(false);
    theme.set_reduced_motion(false);

    let started = Rc::new(Cell::new(None::<Instant>));
    let previous_frame = Rc::new(Cell::new(None::<Instant>));
    let first_frames = Rc::new(RefCell::new(Vec::new()));
    let frame_intervals = Rc::new(RefCell::new(Vec::new()));
    let frame_started = Rc::clone(&started);
    let last_frame = Rc::clone(&previous_frame);
    let frames = Rc::clone(&first_frames);
    let intervals = Rc::clone(&frame_intervals);
    gallery.window().set_rendering_notifier(move |state, _| {
        if matches!(state, slint::RenderingState::AfterRendering)
            && let Some(start) = frame_started.get()
        {
            let now = Instant::now();
            if let Some(previous) = last_frame.replace(Some(now)) {
                intervals.borrow_mut().push((now - previous).as_micros());
            } else {
                frames.borrow_mut().push((now - start).as_micros());
            }
        }
    })?;

    let weak = gallery.as_weak();
    let result = Rc::new(RefCell::new(None));
    let outcome = Rc::clone(&result);
    let mut step: usize = 0;
    let mut input = Vec::new();
    let mut tick = move || -> Result<bool, Box<dyn Error>> {
        let gallery = weak
            .upgrade()
            .ok_or("Gallery closed while profiling CommandBar")?;
        let window = gallery.window();
        if step == 0 {
            let size = window.size();
            println!(
                "command-bar-profile backing={}x{} scale={}x",
                size.width,
                size.height,
                window.scale_factor()
            );
        }
        if step == STEPS {
            for (label, samples) in [
                ("input", &mut *input),
                ("first-frame", &mut *first_frames.borrow_mut()),
                ("opening-frame-interval", &mut *frame_intervals.borrow_mut()),
            ] {
                samples.sort_unstable();
                if !samples.is_empty() {
                    let p95 = samples.len().saturating_mul(95).div_ceil(100) - 1;
                    println!(
                        "{label} samples={} p50_us={} p95_us={} max_us={}",
                        samples.len(),
                        samples[samples.len() / 2],
                        samples[p95],
                        samples[samples.len() - 1]
                    );
                }
            }
            if first_frames.borrow().len() != (STEPS - 4) / 2 {
                return Err("A CommandBar click did not render an open frame".into());
            }
            return Ok(true);
        }
        if step.is_multiple_of(2) {
            let target = gallery.get_command_probe().more;
            let start = Instant::now();
            click(window, target);
            if !gallery.get_command_probe().open {
                return Err("More commands did not open".into());
            }
            if step >= 4 {
                input.push(start.elapsed().as_micros());
                previous_frame.set(None);
                started.set(Some(start));
            }
        } else {
            started.set(None);
            window.dispatch_event(WindowEvent::KeyPressed {
                text: Key::Escape.into(),
            });
            window.dispatch_event(WindowEvent::KeyReleased {
                text: Key::Escape.into(),
            });
        }
        step += 1;
        Ok(false)
    };
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(450), move || {
        let finished = match tick() {
            Ok(false) => return,
            Ok(true) => Ok(()),
            Err(error) => Err(error),
        };
        outcome.replace(Some(finished));
        let _ = slint::quit_event_loop();
    });
    gallery.run()?;
    timer.stop();
    result.take().ok_or("CommandBar profiling did not finish")?
}

slint::slint! {
    #[edition = "1.19"]
    export component RendererProbe inherits Window {
        background: #f3f3f3;
        Text { text: "Skia"; color: #1a1a1a; }
    }
}

/// Measures the same native backend at Gallery resolution without loading SDK controls.
pub(super) fn verify_renderer() -> Result<(), Box<dyn Error>> {
    let probe = RendererProbe::new()?;
    probe.window().set_size(slint::LogicalSize::new(980., 700.));
    let weak = probe.as_weak();
    let outcome = Rc::new(RefCell::new(None));
    let result = Rc::clone(&outcome);
    Timer::single_shot(Duration::from_secs(2), move || {
        result.replace(Some((|| {
            let probe = weak
                .upgrade()
                .ok_or("Renderer probe closed before sampling")?;
            let size = probe.window().size();
            println!(
                "renderer-memory backing={}x{} scale={}x",
                size.width,
                size.height,
                probe.window().scale_factor()
            );
            let report = Command::new("/usr/bin/footprint")
                .args(["--format", "bytes", "-p", &std::process::id().to_string()])
                .output()?;
            if !report.status.success() {
                return Err("macOS footprint could not sample the renderer probe".into());
            }
            print!("{}", String::from_utf8(report.stdout)?);
            Ok(())
        })()));
        let _ = slint::quit_event_loop();
    });
    probe.run()?;
    outcome.take().ok_or("Renderer probe did not finish")?
}

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::NavigationView);
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Left);
    let weak = gallery.as_weak();
    let result = Rc::new(RefCell::new(None));
    let outcome = Rc::clone(&result);
    let timer = Timer::default();
    let ready = Instant::now() + Duration::from_millis(250);
    let mut switches: usize = 0;
    let mut step = move || -> Result<bool, Box<dyn Error>> {
        let gallery = weak.upgrade().ok_or("Gallery closed during navigation")?;
        if switches == 0 {
            let window = gallery.window();
            let size = window.size();
            println!(
                "navigation-memory pid={} backing={}x{} scale={}x mode=Left routes=Library,Mail,Home",
                std::process::id(),
                size.width,
                size.height,
                window.scale_factor()
            );
        }
        if switches.is_multiple_of(WARMUP) {
            let snapshot = gallery.window().take_snapshot()?;
            let size = gallery.window().size();
            if snapshot.width() != size.width || snapshot.height() != size.height {
                return Err("Could not render the NavigationView frame at the window size".into());
            }
            drop(snapshot);
            print_sample(switches, gallery.get_navigation_history_length())?;
        }
        if switches == SWITCHES {
            if gallery.get_navigation_history_length() != 64 {
                return Err(
                    "NavigationView Frame journal exceeded or missed its 64-entry cap".into(),
                );
            }
            return Ok(true);
        }

        let index = ROUTES[switches % ROUTES.len()];
        let nav = gallery.get_navigation_probe();
        if !nav.pane_open || nav.item_step <= 0. {
            return Err("NavigationView entries are outside the open pane".into());
        }
        click(
            gallery.window(),
            LogicalPosition::new(
                nav.items.x + 24.,
                nav.items.y + nav.item_step * (index as f32 + 0.5),
            ),
        );
        let nav = gallery.get_navigation_probe();
        if gallery.get_page() != GalleryPage::NavigationView
            || nav.selected != index
            || nav.route != index
        {
            return Err(format!(
                "NavigationView switch {} expected {index}, got selection {} and Frame route {}",
                switches + 1,
                nav.selected,
                nav.route
            )
            .into());
        }
        gallery.window().request_redraw();
        switches += 1;
        Ok(false)
    };
    timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        if Instant::now() < ready || outcome.borrow().is_some() {
            return;
        }
        match step() {
            Ok(false) => {}
            Ok(true) => {
                outcome.replace(Some(Ok(())));
                let _ = slint::quit_event_loop();
            }
            Err(error) => {
                outcome.replace(Some(Err(error)));
                let _ = slint::quit_event_loop();
            }
        }
    });
    gallery.run()?;
    timer.stop();
    result
        .take()
        .ok_or("Gallery closed before its NavigationView memory check completed")?
}

/// Exercises the real popup backdrop on Metal without issuing test snapshots.
pub(super) fn verify_acrylic(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::NavigationView);
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::LeftCompact);
    gallery
        .global::<FluentTheme>()
        .set_reduce_transparency(false);
    let weak = gallery.as_weak();
    let outcome = Rc::new(RefCell::new(None));
    let result = Rc::clone(&outcome);
    let timer = Timer::default();
    let mut step: usize = 0;
    let mut latencies = Vec::with_capacity(32);
    let mut run_step = move || -> Result<bool, Box<dyn Error>> {
        let gallery = weak
            .upgrade()
            .ok_or("Gallery closed during Acrylic profiling")?;
        let window = gallery.window();
        if step == 0 {
            let size = window.size();
            println!(
                "acrylic-memory pid={} backing={}x{} scale={}x",
                std::process::id(),
                size.width,
                size.height,
                window.scale_factor()
            );
            print_sample(0, gallery.get_navigation_history_length())?;
        }
        if step == 72 {
            latencies.sort_unstable();
            println!(
                "acrylic-open p50_us={} p95_us={}",
                latencies[16], latencies[30]
            );
            print_sample(36, gallery.get_navigation_history_length())?;
            return Ok(true);
        }
        if step.is_multiple_of(2) {
            let toggle = gallery.get_navigation_probe().toggle;
            let started = Instant::now();
            click(window, toggle);
            if step >= 8 {
                latencies.push(started.elapsed().as_micros());
            }
            let opened = gallery.get_navigation_probe();
            if !opened.drawer_open || !opened.drawer_acrylic {
                return Err("NavigationView drawer did not open with Acrylic".into());
            }
        } else {
            if step % 12 == 1 {
                print_sample(step / 2 + 1, gallery.get_navigation_history_length())?;
            }
            window.dispatch_event(WindowEvent::KeyPressed {
                text: Key::Escape.into(),
            });
            window.dispatch_event(WindowEvent::KeyReleased {
                text: Key::Escape.into(),
            });
        }
        step += 1;
        Ok(false)
    };
    timer.start(TimerMode::Repeated, Duration::from_millis(300), move || {
        if result.borrow().is_some() {
            return;
        }
        let finished = match run_step() {
            Ok(false) => return,
            Ok(true) => Ok(()),
            Err(error) => Err(error),
        };
        result.replace(Some(finished));
        let _ = slint::quit_event_loop();
    });
    gallery.run()?;
    timer.stop();
    outcome
        .take()
        .ok_or("Gallery closed before Acrylic profiling completed")?
}

fn print_sample(switches: usize, history: i32) -> Result<(), Box<dyn Error>> {
    let pid = std::process::id().to_string();
    let report = Command::new("/usr/bin/footprint")
        .args(["--noCategories", "--format", "bytes", "-p", &pid])
        .output()?;
    let report_text = String::from_utf8(report.stdout)?;
    let physical: u64 = report_text
        .lines()
        .find_map(|line| line.trim().strip_prefix("phys_footprint: "))
        .filter(|_| report.status.success())
        .and_then(|value| value.split_whitespace().next())
        .ok_or_else(|| format!("footprint did not report physical memory for PID {pid}"))?
        .parse()?;
    let peak = report_text
        .lines()
        .find_map(|line| line.trim().strip_prefix("phys_footprint_peak: "))
        .and_then(|value| value.split_whitespace().next())
        .unwrap_or("unavailable");
    let rss = Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()?;
    let resident: u64 = String::from_utf8(rss.stdout)?.trim().parse()?;
    println!(
        "navigation-memory switches={switches} physical_bytes={physical} peak_bytes={peak} resident_kib={resident} history={history}"
    );
    Ok(())
}
