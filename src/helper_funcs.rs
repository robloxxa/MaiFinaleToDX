use log::error;

pub fn bit_read(input: u8, n: usize) -> bool {
    input & (1 << n) != 0
}

pub fn log_error<E: std::fmt::Display>(err: E) -> E {
    error!("{}", err);
    err
}
