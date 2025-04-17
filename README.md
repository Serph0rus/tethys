# TETHYS
An operating system partially inspired by the likes of Multics and Plan 9 from Bell Labs. The microkernel handles interprocess communication, scheduling, and loading processes into memory. Ring 1 drivers adapt hardware into standard protocols, ring 2 drivers multiplex program I/O, and userspace programs' I/O is handled exclusively through IPC.

## CHECKLISTS
### KERNEL
[ ] Console I/O
[ ] Global Descriptor Table
[ ] Interrupt Descriptor Table
[ ] Paging
[ ] Memory Management
[ ] (Cooperative) Userland
[ ] Application Binary Interface
[ ] (Preemptive) Userland
[ ] Multiprocessing Structures

### PROGRAMS
[ ] Userland Console
[ ] Graphical User Interface
[ ] Permanent Filesystem
