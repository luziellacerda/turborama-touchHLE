/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITabBarController`.
//!
//! A container view controller that shows one of several child view
//! controllers at a time. This is a simplified implementation: it tracks the
//! child controllers and a selected index, and shows the selected controller's
//! view. The tab bar itself isn't rendered, so tab switching by touch isn't
//! modelled; the initially selected controller is displayed.

use crate::frameworks::foundation::NSUInteger;
use crate::objc::{
    autorelease, id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes,
    release, retain, ClassExports, NSZonePtr,
};

#[derive(Default)]
struct UITabBarControllerHostObject {
    superclass: super::UIViewControllerHostObject,
    /// Child view controllers, retained.
    view_controllers: Vec<id>,
    /// Delegate (UITabBarControllerDelegate), non-retaining.
    delegate: id,
    selected_index: NSUInteger,
}
impl_HostObject_with_superclass!(UITabBarControllerHostObject);

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITabBarController: UIViewController

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITabBarControllerHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UITabBarControllerHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UITabBarControllerHostObject>(this).delegate
}

- (())setViewControllers:(id)controllers { // NSArray*
    msg![env; this setViewControllers:controllers animated:false]
}

- (())setViewControllers:(id)controllers // NSArray*
               animated:(bool)_animated {
    // Release the old set.
    let old = std::mem::take(
        &mut env.objc.borrow_mut::<UITabBarControllerHostObject>(this).view_controllers,
    );
    for vc in old {
        release(env, vc);
    }

    let mut new_vcs: Vec<id> = Vec::new();
    if controllers != nil {
        let count: NSUInteger = msg![env; controllers count];
        for i in 0..count {
            let vc: id = msg![env; controllers objectAtIndex:i];
            retain(env, vc);
            new_vcs.push(vc);
        }
    }
    env.objc.borrow_mut::<UITabBarControllerHostObject>(this).view_controllers = new_vcs;
    env.objc.borrow_mut::<UITabBarControllerHostObject>(this).selected_index = 0;

    // Show the initially selected controller's view.
    () = msg![env; this _touchHLE_showSelectedViewController];
}

- (id)viewControllers {
    let vcs = env.objc.borrow::<UITabBarControllerHostObject>(this).view_controllers.clone();
    let array: id = msg_class![env; NSMutableArray arrayWithCapacity:(vcs.len() as NSUInteger)];
    for vc in vcs {
        () = msg![env; array addObject:vc];
    }
    let array: id = msg![env; array copy];
    autorelease(env, array)
}

- (())setSelectedIndex:(NSUInteger)index {
    let count = env.objc.borrow::<UITabBarControllerHostObject>(this).view_controllers.len();
    if (index as usize) < count {
        env.objc.borrow_mut::<UITabBarControllerHostObject>(this).selected_index = index;
        () = msg![env; this _touchHLE_showSelectedViewController];
    }
}
- (NSUInteger)selectedIndex {
    env.objc.borrow::<UITabBarControllerHostObject>(this).selected_index
}

- (id)selectedViewController {
    let &UITabBarControllerHostObject { selected_index, .. } =
        env.objc.borrow::<UITabBarControllerHostObject>(this);
    let vcs = &env.objc.borrow::<UITabBarControllerHostObject>(this).view_controllers;
    vcs.get(selected_index as usize).copied().unwrap_or(nil)
}

- (())setSelectedViewController:(id)view_controller {
    let pos = env
        .objc
        .borrow::<UITabBarControllerHostObject>(this)
        .view_controllers
        .iter()
        .position(|&vc| vc == view_controller);
    if let Some(index) = pos {
        () = msg![env; this setSelectedIndex:(index as NSUInteger)];
    }
}

// The tab bar isn't rendered; report nil rather than a fake object.
- (id)tabBar {
    nil
}

// Helper: display the currently selected child controller's view.
- (())_touchHLE_showSelectedViewController {
    let selected: id = msg![env; this selectedViewController];
    if selected == nil {
        return;
    }
    let self_view: id = msg![env; this view];
    let vc_view: id = msg![env; selected view];
    () = msg![env; selected viewWillAppear:false];
    () = msg![env; self_view addSubview:vc_view];
    () = msg![env; selected viewDidAppear:false];
}

- (())dealloc {
    let vcs = std::mem::take(
        &mut env.objc.borrow_mut::<UITabBarControllerHostObject>(this).view_controllers,
    );
    for vc in vcs {
        release(env, vc);
    }
    msg_super![env; this dealloc]
}

@end

};
