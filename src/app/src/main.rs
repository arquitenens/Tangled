use std::alloc::System;
use std::num::NonZero;
use std::os::unix::fs::chown;
use std::ptr;
use config::config::Config;
use tangled::tangled::Tangled;


fn main() {
    let mut tangled: Tangled<i32> = Tangled::default();

    //let worker1 = tangled.add_worker();
    let worker2 = tangled.add_worker(|mut x|{
        let _ = x.borrow_mut().push(1);
        //println!("{:?}", x);

    });
    let worker1 = tangled.add_worker(|x| {
        let x = x.borrow().get(0);
        println!("{:?}", x);
    });

    let handle = tangled.start();


    handle.join().expect("TODO: panic message");
}

