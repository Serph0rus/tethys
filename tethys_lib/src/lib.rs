#![no_std]
use core::arch::asm;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};
const PAGE_SIZE: usize = 4096;
const HEAP_PAGE_INDEX: usize = 0x0000_4000_0000;
const BUFFER_PAGE_INDEX: usize = 0x0000_8000_0000;
static NEXT_HEAP_PAGE: AtomicUsize = AtomicUsize::new(HEAP_PAGE_INDEX);
static NEXT_BUFFER_PAGE: AtomicUsize = AtomicUsize::new(BUFFER_PAGE_INDEX);
#[repr(usize)]
pub enum Syscall {
    Abort,
    Map,
    Switch,
    Length,
    Send,
    Query,
    Block,
    Respond,
    Check,
    Receive,
}
pub unsafe fn syscall(Syscall: Syscall, arguments: &[usize]) -> Result<usize, ()> {
    let mut length_args: [usize; 5] = [0; 5];
    for i in 0..5 {
        length_args[i] = arguments.get(i).unwrap_or(&0).clone();
    }
    let mut error: usize;
    let mut result: usize;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") Syscall as u64 => error,
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
        let _ = syscall(Syscall::Abort, &[]);
    };
    panic!("thread did not abort!")
}
pub unsafe fn syscall_map(page_index: usize, page_count: usize) -> Result<(), ()> {
    unsafe { syscall(Syscall::Map, &[page_index, page_count]) }.map(|_| ())
}
pub unsafe fn syscall_length(message_tag: usize) -> Result<usize, ()> {
    unsafe { syscall(Syscall::Length, &[message_tag]) }
}
pub unsafe fn syscall_send(page_index: usize, page_count: usize) -> Result<usize, ()> {
    unsafe { syscall(Syscall::Send, &[page_index, page_count]) }
}
pub unsafe fn syscall_query(message_tag: usize) -> Result<bool, ()> {
    unsafe { syscall(Syscall::Query, &[message_tag]) }.map(|x| x != 0)
}
pub unsafe fn syscall_block(message_tag: usize, page_index: usize) -> Result<bool, ()> {
    unsafe { syscall(Syscall::Block, &[message_tag, page_index]) }.map(|x| x != 0)
}
pub unsafe fn syscall_respond(
    server_tag: u64,
    message_tag: u64,
    page_index: usize,
    page_count: usize,
) -> Result<(), ()> {
    unsafe {
        syscall(
            Syscall::Respond,
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
pub unsafe fn syscall_check(server_tag: usize) -> Result<bool, ()> {
    unsafe { syscall(Syscall::Check, &[server_tag]) }.map(|x| x != 0)
}
pub unsafe fn syscall_receive(server_tag: usize) -> Result<usize, ()> {
    unsafe { syscall(Syscall::Receive, &[server_tag]) }
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
                unsafe { self.syscall_map() }.unwrap_or_else(|_| {
                    panic!(
                        "failed to unmap 0x{:x} pages at index 0x{:x} during buffer growing!",
                        self.page_index, self.page_length
                    )
                });
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
#[repr(usize)]
enum MsgSelector {
    ReadState,
    WriteState,
    Drop,
    Walk,
    List,
    ListPeek,
    ListSeekForward,
    ListSeekBackward,
    ListSeekStart,
    ListSeekEnd,
    ListTell,
    Make,
    Remove,
    Rename,
    Read,
    Peek,
    Insert,
    Overwrite,
    Truncate,
    SeekForward,
    SeekBackward,
    SeekStart,
    SeekEnd,
    Bind,
    Unmap,
}
pub struct State {
    walk: bool,
    rename: bool,
    make: bool,
    remove: bool,
    read: bool,
    insert: bool,
    overwrite: bool,
    truncate: bool,
    seek_forward: bool,
    seek_backward: bool,
    seek_start: bool,
    seek_end: bool,
    tell: bool,
    lock: bool,
}
macro_rules! state_chain {
    ($field:ident) => {
        const fn $field(mut self: Self, value: bool) -> State {
            self.$field = value;
            self
        }
    };
}
impl State {
    const fn new() -> State {
        State {
            walk: false,
            rename: false,
            make: false,
            remove: false,
            read: false,
            insert: false,
            overwrite: false,
            truncate: false,
            seek_forward: false,
            seek_backward: false,
            seek_start: false,
            seek_end: false,
            tell: false,
            lock: false,
        }
    }
    state_chain!(walk);
    state_chain!(rename);
    state_chain!(make);
    state_chain!(remove);
    state_chain!(read);
    state_chain!(insert);
    state_chain!(overwrite);
    state_chain!(truncate);
    state_chain!(seek_forward);
    state_chain!(seek_backward);
    state_chain!(seek_start);
    state_chain!(seek_end);
}
pub struct Descriptor {
    index: usize,
}
impl Descriptor {
    pub fn read_state(self: &mut Self) -> Result<State, ()> {
        let mut buffer = Buffer::new(1);
        buffer.as_mut_slice()[8..16].copy_from_slice(&(MsgSelector::ReadState as usize).to_le_bytes());
        let mut state = State::new()
    }
    pub fn write_state(self: &mut Self, state: &State) -> Result<(), ()> {}
    pub fn walk(self: &mut Self, path: &str) -> Result<Descriptor, ()> {}
    pub fn list(self: &mut Self, count: usize) -> Result<&mut [&mut str], ()> {}
    pub fn list_peek(self: &mut Self, count: usize) -> Result<&mut [&mut str], ()> {}
    pub fn list_seek_forward(self: &mut Self, offset: usize) -> Result<(), ()> {}
    pub fn list_seek_backward(self: &mut Self, offset: usize) -> Result<(), ()> {}
    pub fn list_seek_start(self: &mut Self, offset: usize) -> Result<(), ()> {}
    pub fn list_seek_end(self: &mut Self, offset: usize) -> Result<(), ()> {}
    pub fn list_tell(self: &mut Self) -> usize {}
    pub fn make(self: &mut Self, child_state: &State, child_name: &str) -> Result<Descriptor, ()> {}
    pub fn remove(self: &mut Self, child_name: &str) -> Result<(), ()> {}
    pub fn rename(self: &mut Self, new_name: &str) -> Result<(), ()> {}
    pub fn read(self: &mut Self, length: usize) -> Result<Buffer, ()> {}
    pub fn peek(self: &mut Self, length: usize) -> Result<Buffer, ()> {}
    pub fn insert(self: &mut Self, content: Buffer, length: usize) -> Result<usize, ()> {}
    pub fn overwrite(self: &mut Self, content: Buffer, length: usize) -> Result<usize, ()> {}
    pub fn truncate(self: &mut Self, length: usize) -> Result<usize, ()> {}
    pub fn seek_forward(self: &mut Self, offset: usize) -> Result<usize, ()> {}
    pub fn seek_backward(self: &mut Self, offset: usize) -> Result<usize, ()> {}
    pub fn seek_start(self: &mut Self, offset: usize) -> Result<usize, ()> {}
    pub fn seek_end(self: &mut Self, offset: usize) -> Result<usize, ()> {}
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        todo!()
    }
}
impl Clone for Descriptor {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
        }
    }
}
#[repr(C, packed)]
pub struct MessageHeader {
    pub length: u64,
    pub tag: u64,
    pub offset: u64,
}
#[macro_export]
macro_rules! entry {
    ($main_function:expr) => {
        unsafe extern "C" fn _start(workspace_descriptor_index: u64) -> ! {
            let mut workspace_descriptor = Descriptor {
                index: workspace_descriptor_index,
            };
            let main_function_checked: fn(Descriptor) = $main_function;
            main_function_checked(workspace_descriptor);
            panic!()
        }
    };
}
