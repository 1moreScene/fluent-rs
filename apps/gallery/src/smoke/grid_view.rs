use std::{cell::Cell, error::Error, rc::Rc};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Model, ModelNotify, ModelRc, ModelTracker,
    Window,
    platform::{Key, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, ItemData, ListSelectionMode,
};

type SmokeResult = Result<(), Box<dyn Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::GridView);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_grid_show_large(false);
    gallery.set_grid_scroll_horizontally(false);
    gallery.set_grid_selection_mode(ListSelectionMode::Single);
    gallery.window().take_snapshot()?;
    verify_layout_and_selection(gallery)?;
    verify_virtualization(gallery)?;
    gallery.set_grid_show_large(false);
    gallery.set_grid_scroll_horizontally(false);
    gallery.set_grid_selection_mode(ListSelectionMode::Single);
    gallery.set_grid_fixture(gallery.get_composition_items());
    Ok(())
}

fn verify_layout_and_selection(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let initial = gallery.get_grid_probe();
    let scale = gallery.global::<FluentTheme>().get_ui_scale();
    let pitch_x = 152. * scale;
    let pitch_y = 90. * scale;
    check(
        initial.lanes == 3
            && initial.realized == 9
            && (initial.second.x - initial.first.x - pitch_x).abs() < 0.5
            && (initial.fourth.y - initial.first.y - pitch_y).abs() < 0.5
            && initial.fourth.x == initial.first.x,
        "GridView wraps the custom template into three evenly spaced columns",
    )?;

    click(window, initial.third);
    let disabled = gallery.get_grid_probe();
    check(
        disabled.last_clicked == -1 && disabled.selected.row_count() == 0,
        "disabled GridView items cannot be invoked or selected",
    )?;

    click(window, initial.first);
    check(
        selected(gallery, &[0]) && gallery.get_grid_probe().last_clicked == 0,
        "click focuses, invokes and selects a GridView item",
    )?;
    modifier(window, Key::Control, || key(window, Key::RightArrow));
    check(
        selected(gallery, &[0]) && gallery.get_grid_probe().focused == 1,
        "Control+Right moves GridView focus independently of selection",
    )?;
    key(window, Key::RightArrow);
    check(
        selected(gallery, &[3]) && gallery.get_grid_probe().focused == 3,
        "Right skips disabled cells and advances in source order",
    )?;
    key(window, Key::DownArrow);
    check(
        selected(gallery, &[6]) && gallery.get_grid_probe().focused == 6,
        "Down moves one visual row and retains the column",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_grid_probe().last_clicked == 6,
        "Return invokes the focused GridView item",
    )?;

    let accent = gallery.global::<FluentTheme>();
    let edge = LogicalPosition::new(initial.first.x - 73. * scale, initial.first.y);
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(initial.bounds.x, initial.bounds.y - 2.),
    });
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        let before = read_rgb(window, edge)?;
        gallery.invoke_grid_select_index(0);
        let after = read_rgb(window, edge)?;
        let expected = if scheme == FluentColorScheme::HighContrast {
            accent.get_contrast_highlight()
        } else {
            accent.get_accent_fill()
        };
        let expected = [expected.red(), expected.green(), expected.blue()];
        if before == after || after != expected {
            return Err(format!(
                "GridView selection outline in {scheme:?}: before {before:?}, after {after:?}, expected {expected:?}"
            )
            .into());
        }
        gallery.invoke_grid_select_index(6);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);

    gallery.set_grid_selection_mode(ListSelectionMode::Multiple);
    gallery.invoke_grid_select_index(-1);
    window.take_snapshot()?;
    click(window, initial.first);
    click(window, initial.second);
    check(
        selected(gallery, &[0, 1]),
        "multiple mode toggles clicked cells independently",
    )?;
    let checkmark =
        LogicalPosition::new(initial.first.x + 60. * scale, initial.first.y - 29. * scale);
    click(window, checkmark);
    check(
        selected(gallery, &[1]) && gallery.get_grid_probe().last_clicked == 1,
        "the GridView checkbox toggles selection independently of item invocation",
    )?;

    gallery.set_grid_selection_mode(ListSelectionMode::Extended);
    window.take_snapshot()?;
    click(window, initial.first);
    modifier(window, Key::Shift, || key(window, Key::DownArrow));
    check(
        selected(gallery, &[0, 1, 3]),
        "Shift builds an extended range without disabled entries",
    )?;
    modifier(window, Key::Control, || {
        window.dispatch_event(WindowEvent::KeyPressed { text: "a".into() });
        window.dispatch_event(WindowEvent::KeyReleased { text: "a".into() });
    });
    check(
        gallery.get_grid_probe().selected.row_count() == 8,
        "Select All selects enabled cells",
    )?;

    gallery.set_grid_selection_mode(ListSelectionMode::None);
    window.take_snapshot()?;
    click(window, initial.fourth);
    key(window, Key::Space);
    check(
        selected(gallery, &[]) && gallery.get_grid_probe().last_clicked == 3,
        "item invocation remains available in no-selection mode",
    )?;

    gallery.set_grid_selection_mode(ListSelectionMode::Single);
    window.take_snapshot()?;
    click(window, initial.first);
    let original = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(320., original.height),
    });
    window.take_snapshot()?;
    check(
        gallery.get_grid_probe().lanes < 3 && selected(gallery, &[0]),
        "a narrow window reflows the columns and preserves source selection",
    )?;
    window.dispatch_event(WindowEvent::Resized { size: original });
    window.take_snapshot()?;
    Ok(())
}

fn verify_virtualization(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let model = Rc::new(CountingGrid {
        count: 100_000,
        reads: Cell::new(0),
        notify: ModelNotify::default(),
    });
    gallery.set_grid_fixture(ModelRc::from(Rc::clone(&model)));
    gallery.set_grid_show_large(true);
    gallery.set_grid_selection_mode(ListSelectionMode::Single);
    window.take_snapshot()?;
    let initial = gallery.get_grid_probe();
    let initial_reads = model.reads.get();
    check(
        (1..=45).contains(&initial.realized)
            && (1..500).contains(&initial_reads)
            && initial.extent > 2_000_000.,
        "an initial 100,000-cell layout realizes and reads only the visible strips",
    )?;

    click(window, initial.first);
    let before_wheel = model.reads.get();
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: initial.first,
        delta_x: 0.,
        delta_y: -700.,
    });
    window.take_snapshot()?;
    let wheel = gallery.get_grid_probe();
    check(
        wheel.offset > 450.
            && wheel.first_realized > 0
            && selected(gallery, &[0])
            && model.reads.get() - before_wheel < 450,
        "wheel scrolling recycles GridView cells and preserves offscreen selection",
    )?;
    let before_jump = model.reads.get();
    gallery.invoke_grid_scroll_to_index(50_000);
    window.take_snapshot()?;
    let middle = gallery.get_grid_probe();
    check(
        middle.first_realized <= 50_000
            && middle.first_realized + middle.realized > 50_000
            && model.reads.get() - before_jump < 500
            && selected(gallery, &[0]),
        "jumping to an unrealized item does not enumerate the intervening model",
    )?;
    key(window, Key::End);
    window.take_snapshot()?;
    let end = gallery.get_grid_probe();
    check(
        end.focused == 99_999
            && selected(gallery, &[99_999])
            && end.first_realized + end.realized == 100_000
            && end.realized < 45,
        "End realizes the final strip and scrolls the focused item into view",
    )?;

    gallery.set_grid_scroll_horizontally(true);
    window.take_snapshot()?;
    key(window, Key::Home);
    window.take_snapshot()?;
    let column = gallery.get_grid_probe();
    check(
        column.lanes == 3
            && column.offset == 0.
            && (column.first.x - column.second.x).abs() < 0.5
            && column.fourth.x > column.first.x
            && (column.fourth.y - column.first.y).abs() < 0.5,
        "column-first layout wraps vertically and recycles horizontal columns",
    )?;
    key(window, Key::RightArrow);
    check(
        gallery.get_grid_probe().focused == 3,
        "Right steps one column in column-first layout",
    )?;
    let rail = LogicalPosition::new(
        column.bounds.x + column.bounds.width - 15.,
        column.bounds.y + column.bounds.height - 4.,
    );
    click(window, rail);
    window.take_snapshot()?;
    check(
        gallery.get_grid_probe().offset > 0.,
        "the shared horizontal scroll bar pages the grid",
    )?;
    let before_wheel = gallery.get_grid_probe().offset;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: LogicalPosition::new(column.bounds.x + 60., column.bounds.y + 40.),
        delta_x: -450.,
        delta_y: 0.,
    });
    window.take_snapshot()?;
    check(
        gallery.get_grid_probe().offset > before_wheel + 300.,
        "a horizontal trackpad delta scrolls the native Flickable",
    )?;
    check(
        model.reads.get() < 2_500,
        "model reads remain bounded across both grid orientations",
    )?;
    println!(
        "GridView virtualized 100,000 items; initial reads {initial_reads}, total reads {}",
        model.reads.get()
    );

    gallery.set_grid_fixture(ModelRc::default());
    window.take_snapshot()?;
    let empty = gallery.get_grid_probe();
    check(
        empty.realized == 0 && empty.offset == 0. && empty.selected.row_count() == 0,
        "replacing the model clears selection and resets the virtual viewport",
    )
}

fn selected(gallery: &GalleryWindow, expected: &[i32]) -> bool {
    gallery
        .get_grid_probe()
        .selected
        .iter()
        .eq(expected.iter().copied())
}

fn modifier(window: &Window, tag: Key, action: impl FnOnce()) {
    window.dispatch_event(WindowEvent::KeyPressed { text: tag.into() });
    action();
    window.dispatch_event(WindowEvent::KeyReleased { text: tag.into() });
}

struct CountingGrid {
    count: usize,
    reads: Cell<usize>,
    notify: ModelNotify,
}

impl Model for CountingGrid {
    type Data = ItemData;

    fn row_count(&self) -> usize {
        self.count
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if row >= self.count {
            return None;
        }
        self.reads.set(self.reads.get() + 1);
        Some(ItemData {
            text: format!("Tile {}", row + 1).into(),
            disabled: false,
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}
