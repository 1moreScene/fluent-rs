//! Preserve every pasteboard item and format while verifying password clipboard behavior.

use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{NSArray, NSString};

type SmokeResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) struct ClipboardGuard {
    board: Retained<NSPasteboard>,
    saved: Retained<NSArray<ProtocolObject<dyn NSPasteboardWriting>>>,
    last_write: isize,
    restored: bool,
}

impl ClipboardGuard {
    pub(super) fn new() -> SmokeResult<Self> {
        let board = NSPasteboard::generalPasteboard();
        let mut saved = Vec::new();
        if let Some(items) = board.pasteboardItems() {
            for item in items.iter() {
                let copy = NSPasteboardItem::new();
                for kind in item.types().iter() {
                    let data = item
                        .dataForType(&kind)
                        .ok_or("could not preserve a clipboard format")?;
                    if !copy.setData_forType(&data, &kind) {
                        return Err("could not preserve a clipboard item".into());
                    }
                }
                saved.push(ProtocolObject::from_retained(copy));
            }
        }
        let last_write = board.changeCount();
        Ok(Self {
            board,
            saved: NSArray::from_retained_slice(&saved),
            last_write,
            restored: false,
        })
    }

    pub(super) fn set_text(&mut self, text: &str) -> SmokeResult {
        let item = NSPasteboardItem::new();
        if !item.setString_forType(
            &NSString::from_str(text),
            &NSString::from_str("public.utf8-plain-text"),
        ) {
            return Err("could not prepare the clipboard test value".into());
        }
        let item: Retained<ProtocolObject<dyn NSPasteboardWriting>> =
            ProtocolObject::from_retained(item);
        let items = NSArray::from_retained_slice(&[item]);
        self.board.clearContents();
        let written = self.board.writeObjects(&items);
        self.last_write = self.board.changeCount();
        if !written {
            return Err("could not write the clipboard test value".into());
        }
        Ok(())
    }

    pub(super) fn text(&self) -> Option<String> {
        self.board
            .stringForType(&NSString::from_str("public.utf8-plain-text"))
            .map(|value| value.to_string())
    }

    pub(super) fn recognize_test_write(&mut self, expected: &str) {
        if self.text().as_deref() == Some(expected) {
            self.last_write = self.board.changeCount();
        }
    }

    pub(super) fn restore(&mut self) -> SmokeResult {
        if self.restored {
            return Ok(());
        }
        self.restored = true;
        // A newer clipboard write belongs to the user or another application.
        if self.board.changeCount() != self.last_write {
            return Ok(());
        }
        self.board.clearContents();
        if !self.saved.is_empty() && !self.board.writeObjects(&self.saved) {
            return Err("could not restore the preserved clipboard items".into());
        }
        Ok(())
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
