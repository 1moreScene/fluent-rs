use std::{cell::Cell, rc::Rc};

use slint::{Model, ModelRc, ModelTracker, SharedString};

use crate::ItemData;

const ROWS: usize = 100_000;

struct DemoItems {
    reads: Rc<Cell<usize>>,
}

impl Model for DemoItems {
    type Data = ItemData;

    fn row_count(&self) -> usize {
        ROWS
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if row >= ROWS {
            return None;
        }

        self.reads.set(self.reads.get() + 1);
        Some(ItemData {
            text: SharedString::from(format!("Record {}", row + 1)),
            disabled: row % 17 == 2,
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &()
    }
}

pub(super) fn items() -> (ModelRc<ItemData>, Rc<Cell<usize>>) {
    let reads = Rc::new(Cell::new(0));
    (
        ModelRc::new(DemoItems {
            reads: reads.clone(),
        }),
        reads,
    )
}
