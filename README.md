# rustsync

`rustsync` is a work in progress file transfer / backup tool similar to rsync.
 
Currently only local files and directories can be copied, and smart updating
based on mtime/checksum/etc. is not available. 
 
## Planned features
* Remote transfer using `ssh`
* Update mechanism based on
    * file checksum
    * last modified time
* compression when sending/receiving data from remote machines
