#![no_std]
extern crate alloc;
use spinning_top::Spinlock;
use hashbrown::{DefaultHashBuilder, HashMap};
use lazy_static::lazy_static;
const SIZE: usize = 3500;
const BLOCK_SIZE: usize = 4096;
static PARTITION: Spinlock<[u8; SIZE * BLOCK_SIZE]> = Spinlock::new([0; SIZE * BLOCK_SIZE]);
struct DescriptorData {

}
lazy_static! {
    static ref DESCRIPTOR_DATA: HashMap<usize, DescriptorData> = HashMap::new();
}
pub fn main() {

}