#![no_std]
use core::arch::asm;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};
const PAGE_SIZE: usize = 4096;
const HEAP_PAGE_INDEX: usize = 0x0000_4000_0000;
const BUFFER_PAGE_INDEX: usize = 0x0000_8000_0000;
static NEXT_HEAP_PAGE: AtomicUsize = AtomicUsize::new(HEAP_PAGE_INDEX);
static NEXT_BUFFER_PAGE: AtomicUsize = AtomicUsize::new(BUFFER_PAGE_INDEX);
#[repr(C)]
pub enum Selector {
    Abort,
    Map,
    Move,
    Length,
    Send,
    Query,
    Receive,
    Respond,
}
pub unsafe fn syscall(selector: Selector, arguments: &[usize]) -> Result<usize, ()> {
    let mut length_args: [usize; 5] = [0; 5];
    for i in 0..5 {
        length_args[i] = arguments.get(i).unwrap_or(&0).clone();
    }
    let mut error: usize;
    let mut result: usize;
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
pub unsafe fn syscall_abort() -> ! {
    unsafe {
        let _ = syscall(Selector::Abort, &[]);
    };
    panic!("thread did not abort!")
}
pub unsafe fn syscall_map(page_index: usize, page_count: usize) -> Result<(), ()> {
    unsafe { syscall(Selector::Map, &[page_index, page_count]) }.map(|_| ())
}
pub unsafe fn syscall_length(message_tag: usize) -> Result<usize, ()> {
    unsafe { syscall(Selector::Length, &[message_tag]) }
}
pub unsafe fn syscall_send(page_index: usize, page_count: usize) -> Result<usize, ()> {
    unsafe { syscall(Selector::Send, &[page_index, page_count]) }
}
pub unsafe fn syscall_query(message_tag: usize) -> Result<bool, ()> {
    unsafe { syscall(Selector::Query, &[message_tag]) }.map(|x| x != 0)
}
pub unsafe fn syscall_receive(message_tag: usize, page_index: usize) -> Result<(), ()> {
    unsafe { syscall(Selector::Receive, &[message_tag, page_index]) }.map(|_| ())
}
pub unsafe fn syscall_respond(
    server_tag: u64,
    message_tag: u64,
    page_index: usize,
    page_count: usize,
) -> Result<(), ()> {
    unsafe {
        syscall(
            Selector::Respond,
            &[
                server_tag as usize,
                message_tag as usize,
                page_index,
                page_count,
            ],
        )
        .map(|_| ())
    }
}
pub struct Buffer {
    page_index: usize,
    page_length: usize,
}
impl Buffer {
    unsafe fn syscall_map(self: &mut Self) -> Result<(), ()> {
        unsafe { syscall_map(self.page_index, self.page_length) }
    }
    pub fn new(page_length: usize) -> Buffer {
        let mut buffer = Buffer {
            page_index: NEXT_BUFFER_PAGE.fetch_add(page_length, Ordering::Relaxed),
            page_length: page_length,
        };
        unsafe { buffer.syscall_map() }.unwrap_or_else(|_| {
            panic!(
                "failed to map 0x{:x} pages at index 0x{:x} during buffer construction!",
                buffer.page_length, buffer.page_index
            )
        });
        buffer
    }
    pub fn len(self: &Self) -> usize {
        self.page_length * PAGE_SIZE
    }
    pub fn as_slice(self: &Self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                (self.page_index * PAGE_SIZE) as *const u8,
                self.page_length * PAGE_SIZE,
            )
        }
    }
    pub fn as_mut_slice(self: &mut Self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                (self.page_index * PAGE_SIZE) as *mut u8,
                self.page_length * PAGE_SIZE,
            )
        }
    }
    pub fn reconfigure(
        mut self: Self,
        first_page_length: usize,
        second_page_length: usize,
    ) -> (Buffer, Buffer) {
        if first_page_length + second_page_length < self.page_length {
            let mut unmapping_buffer = Buffer {
                page_index: self.page_index + first_page_length + second_page_length,
                page_length: self.page_length - first_page_length - second_page_length,
            };
            unsafe { unmapping_buffer.syscall_map() }.unwrap_or_else(|_| {
                panic!(
                    "failed to unmap 0x{:x} pages at index 0x{:x} during buffer shrinking!",
                    unmapping_buffer.page_index, unmapping_buffer.page_length
                )
            })
        } else if first_page_length + second_page_length > self.page_length {
            let mut mapping_buffer = Buffer {
                page_index: self.page_index + self.page_length,
                page_length: self.page_length - first_page_length - second_page_length,
            };
            unsafe { mapping_buffer.syscall_map() }.unwrap_or_else(|_| {
                unsafe {self.syscall_map()}.unwrap_or_else(|_| panic!(
                    "failed to unmap 0x{:x} pages at index 0x{:x} during buffer growing!",
                    self.page_index, self.page_length
                ));
                self = Buffer::new(first_page_length + second_page_length);
            })
        } else {
        }
        (
            Buffer {
                page_index: self.page_index,
                page_length: first_page_length,
            },
            Buffer {
                page_index: self.page_index + first_page_length,
                page_length: second_page_length,
            },
        )
    }
}
impl Drop for Buffer {
    fn drop(self: &mut Self) {
        unsafe {
            self.syscall_map().unwrap_or_else(|_| {
                panic!(
                    "failed to unmap 0x{:x} pages at index 0x{:x} during buffer dropping!",
                    self.page_length, self.page_index
                )
            });
        }
    }
}
#[repr(C, packed)]
pub struct State {
    walk: u8,
    make: u8,
    remove: u8,
    read: u8,
    insert: u8,
    overwrite: u8,
    truncate: u8,
    seek: u8,
    tell: u8,
    lock: u8,
}
pub struct Descriptor {
    index: u64,
}
impl Descriptor {
    pub fn read_state(self: &mut Self) -> Result<State, ()> {

    }
    pub fn write_state(self: &mut Self, state: &State) -> Result<(), ()> {

    }
    pub fn walk(self: &mut Self, path: &str) -> Result<Descriptor, ()> {

    }
    pub fn list(self: &mut Self, count: usize) -> Result<&mut [&mut str], ()> {

    }
    pub fn list_peek(self: &mut Self, count: usize) -> Result<&mut [&mut str], ()> {

    }
    pub fn list_seek_relative(self: &mut Self, offset: isize) -> Result<(), ()> {

    }
    pub fn list_seek_absolute(self: &mut Self, offset: isize) -> Result<(), ()> {
        
    }
    pub fn list_tell(self: &mut Self) -> usize {

    }
    pub fn make(self: &mut Self, child_state: &State, child_name: &str) -> Result<Descriptor, ()> {

    }
    pub fn remove(self: &mut Self, child_name: &str) -> Result<(), ()> {

    }
    pub fn read(self: &mut Self, length: usize) -> Result<&mut [u8], ()> {

    }
    pub fn peek(self: &mut Self, length: usize) -> Result<&mut [u8], ()> {

    }
    pub fn insert(self: &mut Self, content: Buffer, length: usize) -> Result<usize, ()> {

    }
    pub fn overwrite(self: &mut Self, content: Buffer, length: usize) -> Result<usize, ()> {

    }
    pub fn truncate
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        todo!()
    }
}
impl Clone for Descriptor {
    fn clone(&self) -> Self {
        Self { index: self.index.clone() }
    }
}
#[repr(C, packed)]
pub struct MessageHeader {
    pub length: u64,
    pub tag: u64,
    pub offset: u64,
}
extern fn main();
pub unsafe extern "C" fn _start(workspace_descriptor_index: u64) -> ! {
    let mut workspace_descriptor = Descriptor {
        index: workspace_descriptor_index,
    };
    panic!()
}
