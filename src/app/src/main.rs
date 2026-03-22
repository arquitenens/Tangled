use std::time::Instant;
use tangled::tangled::Tangled;


fn main() {

    let mut tangled: Tangled<i32> = Tangled::default();

    let worker1 = tangled.add_worker(|mut x| {
        let borrowed = x.borrow_mut();
        for i in 0..1000000{
            borrowed.push(i);
        }
        //x.borrow().print();
    });
    let worker2= tangled.add_worker(|mut x| {
        let borrowed = x.borrow_mut();
        for i in 0..1000000{
            borrowed.push(i);
        }
        //x.borrow().print();
    });
    let worker3 = tangled.add_worker(|mut x| {
        let borrowed = x.borrow_mut();
        for i in 0..1000000{
            borrowed.push(i);
        }
        //x.borrow().print();
    });
    let worker4 = tangled.add_worker(|mut x| {
        let borrowed = x.borrow_mut();
        for i in 0..1000000{
            borrowed.push(i);
        }
        //x.borrow().print();
    });
    let worker5 = tangled.add_worker(|x| {
        let now = Instant::now();
        let borrow = x.borrow();
        for i in 0..4000000{
            let y = borrow.get(i);
            //println!("y: {:?}", y);
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:?}", elapsed);
        //x.borrow().print();
    });



    let handle = tangled.start();


    handle.join().expect("TODO: panic message");
    //println!("Hello, world!");
}

