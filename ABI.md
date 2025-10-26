# tethys filesystem
this document is intended to be the authoritative specification of the tethys abi and filesystem, and should be used as reference material for implementation. "tethys" refers to a 9p-like messaging filesystem used for interprocess communication. it is split into "local" and "global" portions, the "local" portion consisting mainly of message buffer manipulation specific to the tethys operating system, and the "global" portion being filesystem messages intended to be os-independent. the tethys operating system prioritises capability-based security, and the global messages are structured to enable this. while messages are designed to allow the construction of a traditional filesystem, the only defined behaviour for a server's response to a message is that it responds with a valid success or error message (e.g. a server may pretend that file "foo" exists when responding to a **list**, but return an error upon attempting to **walk** to "foo"). the tethys operating system's virtual filesystem is simply a plan9-style mounting/binding/unioning of servers' filesystems, except in a heirarchical manner, and assumes the conventions laid out below in order to multiplex permissions based on server-provided file descriptor states. in the rust implementation, all messages require mutable references to a file, as servers may interpret messages however they see fit, including breaking traditional mutability / aliasing rules.

## glossary
- server -> serves a filesystem to clients by responding to messages, allowing access to resources
- client -> accesses a server's resourcesa by sending messages
- descriptor -> like a unix file descriptor, represents a communication channel between a client and a server
- state -> the possible operations available to a file descriptor. is a merging of both technical capability and permission, in order to only grant minimal privilege.
- walk -> moving a file descriptor along a relative path
- thread -> an instance of control flow executing semi-independently through a program.
- process -> a collection of threads sharing address space, tag space, and descriptor space.
- bind -> make one file or folder appear as if it also exists in another location.
- bindpoint -> the destination to which a file is bound.
- read/write head -> the point within a file that a descriptor is reading or writing from. moved by read, write, and seek, and not by peek.
- list head -> similar to read/write head but for the listing of a file's children (a folder's contents).
- zero-walk -> performing a **walk** call with an empty path, conventionally cloning the descriptor.

## file descriptor state
the tethys multiplexer handles states by simply taking the bitwise AND of the bindpoint's permissions and the file server's provided permissions. these permissions describe possible operations rather than specific file types, meaning e.g. a write-only file without walk permissions could represent both e.g. a logging system or e.g. a character device driver. a file descriptor usually has its working set of permissions (e.g. walk permissions at all times) and an invisible larger set of permissions that the server will allow but which are not enabled at a particular moment (write permissions enabled only when necessary, for performance), while the permissions attached to the bindpoint mask these to restrict privileges for children. all messages not listed here are always allowed.
a file descriptor state contains the following boolean fields, applying to the following messages:
### walk
applies to **walk**, **list**, **list_peek**, **list_seek_relative**, **list_seek_absolute**, and **list_tell**.
### make
applies to **make** and **bind**. partially applies to **list_write** which requires both **make** and **remove** permissions.
### remove
applies to **remove**. partially applies to **list_write** which requires both **make** and **remove** permissions.
### read
applies to **read** and **peek**.
### insert
applies to **insert**.
### overwrite
applies to **overwrite**.
### truncate
applies to the **truncate**.
### seek
applies to **seek_relative** and **seek_absolute**.
### tell
applies to **tell**.
### lock
exclusive access to a file. will not return a response until exclusive access is acquired.

## messages
### (rs) read_state(descriptor) -> State
get **descriptor**'s state.
### (ws) write_state(descriptor, state) -> ()
set **descriptor**'s state to **state**
### (dp) drop(descriptor) -> ()
drop a **descriptor**. **descriptor** is no longer valid following this call, and the specific value may refer to a different descriptor later.
### (wk) walk(old_descriptor, path) -> new_descriptor
get a new descriptor pointing to one of a descriptor's children. walking with no path ("zero-walk") conventionally clones the descriptor.
### (ls) list(descriptor, count) -> [\&str]
get the names of the next **count** children of **descriptor**.
### (lp) list_peek(descriptor, count) -> [\&str]
get the next **count** children of **descriptor** without advancing the list head forward.
### (lw) list_write(descriptor, old_name, new_name) -> ()
move **descriptor**'s child, named **old_name**, to be referred to as **new_name**.
### (lr) list_seek_relative(descriptor, offset) -> ()
move **descriptor**'s list head forward or backward by **offset** (signed).
### (la) list_seek_absolute(descriptor, offset) -> ()
move **descriptor**'s list head to the **offset**'th (signed) entry. negative values refer to indices starting from the opposite end of the list.
### (lt) list_tell(descriptor) -> u64
get the index of **descriptor**'s next child to be listed (e.g. 0 means that the first entry will be read next).
### (mk) make(descriptor, state, child_name) -> new_descriptor
create a new file under **descriptor** with state **state**, which will now be accessible under **new_descriptor**.
### (rm) remove(parent_descriptor, child_name) -> ()
remove a child of **parent_descriptor** with name **child_name**. 
### (rd) read(descriptor, length) -> (content)
get **length** bytes of data from **descriptor**. advances the read/head forward by **length** bytes. may return less than **length** bytes.
### (pk) peek(descriptor, length) -> (content)
get **length** bytes of data from **descriptor** without advancing the read/write head. may return less than **length** bytes.
### (in) insert(descriptor, content) -> length
insert **content** into **descriptor**, advancing the read/write head to the end of the inserted region. **length** represents the number of bytes actually written.
### (ov) overwrite(descriptor, content) -> length
insert **content** into **descriptor** over the top of any data that may already have been there, advancing the read/write head to the end of the overwritten region. **length** represents the number of bytes actually written.
### (tc) truncate(descriptor, length) -> length
remove **length** bytes from **descriptor** starting after the read/write head. this will effectively advance the read/write head to the end of the truncated region. may write less than **length** bytes.
### (sr) seek_relative(descriptor, offset) -> ()
move the read/write head of **descriptor** forward or backward by **offset** (signed).
### (sa) seek_absolute(descriptor, offset) -> ()
move the read/write head of **descriptor** to byte index **offset** (signed). a negative offset will refer to an index starting at the end of the file and growing backwards.
### (bd) bind(from_descriptor, to_descriptor, state, child_name) -> ()
local-exclusive. make **from_descriptor** available as /**to_descriptor**/**child_name**, with permissions no greater than **state**. internally clones the descriptor via zero-walk and stores the new descriptor in the kernel, to be cloned again for any new walks to the directory.

## universal syscalls
these are messages to the kernel, which multiplexes tethys filesystems. the tethys operating system's system calls are as follows:
### (ex) exit -> !
exit the current thread.
### (mp) map(index, count) -> ()
map **count** new blank pages to this process's address space starting at **index**, failing if the region overlaps with any other invalid or already-mapped regions.
### (sp) switch(from_index, count, to_index) -> ()
switch **count** pages starting at **from_index** to be mapped starting at **to_index**, and **count** pages starting at **to_index** to be mapped at **from_index**, preserving their content (but not modifying flags, e.g. the same content will now have swapped flags).
### (ln) length(tag) -> u64
return the length of a message from **tag**, in pages, blocking until it is ready.
## client syscalls
### (sd) send(index, count) -> tag
send **count** pages to the kernel starting from **index**. pages remain in the address space under a copy-on-write policy.
### (qy) query(tag) -> bool
queries whether the response to tag is available.
### (bk) block(tag, page_index) -> ()
maps the message **tag** into this process's address space starting at page **page_index**, blocking until it is ready. consumes the tag in the process.
## server syscalls
### (rs) respond(message_tag, page_index, page_count)
sends a response message starting at **page_index** of length **page_count** to message **message_tag**.
### (ck) check(server_tag) -> bool
checks whether a message to the server **server_tag** is available.
### (rc) receive(server_tag) -> tag
gets the tag of the next message sent to the server, consuming it from the queue and blocking until one is available.

## patterns
file servers may employ one or more common patterns to make complex behaviour cleaner. some are listed below:
### class-folder
a folder is used to mimic an object-oriented "class", with each **make** call representing the instantiation of a file as an object.