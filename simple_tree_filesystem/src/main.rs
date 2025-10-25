use spinning_top::Spinlock;
const SIZE: usize = 3500;
const BLOCK_SIZE: usize = 4096;
static PARTITION: Spinlock<[u8; SIZE * BLOCK_SIZE]> = Spinlock::new([0; SIZE * BLOCK_SIZE]);
pub fn main() {
    
}