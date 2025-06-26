use thiserror::Error;

#[derive(Debug, Error)]
#[error("Invalid watermark position: '{0}'. Valid options are: nw, north, ne, west, center, east, sw, south, se")]
pub struct ParseWatermarkPositionError(pub String);