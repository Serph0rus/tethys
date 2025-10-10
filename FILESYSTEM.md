# tethys filesystem
this document is intended to be the authoritative specification of the tethys filesystem, and should be used as reference material for implementation. "tethys" refers to a 9p-like messaging filesystem used for interprocess communication. it is split into "local" and "global" portions, the "local" portion consisting mainly of message buffer manipulation specific to the tethys operating system, and the "global" portion being filesystem messages intended to be os-independent. the tethys operating system prioritises capability-based security, and the global messages are structured to enable this. while messages are designed to allow the construction of a traditional filesystem, the only defined behaviour for a server's response to a message is that it responds with a valid success or error message (e.g. a server may pretend that file "foo" exists when responding to a **list**, but return an error upon attempting to **walk** to "foo"). the tethys operating system's virtual filesystem is simply a plan9-style mounting/binding/unioning of servers' filesystems, except in a heirarchical manner, and assumes the conventions laid out below in order to multiplex permissions based on server-provided file descriptor states. in the rust implementation, all messages require mutable references to a file, as servers may interpret messages however they see fit, including breaking traditional mutability / aliasing rules.

## glossary
- server -> serves a filesystem to clients by responding to messages, allowing access to resources
- client -> accesses a server's resourcesa by sending messages
- descriptor -> like a unix file descriptor, represents a communication channel between a client and a server
- state -> the possible operations available to a file descriptor. is a merging of both technical capability and permission, in order to only grant minimal privilege.
- walk -> moving a file descriptor along a relative path

## file descriptor state
the tethys multiplexer handles states by simply taking the bitwise AND of the mountpoint's permissions, the process's permissions, and the file server's provided permissions. these permissions describe possible operations rather than specific file types, meaning e.g. a write-only file without walk permissions could represent both a log or a character device driver. a file descriptor state contains the following boolean fields:
### walk
discover the structure of a file descriptor's children. applies to the messages **walk**, **list**, **list_peek**, **list_seek_relative**, **list_seek_absolute**, and **list_tell**.
### make
create a new file, applies to the **make** message
### remove
remove a file, applies to the **remove** message
### read
read data from a descriptor, applies to the **read** message
### peek
read data from a descriptor without moving its position within a file. applies to the **peek** message
### write
write data to a descriptor, applies to the **write** message
### truncate
remove data following a descriptor's position within a file. applies to the **truncate** message.
### seek
change a descriptor's position within a file. applies to the **seek_relative** and **seek_absolute** messages.
### tell
read a descriptor's position within a file. applies to the **tell** message.
### lock
exclusive access to a file. will not return a response until exclusive access is acquired.
### serve
local-exclusive. permission to add files as children to the current file that the server will not be able to see, either as a process creating its own server or mounting a server it has access to from elsewhere.
