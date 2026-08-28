/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableView`.
//!
//! This is a functional but simplified implementation: reloadData asks the data
//! source for the row count and each row's cell, then stacks the cells
//! vertically as subviews. Cells are real UIViews, so the existing UIView/
//! CALayer rendering makes them visible. Reuse queues, sections beyond one,
//! headers/footers, and scrolling-driven lazy loading are not modelled.

use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::{NSInteger, NSUInteger};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, ClassExports,
    NSZonePtr,
};

/// Default row height in points, matching iOS's default.
const DEFAULT_ROW_HEIGHT: CGFloat = 44.0;

pub struct UITableViewHostObject {
    superclass: super::ui_scroll_view::UIScrollViewHostObject,
    /// UITableViewDataSource, weak reference.
    data_source: id,
    /// UITableViewDelegate, weak reference.
    delegate: id,
    row_height: CGFloat,
    style: NSInteger,
}
impl_HostObject_with_superclass!(UITableViewHostObject);
impl Default for UITableViewHostObject {
    fn default() -> Self {
        UITableViewHostObject {
            superclass: Default::default(),
            data_source: nil,
            delegate: nil,
            row_height: DEFAULT_ROW_HEIGHT,
            style: 0,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableView: UIScrollView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITableViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithFrame:(CGRect)frame style:(NSInteger)style {
    let this: id = msg![env; this initWithFrame:frame];
    env.objc.borrow_mut::<UITableViewHostObject>(this).style = style;
    this
}

- (())setDataSource:(id)data_source {
    env.objc.borrow_mut::<UITableViewHostObject>(this).data_source = data_source;
}
- (id)dataSource {
    env.objc.borrow::<UITableViewHostObject>(this).data_source
}

- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UITableViewHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UITableViewHostObject>(this).delegate
}

- (())setRowHeight:(CGFloat)row_height {
    env.objc.borrow_mut::<UITableViewHostObject>(this).row_height = row_height;
}
- (CGFloat)rowHeight {
    env.objc.borrow::<UITableViewHostObject>(this).row_height
}

- (())setSeparatorStyle:(NSInteger)_style {
    // No separators are drawn; accepted for compatibility.
}
- (())setBackgroundColor:(id)color {
    () = msg_super![env; this setBackgroundColor:color];
}

// No reuse queue is modelled, so nothing can be dequeued; the data source will
// create a fresh cell, which is correct (if less efficient) behaviour.
- (id)dequeueReusableCellWithIdentifier:(id)_identifier { // NSString*
    nil
}

- (id)cellForRowAtIndexPath:(id)_index_path {
    // Not tracked after layout; callers generally tolerate nil here.
    nil
}

- (())reloadData {
    let data_source = env.objc.borrow::<UITableViewHostObject>(this).data_source;
    if data_source == nil {
        return;
    }

    // Remove previously laid-out cells. (We re-add everything below.)
    let old_subviews: id = msg![env; this subviews];
    let count: NSUInteger = msg![env; old_subviews count];
    let mut removals: Vec<id> = Vec::new();
    for i in 0..count {
        let subview: id = msg![env; old_subviews objectAtIndex:i];
        removals.push(subview);
    }
    for subview in removals {
        () = msg![env; subview removeFromSuperview];
    }

    // Single-section table: ask for the row count in section 0.
    let section: NSInteger = 0;
    let num_rows: NSInteger = msg![env; data_source tableView:this numberOfRowsInSection:section];

    let bounds: CGRect = msg![env; this bounds];
    let row_height = env.objc.borrow::<UITableViewHostObject>(this).row_height;

    let mut y: CGFloat = 0.0;
    for row in 0..num_rows {
        let index_path: id =
            msg_class![env; NSIndexPath indexPathForRow:(row as NSUInteger)
                                             inSection:(section as NSUInteger)];
        let cell: id = msg![env; data_source tableView:this cellForRowAtIndexPath:index_path];
        if cell != nil {
            let cell_frame = CGRect {
                origin: CGPoint { x: 0.0, y },
                size: CGSize {
                    width: bounds.size.width,
                    height: row_height,
                },
            };
            () = msg![env; cell setFrame:cell_frame];
            () = msg![env; this addSubview:cell];
        }
        y += row_height;
    }

    // Update the scrollable content size to fit all rows.
    let content_size = CGSize {
        width: bounds.size.width,
        height: y,
    };
    () = msg![env; this setContentSize:content_size];
}

@end

};
