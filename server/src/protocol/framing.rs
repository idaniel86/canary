use prost::Message;
use tokio::io::AsyncReadExt;

/// Reads a single Protobuf varint from an asynchronous reader, one byte at a time.
async fn read_varint<R>(reader: &mut R) -> std::io::Result<u64>
where
    R: AsyncReadExt + Unpin,
{
    let mut value: u64 = 0;
    let mut shift = 0u32;
    loop {
        let byte = reader.read_u8().await?;
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
}

/// Reads a length-delimited Protobuf message from an asynchronous reader.
///
/// A single `read`/`read_buf` call over a TCP stream is not guaranteed to return a
/// whole message, so the varint length prefix is parsed first and then exactly that
/// many bytes are read via `read_exact` before decoding.
///
/// # Arguments
/// * `reader` - A mutable reference to an object that implements `AsyncReadExt` and `Unpin`.
/// * `T` - The type of the Protobuf message to be read, which must implement `Message` and `Default`.
///
/// # Returns
/// * `Result<T, crate::error::Error>` - Returns the decoded Protobuf message of type `T` on success, or an error if reading or decoding fails.
pub async fn read_message<R, T>(reader: &mut R) -> Result<T, crate::error::Error>
where
    R: AsyncReadExt + Unpin,
    T: Message + Default,
{
    let len = read_varint(reader).await?;

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;

    let message = T::decode(buf.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(message)
}
