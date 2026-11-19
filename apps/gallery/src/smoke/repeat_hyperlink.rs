use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    time::{Duration, Instant},
};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    ButtonClickMode, ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow,
    HyperlinkProbe, RepeatButtonProbe, UriLauncher,
};

type SmokeResult = Result<(), Box<dyn Error>>;
type BatchResult = Result<((u32, u32), (u32, u32)), Box<dyn Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    verify_repeat(gallery)?;
    verify_hyperlink(gallery)
}

fn repeat_center(probe: &RepeatButtonProbe) -> LogicalPosition {
    LogicalPosition::new(
        probe.bounds.x + probe.bounds.width / 2.,
        probe.bounds.y + probe.bounds.height / 2.,
    )
}

fn link_center(probe: &HyperlinkProbe) -> LogicalPosition {
    LogicalPosition::new(
        probe.bounds.x + probe.bounds.width / 2.,
        probe.bounds.y + probe.bounds.height / 2.,
    )
}

fn pointer(window: &Window, position: LogicalPosition, down: bool) {
    window.dispatch_event(if down {
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        }
    } else {
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        }
    });
}

fn verify_repeat(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::RepeatButton);
    gallery.set_repeat_enabled(true);
    gallery.set_repeat_disable_on_click(false);
    gallery.set_repeat_click_mode(ButtonClickMode::Press);
    gallery.set_repeat_delay(500);
    gallery.set_repeat_interval(33);
    let window = gallery.window();
    window.take_snapshot()?;
    let center = repeat_center(&gallery.get_repeat_probe());
    let before = gallery.get_repeat_clicks();
    window.dispatch_event(WindowEvent::PointerMoved { position: center });
    pointer(window, center, true);
    window.take_snapshot()?;

    check(
        gallery.get_repeat_clicks() == before + 1
            && gallery.get_repeat_probe().repeating
            && gallery.get_repeat_probe().focused
            && !gallery.get_repeat_probe().focus_visible,
        "RepeatButton immediately clicks on pointer press and starts only its repeat timer",
    )?;
    pointer(window, center, false);
    window.take_snapshot()?;
    check(
        gallery.get_repeat_clicks() == before + 1 && !gallery.get_repeat_probe().repeating,
        "releasing a Press RepeatButton adds no click and stops scheduling",
    )?;
    gallery.invoke_focus_repeat();
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    window.take_snapshot()?;
    check(
        gallery.get_repeat_clicks() == before + 2
            && gallery.get_repeat_probe().repeating
            && gallery.get_repeat_probe().focus_visible,
        "Space presses once, starts repeating, and uses keyboard focus visuals",
    )?;
    window.dispatch_event(WindowEvent::KeyPressRepeated {
        text: Key::Space.into(),
    });
    check(
        gallery.get_repeat_clicks() == before + 2,
        "OS key autorepeat does not duplicate the timer's clicks",
    )?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    window.take_snapshot()?;
    check(
        !gallery.get_repeat_probe().repeating,
        "Space release stops the repeat timer",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.take_snapshot()?;
    check(
        gallery.get_repeat_clicks() == before + 3 && !gallery.get_repeat_probe().repeating,
        "Enter presses once without the Space repeat timer",
    )?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    check(
        gallery.get_repeat_clicks() == before + 3,
        "Enter release does not double-activate",
    )?;

    gallery.set_repeat_click_mode(ButtonClickMode::Release);
    pointer(window, center, true);
    check(
        gallery.get_repeat_clicks() == before + 3,
        "Release mode defers the initial action",
    )?;
    pointer(window, center, false);
    window.take_snapshot()?;
    check(
        gallery.get_repeat_clicks() == before + 4,
        "Release mode activates once on release",
    )?;

    gallery.set_repeat_click_mode(ButtonClickMode::Press);
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    key(window, Key::Escape);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    window.take_snapshot()?;
    check(
        !gallery.get_repeat_probe().repeating,
        "an unrelated key cancels the held keyboard press",
    )?;

    gallery.set_repeat_disable_on_click(true);
    pointer(window, center, true);
    window.take_snapshot()?;
    check(
        !gallery.get_repeat_enabled() && !gallery.get_repeat_probe().repeating,
        "disabling inside Click cancels before a timer can start",
    )?;
    gallery.set_repeat_disable_on_click(false);
    gallery.set_repeat_enabled(true);
    window.take_snapshot()?;
    check(
        !gallery.get_repeat_probe().repeating,
        "re-enabling does not resurrect the canceled press",
    )?;
    pointer(window, center, false);
    gallery.set_repeat_interval(0);
    pointer(window, center, true);
    window.take_snapshot()?;
    check(
        !gallery.get_repeat_probe().timing_valid && !gallery.get_repeat_probe().repeating,
        "invalid timing cannot create an idle busy loop",
    )?;
    pointer(window, center, false);
    gallery.set_repeat_interval(33);
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        window.dispatch_event(WindowEvent::PointerMoved {
            position: LogicalPosition::new(1., 1.),
        });
        gallery.invoke_leave_repeat();
        window.take_snapshot()?;
        let probe = gallery.get_repeat_probe();
        let sample = LogicalPosition::new(
            probe.bounds.x + 4.,
            probe.bounds.y + probe.bounds.height / 2.,
        );
        let normal = read_rgb(window, sample)?;
        window.dispatch_event(WindowEvent::PointerMoved { position: center });
        let hovered = read_rgb(window, sample)?;
        pointer(window, center, true);
        let pressed = read_rgb(window, sample)?;
        if scheme == FluentColorScheme::HighContrast {
            check(
                normal == hovered && normal == pressed,
                "RepeatButton preserves ButtonFace across high-contrast states",
            )?;
        } else {
            check(
                normal != hovered && hovered != pressed,
                "RepeatButton renders normal, hover, and pressed fills",
            )?;
        }
        pointer(window, center, false);
        gallery.set_repeat_enabled(false);
        window.take_snapshot()?;
        check(
            gallery.get_repeat_probe().state == ButtonState::Disabled
                && !gallery.get_repeat_probe().focus_visible,
            "disabled RepeatButton suppresses input and focus feedback",
        )?;
        gallery.set_repeat_enabled(true);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_hyperlink(gallery: &GalleryWindow) -> SmokeResult {
    let records = Rc::new(RefCell::new(Vec::new()));
    let requests = Rc::clone(&records);
    let weak = gallery.as_weak();
    gallery.global::<UriLauncher>().on_open(move |uri| {
        if let Some(gallery) = weak.upgrade() {
            requests
                .borrow_mut()
                .push((uri.to_string(), gallery.get_hyperlink_clicks()));
        }
        SharedString::default()
    });
    gallery.set_page(GalleryPage::HyperlinkButton);
    gallery.set_hyperlink_enabled(true);
    gallery.set_hyperlink_long_label(false);
    gallery.set_hyperlink_suppress_uri(false);
    gallery.set_hyperlink_uri("https://www.microsoft.com".into());
    let window = gallery.window();
    window.take_snapshot()?;
    let center = link_center(&gallery.get_hyperlink_probe());
    let before = gallery.get_hyperlink_clicks();
    let requests_before = gallery.get_hyperlink_requests();
    pointer(window, center, true);
    check(
        gallery.get_hyperlink_clicks() == before,
        "HyperlinkButton waits for release",
    )?;
    pointer(window, center, false);
    window.take_snapshot()?;
    check(
        gallery.get_hyperlink_clicks() == before + 1
            && gallery.get_hyperlink_requests() == requests_before + 1
            && records.borrow().last()
                == Some(&("https://www.microsoft.com".to_owned(), before + 1))
            && !gallery.get_hyperlink_probe().focus_visible,
        "Click precedes URI host dispatch and pointer focus has no keyboard ring",
    )?;
    gallery.set_hyperlink_suppress_uri(true);
    click(window, center);
    check(
        gallery.get_hyperlink_clicks() == before + 2
            && gallery.get_hyperlink_requests() == requests_before + 1,
        "Click can clear NavigateUri before the URI is read",
    )?;
    gallery.set_hyperlink_suppress_uri(false);
    gallery.set_hyperlink_uri("".into());
    gallery.invoke_focus_hyperlink();
    key(window, Key::Space);
    key(window, Key::Return);
    check(
        gallery.get_hyperlink_clicks() == before + 4
            && gallery.get_hyperlink_requests() == requests_before + 1
            && gallery.get_hyperlink_probe().focus_visible,
        "Space and Enter activate URI-free links with keyboard focus",
    )?;
    let before_drag = gallery.get_hyperlink_clicks();
    pointer(window, center, true);
    let outside = LogicalPosition::new(1., 1.);
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    pointer(window, outside, false);
    check(
        gallery.get_hyperlink_clicks() == before_drag,
        "dragging outside cancels hyperlink activation",
    )?;
    gallery.set_hyperlink_enabled(false);
    click(window, center);
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_hyperlink_clicks() == before_drag
            && gallery.get_hyperlink_probe().state == ButtonState::Disabled,
        "disabled hyperlink does not click or navigate",
    )?;
    gallery.set_hyperlink_enabled(true);
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.invoke_leave_hyperlink();
        window.dispatch_event(WindowEvent::PointerMoved { position: outside });
        window.take_snapshot()?;
        let normal = gallery.get_hyperlink_probe();
        let normal_ink = count_link_ink(gallery)?;
        window.dispatch_event(WindowEvent::PointerMoved { position: center });
        window.take_snapshot()?;
        let over = gallery.get_hyperlink_probe();
        check(
            count_link_ink(gallery)? > 3 && normal_ink > 3,
            "link text uses its actual accent foreground in the rendered buffer",
        )?;
        pointer(window, center, true);
        window.take_snapshot()?;
        let pressed = gallery.get_hyperlink_probe();
        let expected = if scheme == FluentColorScheme::HighContrast {
            gallery.global::<FluentTheme>().get_contrast_highlight()
        } else {
            gallery
                .global::<FluentTheme>()
                .get_accent_foreground_pressed()
        };
        check(
            count_link_ink(gallery)? > 3 && pressed.foreground == expected,
            "pressed hyperlink renders the tertiary text color",
        )?;
        pointer(window, center, false);
        check(
            normal.underline == (scheme == FluentColorScheme::HighContrast)
                && (normal.foreground != over.foreground || scheme == FluentColorScheme::Dark),
            "hyperlink color and underline follow the official theme state mapping",
        )?;
        gallery.set_hyperlink_enabled(false);
        window.take_snapshot()?;
        check(
            gallery.get_hyperlink_probe().foreground != normal.foreground
                && count_link_ink(gallery)? > 3,
            "disabled hyperlink uses its disabled text color",
        )?;
        gallery.set_hyperlink_enabled(true);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_hyperlink_uri("https://www.microsoft.com".into());
    gallery
        .global::<UriLauncher>()
        .on_open(|_| "The link could not be opened.".into());
    click(window, center);
    check(
        gallery.get_hyperlink_status() == "The link could not be opened.",
        "a URI host failure remains visible to the user",
    )?;
    gallery.invoke_focus_hyperlink();
    key(window, Key::Tab);
    key(window, Key::Return);
    check(
        gallery.get_page() == GalleryPage::Toggles,
        "URI-free hyperlink navigates to the ToggleButton page",
    )?;
    gallery.set_hyperlink_uri("https://www.microsoft.com".into());
    Ok(())
}

fn count_link_ink(gallery: &GalleryWindow) -> Result<usize, Box<dyn Error>> {
    let frame = gallery.window().take_snapshot()?;
    let scale = gallery.window().scale_factor();
    let probe = gallery.get_hyperlink_probe();
    let target = probe.foreground;
    let mut count = 0;
    let background = read_background(&frame, scale, &probe);
    for y in ((probe.bounds.y + 3.) * scale).ceil() as usize
        ..((probe.bounds.y + probe.bounds.height - 3.) * scale).floor() as usize
    {
        for x in ((probe.bounds.x + 3.) * scale).ceil() as usize
            ..((probe.bounds.x + probe.bounds.width - 3.) * scale).floor() as usize
        {
            let pixel = frame.as_slice()[y * frame.width() as usize + x];
            // Disabled colors are translucent; compare their composited color through the snapshot itself below.
            if target.alpha() == 255
                && pixel.r.abs_diff(target.red()) <= 3
                && pixel.g.abs_diff(target.green()) <= 3
                && pixel.b.abs_diff(target.blue()) <= 3
            {
                count += 1;
            } else if target.alpha() != 255 {
                let blend = |ink: u8, fill: u8| {
                    ((ink as u16 * target.alpha() as u16
                        + fill as u16 * (255 - target.alpha() as u16))
                        / 255) as u8
                };
                if pixel.r.abs_diff(blend(target.red(), background.0)) <= 3
                    && pixel.g.abs_diff(blend(target.green(), background.1)) <= 3
                    && pixel.b.abs_diff(blend(target.blue(), background.2)) <= 3
                {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn read_background(
    frame: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    scale: f32,
    probe: &HyperlinkProbe,
) -> (u8, u8, u8) {
    let x = ((probe.bounds.x + 2.) * scale) as usize;
    let y = ((probe.bounds.y + probe.bounds.height / 2.) * scale) as usize;
    let pixel = frame.as_slice()[y * frame.width() as usize + x];
    (pixel.r, pixel.g, pixel.b)
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::RepeatButton);
    gallery.window().take_snapshot()?;
    let probe = gallery.get_repeat_probe();
    check(
        probe.bounds.x >= 0.
            && probe.bounds.x + probe.bounds.width <= width
            && probe.bounds.height >= 44.,
        "compact RepeatButton fits and retains its touch target",
    )?;
    gallery.set_page(GalleryPage::HyperlinkButton);
    gallery.set_hyperlink_long_label(true);
    gallery.window().take_snapshot()?;
    let probe = gallery.get_hyperlink_probe();
    check(
        probe.bounds.x >= 0.
            && probe.bounds.x + probe.bounds.width <= width
            && probe.bounds.height > 44.,
        "compact HyperlinkButton wraps a long label without clipping",
    )?;
    gallery.set_hyperlink_long_label(false);
    Ok(())
}

pub(super) fn verify_batch(gallery: &GalleryWindow, done: impl FnOnce(BatchResult) + 'static) {
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    gallery.set_shell_pane_visible(false);
    if let Err(error) = verify(gallery).and_then(|()| {
        gallery.global::<FluentTheme>().set_ui_scale(1.);
        verify(gallery)
    }) {
        done(Err(error));
        return;
    }
    let size = gallery.window().size();
    let wide = (size.width, size.height);
    gallery.set_touch_targets(true);
    gallery.window().set_size(LogicalSize::new(320., 640.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            done(Err(
                "Gallery closed during compact button verification".into()
            ));
            return;
        };
        if let Err(error) = verify_compact(&gallery, 320.) {
            done(Err(error));
            return;
        }
        let size = gallery.window().size();
        let compact = (size.width, size.height);
        verify_timing(&gallery, move |result| {
            done(result.map(|()| (wide, compact)))
        });
    });
}

pub(super) fn verify_timing(gallery: &GalleryWindow, done: impl FnOnce(SmokeResult) + 'static) {
    let trace = Rc::new(RefCell::new(TimingTrace::default()));
    let events = Rc::clone(&trace);
    gallery.on_repeat_clicked(move || {
        events.borrow_mut().clicks.push(Instant::now());
    });
    timing_step(gallery.as_weak(), 0, 0, trace, done);
}

#[derive(Default)]
struct TimingTrace {
    clicks: Vec<Instant>,
    resumed_at: Option<Instant>,
}

fn first_delay(trace: &TimingTrace, resumed: bool, minimum_ms: u64, required: bool) -> SmokeResult {
    let elapsed = if resumed {
        trace
            .resumed_at
            .zip(trace.clicks.first())
            .map(|(start, click)| click.saturating_duration_since(start))
    } else {
        trace
            .clicks
            .first()
            .zip(trace.clicks.get(1))
            .map(|(initial, repeated)| repeated.saturating_duration_since(*initial))
    };
    if let Some(elapsed) = elapsed {
        if elapsed < Duration::from_millis(minimum_ms.saturating_sub(5)) {
            return Err(format!(
                "RepeatButton ticked after {:.2}ms, before its {minimum_ms}ms delay",
                elapsed.as_secs_f64() * 1000.
            )
            .into());
        }
        if required {
            println!(
                "RepeatButton observed first tick after {:.2}ms (requested {minimum_ms}ms)",
                elapsed.as_secs_f64() * 1000.
            );
        }
    } else if required {
        return Err("RepeatButton did not produce its delayed callback".into());
    }
    Ok(())
}

fn timing_step(
    weak: slint::Weak<GalleryWindow>,
    step: u8,
    baseline: i32,
    trace: Rc<RefCell<TimingTrace>>,
    done: impl FnOnce(SmokeResult) + 'static,
) {
    let Some(gallery) = weak.upgrade() else {
        done(Err(
            "Gallery closed during RepeatButton timing verification".into(),
        ));
        return;
    };
    let window = gallery.window();
    let next = (|| {
        let center = repeat_center(&gallery.get_repeat_probe());
        let outside = LogicalPosition::new(1., 1.);
        let mut delay = 150;
        match step {
            0 => {
                gallery.set_page(GalleryPage::RepeatButton);
                gallery.set_repeat_enabled(true);
                gallery.set_repeat_delay(500);
                gallery.set_repeat_interval(33);
                gallery.set_repeat_click_mode(ButtonClickMode::Press);
                window.take_snapshot()?;
                let center = repeat_center(&gallery.get_repeat_probe());
                window.dispatch_event(WindowEvent::PointerMoved { position: center });
                trace.borrow_mut().clicks.clear();
                pointer(window, center, true);
                delay = 90;
            }
            1 => {
                first_delay(&trace.borrow(), false, 500, false)?;
                gallery.set_repeat_delay(900);
                gallery.set_repeat_interval(40);
                delay = 490;
            }
            2 => {
                check(
                    gallery.get_repeat_clicks() > baseline && gallery.get_repeat_probe().repeating,
                    "the pending Delay is stable while timing properties change; repeats follow it",
                )?;
                first_delay(&trace.borrow(), false, 500, true)?;
                gallery.set_repeat_delay(180);
                window.dispatch_event(WindowEvent::PointerMoved { position: outside });
            }
            3 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "moving out pauses the repeat timer",
                )?;
                {
                    let mut trace = trace.borrow_mut();
                    trace.clicks.clear();
                    trace.resumed_at = Some(Instant::now());
                }
                window.dispatch_event(WindowEvent::PointerMoved { position: center });
                delay = 90;
            }
            4 => {
                first_delay(&trace.borrow(), true, 180, false)?;
                delay = 200;
            }
            5 => {
                check(
                    gallery.get_repeat_clicks() > baseline,
                    "re-entering resumes repeated actions after Delay",
                )?;
                first_delay(&trace.borrow(), true, 180, true)?;
                pointer(window, center, false);
            }
            6 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "release stops all subsequent ticks",
                )?;
                gallery.invoke_focus_repeat();
                trace.borrow_mut().clicks.clear();
                window.dispatch_event(WindowEvent::KeyPressed {
                    text: Key::Space.into(),
                });
                delay = 280;
            }
            7 => {
                check(
                    gallery.get_repeat_clicks() > baseline,
                    "Space uses the same delayed repeat cadence",
                )?;
                first_delay(&trace.borrow(), false, 180, true)?;
                window.dispatch_event(WindowEvent::KeyReleased {
                    text: Key::Space.into(),
                });
            }
            8 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "Space release leaves no timer running",
                )?;
                pointer(window, center, true);
                gallery.invoke_leave_repeat();
                delay = 280;
            }
            9 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "focus loss cancels pointer repetition",
                )?;
                pointer(window, center, false);
                window.dispatch_event(WindowEvent::PointerMoved { position: outside });
                gallery.set_repeat_click_mode(ButtonClickMode::Hover);
                window.take_snapshot()?;
                window.dispatch_event(WindowEvent::PointerMoved { position: center });
                window.take_snapshot()?;
                delay = 280;
            }
            10 => {
                check(
                    gallery.get_repeat_clicks() > baseline && gallery.get_repeat_probe().repeating,
                    "Hover mode repeats without a held pointer button",
                )?;
                window.dispatch_event(WindowEvent::PointerMoved { position: outside });
                delay = 280;
            }
            11 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "leaving Hover mode stops all ticks",
                )?;
                gallery.set_repeat_click_mode(ButtonClickMode::Press);
                window.dispatch_event(WindowEvent::PointerMoved { position: center });
                pointer(window, center, true);
                gallery.set_repeat_enabled(false);
                window.take_snapshot()?;
                gallery.set_repeat_enabled(true);
                delay = 280;
            }
            12 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "disable then re-enable cannot resume the old hold",
                )?;
                pointer(window, center, false);
                pointer(window, center, true);
                gallery.set_page(GalleryPage::Button);
                delay = 280;
            }
            13 => {
                check(
                    gallery.get_repeat_clicks() == baseline
                        && !gallery.get_repeat_probe().repeating,
                    "leaving the page stops a captured hold",
                )?;
                pointer(window, outside, false);
                gallery.set_repeat_delay(500);
                gallery.set_repeat_interval(33);
                gallery.on_repeat_clicked(|| {});
                return Ok::<_, Box<dyn Error>>(None);
            }
            _ => return Err("invalid RepeatButton test step".into()),
        }
        // Rendering can itself exceed the initial Delay under a full-suite workload.
        // Observe callback Instants rather than postponing checks behind another render.
        Ok(Some((delay, gallery.get_repeat_clicks())))
    })();
    match next {
        Ok(Some((delay, baseline))) => {
            slint::Timer::single_shot(Duration::from_millis(delay), move || {
                timing_step(weak, step + 1, baseline, trace, done)
            })
        }
        Ok(None) => done(Ok(())),
        Err(error) => done(Err(error)),
    }
}
