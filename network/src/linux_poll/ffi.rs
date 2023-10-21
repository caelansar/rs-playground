pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLLIN: i32 = 0x1;
pub const EPOLLONESHOT: i32 = 0x40000000;

/// Since the same name is used multiple times, it can be confusing but we have an `Event` structure.
/// This structure ties a file descriptor and a field called `events` together. The field `events` holds information
/// about what events are ready for that file descriptor.
#[repr(C, packed)]
pub struct Event {
    /// This can be confusing, but this is the events that are ready on the file descriptor.
    events: u32,
    epoll_data: usize,
}

impl Event {
    pub fn new(events: i32, id: usize) -> Self {
        Event {
            events: events as u32,
            epoll_data: id,
        }
    }
    pub fn data(&self) -> usize {
        self.epoll_data
    }
}

#[link(name = "c")]
extern "C" {
    pub fn epoll_create(size: i32) -> i32;

    pub fn close(fd: i32) -> i32;

    pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;

    /// - epoll_event is a pointer to an array of Events
    /// - timeout of -1 means indefinite
    pub fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;

    pub fn eventfd(initva: u32, flags: i32) -> i32;
}
