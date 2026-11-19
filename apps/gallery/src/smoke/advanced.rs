use std::{cell::Cell, error::Error, rc::Rc};

use slint::{
    Color, ComponentHandle, LogicalPosition, LogicalSize, Model, ModelRc, ModelTracker, VecModel,
    platform::{Key, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    CommandBarItem, CommandBarProbe, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow,
    ItemData, TreeLogic,
};

type SmokeResult = Result<(), Box<dyn Error>>;

pub(super) fn verify_tree(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::TreeView);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;
    let tree = gallery.get_tree_probe();
    let row = |row: usize| {
        LogicalPosition::new(
            tree.bounds.x + 170.,
            tree.bounds.y + tree.row_height * (row as f32 + 0.5),
        )
    };
    check(
        tree.count == 5 && tree.selected.row_count() == 0,
        "TreeView projects the visible roots and children",
    )?;
    click(window, row(1));
    check(
        gallery.get_tree_probe().invoked == 1,
        "clicking a tree row invokes its source node",
    )?;
    key(window, Key::RightArrow);
    window.take_snapshot()?;
    check(
        gallery.get_tree_probe().count == 6 && gallery.get_tree_probe().focused == 1,
        "Right expands the branch while retaining focus",
    )?;
    key(window, Key::RightArrow);
    key(window, Key::Return);
    check(
        gallery.get_tree_probe().focused == 2
            && gallery.get_tree_probe().selected.iter().collect::<Vec<_>>() == [2],
        "Right enters the child and Enter selects it",
    )?;
    key(window, Key::LeftArrow);
    key(window, Key::LeftArrow);
    window.take_snapshot()?;
    check(
        gallery.get_tree_probe().count == 5
            && gallery.get_tree_probe().focused == 1
            && gallery.get_tree_probe().selected.iter().collect::<Vec<_>>() == [2],
        "Left returns to the parent and collapses it while preserving hidden selection",
    )?;
    click(window, row(4));
    check(
        gallery.get_tree_probe().invoked == 2,
        "disabled tree rows ignore clicks",
    )?;

    gallery.set_tree_multiple(true);
    window.take_snapshot()?;
    click(window, LogicalPosition::new(tree.bounds.x + 50., row(2).y));
    check(
        gallery.get_tree_probe().selected.iter().collect::<Vec<_>>() == [3]
            && gallery.get_tree_probe().mixed.iter().collect::<Vec<_>>() == [0],
        "checking a leaf gives its ancestor a mixed state",
    )?;
    click(window, LogicalPosition::new(tree.bounds.x + 24., row(0).y));
    check(
        gallery.get_tree_probe().selected.iter().collect::<Vec<_>>() == [0, 1, 2, 3],
        "checking a branch includes descendants behind its collapsed child",
    )?;
    key(window, Key::Space);
    check(
        gallery.get_tree_probe().selected.row_count() == 0,
        "Space toggles the checked subtree",
    )?;
    click(window, LogicalPosition::new(tree.bounds.x + 74., row(1).y));
    window.take_snapshot()?;
    check(
        gallery.get_tree_probe().count == 6 && gallery.get_tree_probe().selected.row_count() == 0,
        "the disclosure hit region expands without toggling the checkbox",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Dark);
    let dark = read_rgb(window, LogicalPosition::new(tree.bounds.x + 7., row(2).y))?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    check(
        dark != read_rgb(window, LogicalPosition::new(tree.bounds.x + 7., row(2).y))?,
        "TreeView draws on the active theme",
    )?;

    let reads = Rc::new(Cell::new(0));
    let row_reads = Rc::clone(&reads);
    gallery
        .global::<TreeLogic>()
        .on_project(move |nodes, expanded, count| {
            let mut projection = crate::tree_view::project(nodes, expanded, count);
            projection.items = ModelRc::new(CountingRows {
                rows: projection.items,
                reads: Rc::clone(&row_reads),
            });
            projection
        });
    let long_test = (|| -> SmokeResult {
        gallery.set_tree_large(true);
        window.take_snapshot()?;
        let wide = gallery.get_tree_probe();
        check(
            wide.count == 1_000,
            "the large TreeView starts with all 1,000 root rows visible in the model",
        )?;
        click(window, row(0));
        key(window, Key::End);
        window.take_snapshot()?;
        let last = gallery.get_tree_probe();
        check(
            last.focused == 3_996 && last.offset > 0.,
            "End seeks the last root and scrolls it into view",
        )?;
        check(
            reads.get() < 200,
            "TreeView reads the viewport rows through the shared virtual repeater",
        )?;
        println!(
            "TreeView renderer read {} of 1000 visible-model rows",
            reads.get()
        );
        Ok(())
    })();
    gallery
        .global::<TreeLogic>()
        .on_project(crate::tree_view::project);
    gallery.set_tree_large(false);
    gallery.set_tree_multiple(false);
    long_test
}

struct CountingRows {
    rows: ModelRc<ItemData>,
    reads: Rc<Cell<usize>>,
}

impl Model for CountingRows {
    type Data = ItemData;
    fn row_count(&self) -> usize {
        self.rows.row_count()
    }
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.reads.set(self.reads.get() + 1);
        self.rows.row_data(row)
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        self.rows.model_tracker()
    }
}

pub(super) fn verify_command_bar_startup_scale(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();
    gallery.set_page(GalleryPage::CommandBar);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;
    click(window, gallery.get_command_probe().more);
    let open = gallery.get_command_probe();
    check(
        open.open && (open.panel.width - 160. * theme.get_ui_scale()).abs() < 0.5,
        "the ordinary menu fits the minimum width at the Gallery startup scale",
    )?;
    for row in 0..3 {
        verify_command_icon_and_label(gallery, &open, row)?;
    }
    key(window, Key::Escape);
    Ok(())
}

pub(super) fn verify_command_bar(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::CommandBar);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;
    let wide = gallery.get_command_probe();
    check(
        wide.primary.iter().collect::<Vec<_>>() == [0, 1, 2]
            && wide.overflow.iter().collect::<Vec<_>>() == [3, 4, 5],
        "CommandBar keeps primary commands on the wide bar",
    )?;
    click(window, wide.more);
    window.take_snapshot()?;
    let open = gallery.get_command_probe();
    check(
        open.open
            && open.row == 0
            && (open.panel.y - open.bar.y - open.bar.height).abs() < 0.5
            && (open.panel.x + open.panel.width - open.bar.x - open.bar.width).abs() < 0.5,
        "More opens below the bar without a gap and aligns its right edge",
    )?;
    check(
        (open.panel.height - open.row_height * 3. - 10.).abs() < 0.5 && open.row_height < 40.,
        "More uses the AppBarButton row metrics and four pixels of presenter margin",
    )?;
    for row in 0..3 {
        verify_command_icon_and_label(gallery, &open, row)?;
    }
    key(window, Key::DownArrow);
    check(
        gallery.get_command_probe().row == 2,
        "Down skips disabled commands",
    )?;
    key(window, Key::Return);
    window.take_snapshot()?;
    let preview = gallery.get_command_probe();
    check(
        !preview.open && preview.more_focused && preview.last == 2 && preview.last_secondary,
        "Preview invokes a regular secondary command and restores focus to More",
    )?;

    gallery.set_command_narrow(true);
    window.take_snapshot()?;
    let narrow = gallery.get_command_probe();
    check(
        narrow.overflow.iter().collect::<Vec<_>>() == [2, 3, 4, 5],
        "a narrow bar moves Share ahead of the secondary commands",
    )?;
    click(window, narrow.more);
    let mixed = gallery.get_command_probe();
    check(
        (mixed.panel.height - mixed.row_height * mixed.overflow.row_count() as f32 - 19.).abs()
            < 0.5,
        "moved primary commands are separated from the secondary group by nine pixels",
    )?;
    verify_command_icon_and_label(gallery, &mixed, 0)?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        !gallery.get_command_probe().open && gallery.get_command_probe().more_focused,
        "Escape closes overflow without invoking its focused command",
    )?;
    key(window, Key::DownArrow);
    window.take_snapshot()?;
    check(
        gallery.get_command_probe().open,
        "Down on More opens keyboard overflow",
    )?;
    key(window, Key::Return);
    window.take_snapshot()?;
    let share = gallery.get_command_probe();
    check(
        !share.open && share.last == 2 && !share.last_secondary,
        "a moved primary command retains its identity",
    )?;
    click(window, share.more);
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_command_probe().open,
        "outside click dismisses overflow",
    )?;
    gallery.set_command_narrow(false);
    window.take_snapshot()?;
    check(
        gallery.get_command_probe().primary.row_count() == 3,
        "widening returns the primary commands",
    )?;

    for scheme in [FluentColorScheme::Dark, FluentColorScheme::HighContrast] {
        gallery.invoke_select_theme(scheme);
        click(window, gallery.get_command_probe().more);
        let panel = gallery.get_command_probe().panel;
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 10., panel.y + 10.))?
                == [fill.red(), fill.green(), fill.blue()],
            "CommandBar paints its dark and high-contrast overflow surfaces",
        )?;
        click(window, LogicalPosition::new(2., 2.));
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    verify_command_overflow_placement(gallery, open.panel.height)?;
    verify_command_live_acrylic(gallery)
}

fn verify_command_icon_and_label(
    gallery: &GalleryWindow,
    probe: &CommandBarProbe,
    row: usize,
) -> SmokeResult {
    let ui_scale = gallery.global::<FluentTheme>().get_ui_scale();
    let top = probe.panel.y + 5. * ui_scale + row as f32 * probe.row_height;
    let window = gallery.window();
    let frame = window.take_snapshot()?;
    let scale = window.scale_factor();
    let row_inset = 4. * ui_scale;
    let y_start = ((top + row_inset) * scale).ceil() as usize;
    let y_end = ((top + probe.row_height - row_inset) * scale).floor() as usize;
    let has_ink = |column: i32| {
        let x = ((probe.panel.x + (column as f32 + 0.5) * ui_scale) * scale).floor() as usize;
        (y_start..y_end).any(|y| {
            let pixel = frame.as_slice()[y * frame.width() as usize + x];
            pixel.r.max(pixel.g).max(pixel.b) < 190
        })
    };
    check(
        !(5..13).any(has_ink)
            && (13..21).any(has_ink)
            && !(29..39).any(has_ink)
            && (39..47).any(has_ink),
        "CommandBar renders the icon at 12 and the label at 38 inside the one-pixel border",
    )
}

fn verify_command_live_acrylic(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();
    let previous_preference = theme.get_reduce_transparency();
    theme.set_reduce_transparency(false);
    let result = (|| -> SmokeResult {
        gallery.set_command_fixture_backdrop(Color::from_rgb_u8(220, 20, 110));
        window.take_snapshot()?;
        click(window, gallery.get_command_probe().more);
        let panel = gallery.get_command_probe().panel;
        let sample = LogicalPosition::new(panel.x + 9., panel.y + 21.);
        let warm = read_rgb(window, sample)?;
        gallery.set_command_fixture_backdrop(Color::from_rgb_u8(10, 180, 220));
        let cool = read_rgb(window, sample)?;
        check(
            gallery.get_command_probe().open
                && warm[0].saturating_sub(cool[0]) > 10
                && cool[1].saturating_sub(warm[1]) > 10,
            "open CommandBar Acrylic responds to changes behind the popup",
        )
    })();
    key(window, Key::Escape);
    gallery.set_command_fixture_backdrop(Color::default());
    theme.set_reduce_transparency(previous_preference);
    result
}

fn verify_command_overflow_placement(gallery: &GalleryWindow, natural_height: f32) -> SmokeResult {
    let window = gallery.window();
    let original = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(380., original.height),
    });
    window.take_snapshot()?;
    click(window, gallery.get_command_probe().more);
    let full = gallery.get_command_probe();
    check(
        full.open && full.panel.x.abs() < 0.5 && (full.panel.width - 380.).abs() < 0.5,
        "overflow spans the visible window on narrow hosts",
    )?;
    key(window, Key::Escape);
    window.dispatch_event(WindowEvent::Resized { size: original });
    window.take_snapshot()?;

    let closed = gallery.get_command_probe();
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(original.width, closed.bar.y + 64. + natural_height - 20.),
    });
    window.take_snapshot()?;
    click(window, gallery.get_command_probe().more);
    let upper = gallery.get_command_probe();
    check(
        upper.open
            && (upper.panel.y + upper.panel.height - upper.bar.y).abs() < 0.5
            && (upper.bar.y + upper.bar.height - closed.bar.y - closed.bar.height).abs() < 0.5,
        "overflow opens upward and keeps the compact bar's bottom edge fixed",
    )?;
    key(window, Key::Escape);
    window.dispatch_event(WindowEvent::Resized { size: original });
    gallery.set_command_fixture(ModelRc::new(VecModel::from(
        (0..24)
            .map(|index| CommandBarItem {
                text: format!("Command {index} with a longer label to measure").into(),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    )));
    window.take_snapshot()?;
    click(window, gallery.get_command_probe().more);
    let long = gallery.get_command_probe();
    check(
        long.open
            && long.panel.width > 160.
            && long.panel.width <= 480.
            && (long.panel.height - original.height / 2.).abs() < 1.,
        "overflow measures long labels and limits height to half the viewport",
    )?;
    let top_margin =
        LogicalPosition::new(long.panel.x + long.panel.width - 30., long.panel.y + 2.5);
    let outside = LogicalPosition::new(2., 2.);
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    let margin = read_rgb(window, top_margin)?;
    window.dispatch_event(WindowEvent::PointerMoved {
        position: top_margin,
    });
    check(
        read_rgb(window, top_margin)? == margin,
        "the initial presenter margin is outside command hit targets",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    key(window, Key::End);
    window.take_snapshot()?;
    let scrolled = gallery.get_command_probe();
    check(
        scrolled.row == 23 && scrolled.scroll_offset > 0.,
        "long command lists scroll to the focused item",
    )?;
    let before_hover = read_rgb(window, top_margin)?;
    window.dispatch_event(WindowEvent::PointerMoved {
        position: top_margin,
    });
    check(
        read_rgb(window, top_margin)? != before_hover,
        "the presenter margin scrolls away and a command can reach the top of the viewport",
    )?;
    key(window, Key::Escape);
    gallery.set_command_fixture(ModelRc::default());
    Ok(())
}

pub(super) fn verify_command_motion(
    gallery: &GalleryWindow,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    use std::time::Duration;
    gallery.set_page(GalleryPage::CommandBar);
    gallery
        .window()
        .set_size(slint::LogicalSize::new(980., 700.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed before CommandBar motion".into()));
        };
        let closed = gallery.get_command_probe();
        click(gallery.window(), closed.more);
        slint::Timer::single_shot(Duration::from_millis(78), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed during CommandBar motion".into()));
            };
            let middle = gallery.get_command_probe();
            if let Err(error) = check(
                middle.open && middle.height > closed.height && middle.height < 64.,
                "CommandBar shows an intermediate expansion height",
            ) {
                return finish(Err(error));
            }
            slint::Timer::single_shot(Duration::from_millis(250), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed before CommandBar settled".into()));
                };
                let expanded = gallery.get_command_probe();
                if let Err(error) = check(
                    (expanded.height - 64.).abs() < 0.1,
                    "CommandBar settles at its open height",
                ) {
                    return finish(Err(error));
                }
                key(gallery.window(), Key::Escape);
                slint::Timer::single_shot(Duration::from_millis(220), move || {
                    let result = weak
                        .upgrade()
                        .ok_or_else(|| "Gallery closed before CommandBar restored focus".into())
                        .and_then(|gallery| {
                            let closed = gallery.get_command_probe();
                            check(
                                !closed.open
                                    && closed.more_focused
                                    && (closed.height - 48.).abs() < 0.1,
                                "CommandBar finishes closing and restores focus to More",
                            )
                        });
                    finish(result);
                });
            });
        });
    });
}
