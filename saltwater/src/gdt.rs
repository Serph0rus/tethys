use core::{arch::asm, mem::MaybeUninit};
use x86_64::registers::segmentation::Segment;

use crate::{mapping, println};
#[repr(C, packed)]
struct TaskStateSegment {
    reserved_0: u32,
    pub privilege_stacks: [u64; 3],
    reserved_1: u64,
    pub interrupt_stacks: [u64; 7],
    reserved_2: u64,
    reserved_3: u16,
    io_permission_bitmap_base: u16,
}
#[repr(C, packed)]
struct DescriptorAccess(u8);
#[repr(C, packed)]
struct DescriptorFlags(u8);
#[repr(C, packed)]
struct Descriptor(u64);
#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}
#[repr(C, packed)]
struct SegmentSelector(u16);
impl TaskStateSegment {
    const fn new(
        privilege_stacks: [u64; 3],
        interrupt_stacks: [u64; 7],
    ) -> TaskStateSegment {
        TaskStateSegment {
            reserved_0: 0,
            privilege_stacks: privilege_stacks,
            reserved_1: 0,
            interrupt_stacks: interrupt_stacks,
            reserved_2: 0,
            reserved_3: 0,
            io_permission_bitmap_base: size_of::<TaskStateSegment>() as u16,
        }
    }
}
impl DescriptorAccess {
    const fn new(
        accessed: bool,
        read_write: bool,
        direction_conforming: bool,
        executable: bool,
        non_system: bool,
        user_mode: bool,
        present: bool,
    ) -> DescriptorAccess {
        DescriptorAccess(
              ((accessed as u8) << 0)
            | ((read_write as u8) << 1)
            | ((direction_conforming as u8) << 2)
            | ((executable as u8) << 3)
            | ((non_system as u8) << 4)
            | (if user_mode {0b11} else {0} << 5)
            | ((present as u8) << 7)
        )
    }
}
impl DescriptorFlags {
    const fn new(
        available: bool,
        long: bool,
        size: bool,
        granularity: bool,
    ) -> DescriptorFlags {
        DescriptorFlags(
              ((available as u8) << 0)
            | ((long as u8) << 1)
            | ((size as u8) << 2)
            | ((granularity as u8) << 3)
        )
    }
}
impl Descriptor {
    const fn new(
        base: u32,
        limit: u32,
        access: DescriptorAccess,
        flags: DescriptorFlags,
    ) -> Descriptor {
        Descriptor(
              (limit as u64 & 0xffff << 0) << 0
            | (base as u64 & 0xffff << 0) << 16
            | (base as u64 & 0xf << 16) << 16
            | (access.0 as u64 & 0xff << 0) << 40
            | (limit as u64 & 0xf << 16) << 32
            | (flags.0 as u64 & 0xf << 0) << 52
            | (base as u64 & 0xff << 24) << 32
        )
    }
    const fn null() -> Descriptor {
        Descriptor(0)
    }
    fn tss(tss: *mut TaskStateSegment) -> (Descriptor, Descriptor) {
        let ptr = tss as *mut TaskStateSegment as u64;
        (
            Descriptor::new(
                (ptr & u32::MAX as u64) as u32,
                (size_of::<TaskStateSegment>() - 1) as u32,
                DescriptorAccess(
                    0x9
                    | DescriptorAccess::new(
                          false,
                          false,
                          false,
                          false,
                          false,
                          false,
                          true,
                      ).0 & (0xf << 4)
                ),
                DescriptorFlags::new(
                    false,
                    false,
                    false,
                    false,
                )
            ),
            Descriptor((ptr & (u32::MAX as u64) << 32) >> 32),
        )
    }
}
impl GdtDescriptor {
    fn new(base: u64, entry_count: usize) -> GdtDescriptor {
        GdtDescriptor {
            limit: (entry_count * size_of::<Descriptor>() - 1) as u16,
            base: base,
        }
    }
}
impl SegmentSelector {
    const fn new(index: u16, is_user: bool) -> SegmentSelector {
        SegmentSelector(index << 3 | if is_user {3} else {0})
    }
}
const KERNEL_CODE_DESCRIPTOR: Descriptor = Descriptor::new(
    u32::MIN,
    u32::MAX,
    DescriptorAccess::new(
        true,
        true,
        false,
        true,
        true,
        false,
        true,
    ),
    DescriptorFlags::new(
        false,
        true,
        false,
        true,
    )
);
const KERNEL_DATA_DESCRIPTOR: Descriptor = Descriptor::new(
    u32::MIN,
    u32::MAX,
    DescriptorAccess::new(
        true,
        true,
        false,
        false,
        true,
        false,
        true,
    ),
    DescriptorFlags::new(
        false,
        false,
        false,
        true,
    ),
);
static mut BOOTSTRAP_TSS: TaskStateSegment = TaskStateSegment::new(
    [mapping::BOOTSTRAP_STACK; 3],
    [mapping::BOOTSTRAP_STACK; 7],
);
const BOOTSTRAP_GDT_COUNT: usize = 5;
static mut BOOTSTRAP_GDT: [MaybeUninit<Descriptor>; BOOTSTRAP_GDT_COUNT] = [
    MaybeUninit::new(Descriptor::null()),
    MaybeUninit::new(KERNEL_CODE_DESCRIPTOR),
    MaybeUninit::new(KERNEL_DATA_DESCRIPTOR),
    MaybeUninit::uninit(),
    MaybeUninit::uninit(),
];
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, false);
const KERNEL_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(2, false);
pub fn bootstrap_initialise(_boot_info: &mut bootloader_api::BootInfo) {
    unsafe {
        BOOTSTRAP_GDT[3] = MaybeUninit::new(Descriptor::tss(&raw mut BOOTSTRAP_TSS).0);
        BOOTSTRAP_GDT[4] = MaybeUninit::new(Descriptor::tss(&raw mut BOOTSTRAP_TSS).1);
        println!("constructed bootstrap global descriptor table...");
        asm!("lgdt [{}]", in(reg) &GdtDescriptor::new(&raw mut BOOTSTRAP_GDT as u64, BOOTSTRAP_GDT_COUNT), options(readonly, nostack, preserves_flags));
        println!("loaded bootstrap global descriptor table...");
        x86_64::registers::segmentation::CS::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_CODE_SELECTOR.0));
        println!("far-returned into new code segment...");
        x86_64::registers::segmentation::DS::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0));
        x86_64::registers::segmentation::ES::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0));
        x86_64::registers::segmentation::FS::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0));
        x86_64::registers::segmentation::GS::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0));
        x86_64::registers::segmentation::SS::set_reg(x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0));
        println!("reloaded data segment registers...");
    }
}
