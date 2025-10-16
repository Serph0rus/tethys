#![no_std]
use core::arch::asm;
#[repr(C)]
pub enum Selector {
    Exit,
    Map,
    Length,
    Send,
    Query,
    Block,
    Respond,
    Check,
    Receive,
}
pub unsafe fn syscall(selector: Selector, arguments: &[u64]) -> Result<u64, ()> {
    let mut length_args: [u64; 5] = [0; 5];
    for i in 0..5 {
        length_args[i] = arguments.get(i).unwrap_or(&0).clone();
    }
    let mut error: u64;
    let mut result: u64;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") selector as u64 => error,
            inlateout("rdi") length_args[0] => result,
            in("rsi") length_args[1],
            in("rdx") length_args[2],
            in("r10") length_args[3],
            in("r8") length_args[4],
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack, preserves_flags)
        );
    }
    if error == 0 { Err(()) } else { Ok(result) }
}
pub unsafe fn syscall_exit() -> ! {
    unsafe { let _ = syscall(Selector::Exit, &[]); };
    panic!("thread did not exit!")
}
pub unsafe fn syscall_map(page_index: u64, page_count: u64) -> Result<(), ()> {
    unsafe { syscall(Selector::Map, &[page_index, page_count]) }.map(|_| ())
}
pub unsafe fn syscall_length(message_tag: u64) -> Result<u64, ()> {
    unsafe { syscall(Selector::Length, &[message_tag]) }
}
pub unsafe fn syscall_send(page_index: u64, page_count: u64) -> Result<u64, ()> {
    unsafe { syscall(Selector::Send, &[page_index, page_count]) }
}
pub unsafe fn syscall_query(message_tag: u64) -> Result<bool, ()> {
    unsafe { syscall(Selector::Query, &[message_tag]) }.map(|x| x != 0)
}
pub unsafe fn syscall_block(message_tag: u64, page_index: u64) -> Result<(), ()> {
    unsafe { syscall(Selector::Block, &[message_tag, page_index]) }.map(|_| ())
}
pub unsafe fn syscall_respond(
    server_tag: u64,
    message_tag: u64,
    page_index: u64,
    page_count: u64,
) -> Result<(), ()> {
    unsafe {
        syscall(
            Selector::Respond,
            &[server_tag, message_tag, page_index, page_count],
        )
        .map(|_| ())
    }
}