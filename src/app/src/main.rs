use std::alloc::System;
use std::os::unix::fs::chown;
use std::ptr;
use config::config::Config;
use tangled::borrow::BorrowedTangled;
use tangled::tangled::Tangled;



fn main() {



    let mut tangled: Tangled<i32> = Tangled::default();
    tangled.add_child();
    tangled.add_child();

    //println!("{:#?}", tangled);
    let handle = tangled.start();
    std::thread::spawn(move || {
        let x = tangled.inners.get(0).unwrap().borrow().get(0);
        println!("hi {:?}", x);
    });

    handle.join().expect("TODO: panic message");
}

