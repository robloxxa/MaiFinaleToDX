use serial2::SerialPort;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub trait Port: io::Read + io::Write + Send + 'static {
    fn set_read_timeout(&mut self, timeout: Duration) -> io::Result<()>;
    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()>;
    fn discard_buffers(&self) -> io::Result<()>;
    fn discard_input_buffer(&self) -> io::Result<()>;
    fn discard_output_buffer(&self) -> io::Result<()>;
    fn try_clone(&self) -> io::Result<Box<dyn Port>>;
}

pub struct RealPort(SerialPort);

impl RealPort {
    pub fn open(name: &str, baud_rate: u32) -> io::Result<Self> {
        SerialPort::open(name, baud_rate).map(RealPort)
    }
}

impl io::Read for RealPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for RealPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Port for RealPort {
    fn set_read_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.0.set_read_timeout(timeout)
    }

    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.0.set_write_timeout(timeout)
    }

    fn discard_buffers(&self) -> io::Result<()> {
        self.0.discard_buffers()
    }

    fn discard_input_buffer(&self) -> io::Result<()> {
        self.0.discard_input_buffer()
    }

    fn discard_output_buffer(&self) -> io::Result<()> {
        self.0.discard_output_buffer()
    }

    fn try_clone(&self) -> io::Result<Box<dyn Port>> {
        self.0.try_clone().map(|p| Box::new(RealPort(p)) as Box<dyn Port>)
    }
}

struct MockPortInner {
    read_buf: Mutex<VecDeque<u8>>,
    read_condvar: Condvar,
    write_buf: Mutex<VecDeque<u8>>,
    read_timeout: Mutex<Duration>,
    write_timeout: Mutex<Duration>,
}

pub struct MockPort {
    inner: Arc<MockPortInner>,
}

impl MockPort {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MockPortInner {
                read_buf: Mutex::new(VecDeque::new()),
                read_condvar: Condvar::new(),
                write_buf: Mutex::new(VecDeque::new()),
                read_timeout: Mutex::new(Duration::ZERO),
                write_timeout: Mutex::new(Duration::ZERO),
            }),
        }
    }

    pub fn push_read_data(&self, data: &[u8]) {
        let mut buf = self.inner.read_buf.lock().unwrap();
        buf.extend(data);
        self.inner.read_condvar.notify_all();
    }

    pub fn take_write_data(&self) -> Vec<u8> {
        let mut buf = self.inner.write_buf.lock().unwrap();
        buf.drain(..).collect()
    }
}

impl io::Read for MockPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let timeout = *self.inner.read_timeout.lock().unwrap();
        let start = Instant::now();

        let mut read_buf = self.inner.read_buf.lock().unwrap();

        loop {
            let n = read_buf.len().min(buf.len());
            if n > 0 {
                for (i, b) in read_buf.drain(..n).enumerate() {
                    buf[i] = b;
                }
                return Ok(n);
            }

            if timeout.is_zero() {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "mock read timeout"));
            }

            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "mock read timeout"));
            }

            let (guard, result) = self.inner.read_condvar.wait_timeout(read_buf, remaining).unwrap();
            read_buf = guard;

            if result.timed_out() && read_buf.is_empty() {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "mock read timeout"));
            }
        }
    }
}

impl io::Write for MockPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut write_buf = self.inner.write_buf.lock().unwrap();
        write_buf.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Port for MockPort {
    fn set_read_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        *self.inner.read_timeout.lock().unwrap() = timeout;
        Ok(())
    }

    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        *self.inner.write_timeout.lock().unwrap() = timeout;
        Ok(())
    }

    fn discard_buffers(&self) -> io::Result<()> {
        self.inner.read_buf.lock().unwrap().clear();
        self.inner.write_buf.lock().unwrap().clear();
        Ok(())
    }

    fn discard_input_buffer(&self) -> io::Result<()> {
        self.inner.read_buf.lock().unwrap().clear();
        Ok(())
    }

    fn discard_output_buffer(&self) -> io::Result<()> {
        self.inner.write_buf.lock().unwrap().clear();
        Ok(())
    }

    fn try_clone(&self) -> io::Result<Box<dyn Port>> {
        Ok(Box::new(MockPort {
            inner: Arc::clone(&self.inner),
        }))
    }
}
