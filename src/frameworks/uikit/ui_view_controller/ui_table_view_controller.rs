/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableViewController`.
//!
//! A view controller whose root view is a UITableView. It also acts as the
//! table view's data source and delegate by default (as on iOS), so a subclass
//! only needs to override the data source methods.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::NSInteger;
use crate::objc::{id, msg, msg_class, objc_classes, ClassExports};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableViewController: UIViewController

- (())loadView {
    // Create a UITableView filling the screen and make it the root view.
    let screen: id = msg_class![env; UIScreen mainScreen];
    let bounds: CGRect = msg![env; screen bounds];
    let style: NSInteger = 0; // UITableViewStylePlain
    let table_view: id = msg_class![env; UITableView alloc];
    let table_view: id = msg![env; table_view initWithFrame:bounds style:style];

    // The controller is its own data source and delegate by default.
    () = msg![env; table_view setDataSource:this];
    () = msg![env; table_view setDelegate:this];

    () = msg![env; this setView:table_view];
}

- (id)tableView {
    // The root view is the table view (created in loadView).
    msg![env; this view]
}

- (())viewDidLoad {
    // Nothing extra to do; subclasses may override.
}

// Default data source: an empty table. Subclasses override these.
- (NSInteger)tableView:(id)_table_view numberOfRowsInSection:(NSInteger)_section {
    0
}

- (NSInteger)numberOfSectionsInTableView:(id)_table_view {
    1
}

@end

};
