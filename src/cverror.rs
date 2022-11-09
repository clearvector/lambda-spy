use std::error;

pub type CvResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type MemResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type NetResult<T> = std::result::Result<T, Box<dyn error::Error>>;
