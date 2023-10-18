use std::io::Cursor;

#[test]
fn test_bft_run() {
    let bf_info = bft_types::BFProgram::new(
        "",
        "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..
        +++.>>.<-.<.+++.------.--------.>>+.>++.",
    );
    let mut bf_vm = bft_interp::BFVirtualMachine::<u8>::new(1000, false, &bf_info);
    let mut r_buf = Cursor::new(Vec::<u8>::new());
    let mut w_buf = Cursor::new(Vec::<u8>::new());
    let result = bf_vm.interpret(&mut r_buf, &mut w_buf);

    assert!(result.is_ok());
    let hello_world = Vec::<u8>::from("Hello World!\n");
    assert_eq!(w_buf.get_ref(), &hello_world);
}

#[test]
fn test_input_output() {
    let bf_info = bft_types::BFProgram::new("", ",>,>,>,>,>,.<.<.<.<.<.");
    let mut bf_vm = bft_interp::BFVirtualMachine::<u8>::new(1000, false, &bf_info);
    let mut r_buf = Cursor::new(vec![1, 2, 3, 4, 5, 6]);
    let mut w_buf = Cursor::new(Vec::<u8>::new());
    let result = bf_vm.interpret(&mut r_buf, &mut w_buf);

    assert!(result.is_ok());
    assert_eq!(w_buf.get_ref(), &vec![6, 5, 4, 3, 2, 1, 10]);
}

#[test]
fn test_increment() {
    let bf_info = bft_types::BFProgram::new("", ",+>,+>,+>,+>,+>,+.<.<.<.<.<.");
    let mut bf_vm = bft_interp::BFVirtualMachine::<u8>::new(10, false, &bf_info);
    let mut r_buf = Cursor::new(vec![1, 2, 3, 4, 5, 6]);
    let mut w_buf = Cursor::new(Vec::<u8>::new());
    let result = bf_vm.interpret(&mut r_buf, &mut w_buf);

    assert!(result.is_ok());
    assert_eq!(w_buf.get_ref(), &vec![7, 6, 5, 4, 3, 2, 10]);
}

#[test]
fn test_decrement() {
    let bf_info = bft_types::BFProgram::new("", ",->,->,->,->,->,-.<.<.<.<.<.");
    let mut bf_vm = bft_interp::BFVirtualMachine::<u8>::new(10, false, &bf_info);
    let mut r_buf = Cursor::new(vec![1, 2, 3, 4, 5, 6]);
    let mut w_buf = Cursor::new(Vec::<u8>::new());
    let result = bf_vm.interpret(&mut r_buf, &mut w_buf);

    assert!(result.is_ok());
    assert_eq!(w_buf.get_ref(), &vec![5, 4, 3, 2, 1, 0, 10]);
}

#[test]
fn test_loop() {
    let bf_info = bft_types::BFProgram::new("", ",[-.]");
    let mut bf_vm = bft_interp::BFVirtualMachine::<u8>::new(10, false, &bf_info);
    let mut r_buf = Cursor::new(vec![6]);
    let mut w_buf = Cursor::new(Vec::<u8>::new());
    let result = bf_vm.interpret(&mut r_buf, &mut w_buf);

    assert!(result.is_ok());
    assert_eq!(w_buf.get_ref(), &vec![5, 4, 3, 2, 1, 0, 10]);
}
