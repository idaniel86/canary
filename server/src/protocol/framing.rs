use prost::Message;
use tokio::io::AsyncReadExt;

/// Reads a length-delimited Protobuf message from an asynchronous reader.
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
    let mut buf: Vec<u8> = Vec::new();
    reader.read_buf(&mut buf).await?;

    let message = T::decode_length_delimited(buf.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(message)
}
