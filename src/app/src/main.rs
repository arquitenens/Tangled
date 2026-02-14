use std::alloc::System;
use std::mem::ManuallyDrop;
use std::num::NonZero;
use std::os::unix::fs::chown;
use std::pin::Pin;
use std::ptr;
use config::config::Config;
use tangled::tangled::Tangled;




fn main() {

    let mut tangled: Tangled<i32> = Tangled::default();

    let worker1 = tangled.add_worker(|mut x| {
        x.borrow_mut().push(1);
        //x.borrow().print();
    });
    let worker2 = tangled.add_worker(|mut x| {
        x.borrow_mut().push(2);
        x.borrow().print();
    });


    let handle = tangled.start();


    handle.join().expect("TODO: panic message");
    //println!("Hello, world!");
}

