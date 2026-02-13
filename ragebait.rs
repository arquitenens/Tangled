enum PrintingError{
    CouldntPrint,
}

fn optimized_string_printing(string: &str) -> Result<String, PrintingError> {
    let string = ManuallyDrop::new(Pin::new(Box::new(String::from(string))));
    //SAFETY: it's a box so its stable on the heap!
    let ptr_tup = ((&string as *const _) as *const &[u8], string.len());
    let (x, _): (&*const &[u8], usize) = unsafe { std::mem::transmute_copy(&ptr_tup) };
    let offset = unsafe { x.byte_add(size_of::<usize>()) };
    let deref = unsafe {&*offset};
    let error = println!("{:?}", deref.iter().map(|a| *a as char).collect::<String>());
    if error == (){
        Ok("ok".to_string())
    }else {
        Err(PrintingError::CouldntPrint)
    }
}
