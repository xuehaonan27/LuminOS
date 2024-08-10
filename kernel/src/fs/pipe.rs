use alloc::sync::{Arc, Weak};
use spin::Mutex;

use crate::{mm::UserBuffer, task::suspend_current_and_run_next};

use super::File;

const RING_BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Pipe {
    readable: bool,
    writable: bool,
    // The `buffer` field holds the strong reference to `PipeRingBuffer`.
    // After all `Pipe`s dropped, the `PipeRingBuffer` will be dropped
    // automatically.
    // So `PipeRingBuffer` must hold weak reference to `Pipe` to avoid
    // cyclic reference, thus preventing memory leak.
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, buf: UserBuffer) -> usize {
        assert!(self.readable());
        let want_to_read = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut already_read = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read_bytes();
            if loop_read == 0 {
                // fail to fetch content from the pipe.
                // first check whether all write ends closed.
                // if so, then there will not be any content to read.
                if ring_buffer.all_write_ends_closed() {
                    return already_read;
                }
                // if not, then wait until there's content to fetch.
                // drop the lock to avoid dead lock.
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    // there's still space to fill
                    unsafe { *byte_ref = ring_buffer.read_byte() };
                    already_read += 1;
                    if already_read == want_to_read {
                        return want_to_read;
                    }
                } else {
                    // no more buffer to be filled
                    return already_read;
                }
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        assert!(self.writable());
        let want_to_write = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut already_write = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write_bytes();
            if loop_write == 0 {
                // if there's no space to write, then wait
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref });
                    already_write += 1;
                    if already_write == want_to_write {
                        return want_to_write;
                    }
                } else {
                    return already_write;
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum RingBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}

/// Actual body of Pipe.
#[derive(Debug)]
pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    // Weak reference to avoid memory leak.
    write_end: Option<Weak<Pipe>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::EMPTY,
            write_end: None,
        }
    }
    /// Bind the write end to a `Pipe` struct
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        // downgrade to weak reference to avoid cyclic reference
        self.write_end = Some(Arc::downgrade(write_end));
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::NORMAL;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::EMPTY;
        }
        c
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::NORMAL;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::FULL;
        }
    }
    pub fn available_read_bytes(&self) -> usize {
        if self.status == RingBufferStatus::EMPTY {
            0
        } else {
            if self.tail > self.head {
                self.tail - self.head
            } else {
                self.tail + RING_BUFFER_SIZE - self.head
            }
        }
    }
    pub fn available_write_bytes(&self) -> usize {
        if self.status == RingBufferStatus::FULL {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read_bytes()
        }
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(Pipe::read_end_with_buffer(buffer.clone()));
    let write_end = Arc::new(Pipe::write_end_with_buffer(buffer.clone()));
    buffer.lock().set_write_end(&write_end);
    (read_end, write_end)
}
