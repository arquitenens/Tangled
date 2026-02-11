use std::alloc::System;
use std::os::unix::fs::chown;
use config::config::Config;
use tangled::borrow::BorrowedTangled;
use tangled::tangled::Tangled;



fn main() {

    /*

    parent: [child_thread1, child_thread2, child_thread3...]

    parent.message_loop.start()

    child_thread1.borrow().some_method() -> send tx parent

    parent.message_loop -> reply rx child_thread1

    let result = child_thread1.recv()

    */

    
    let mut tangled: Tangled<i32> = Tangled::default();
    tangled.add_child();
    let x =tangled.inners.get(0).unwrap().borrow().get(0);

    tangled.handle_request();
}

