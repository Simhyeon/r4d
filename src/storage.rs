/// Result alias for storage operation
///
/// Error is a boxed container for generic error trait. Therefore any kind of errors can be
/// captured by storageresult.
pub type StorageResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Triat for storage interaction
///
/// Rad can utilizes storage to save given input as modified form and extract data from
///
/// # Example
///
/// ```rust
/// use r4d::{RadStorage, RadError, StorageOutput, StorageResult};
///
/// pub struct StorageDemo {
///     content: Vec<String>,
/// }
///
/// impl RadStorage for StorageDemo {
///     fn update(&mut self, args: &[String]) -> StorageResult<()> {
///         if args.is_empty() {
///             return Err(Box::new(RadError::InvalidArgument("Not enough arguments".to_string())));
///         }
///         self.content.push(args[0].clone());
///
///         Ok(())
///     }
///     fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>> {
///         let result = if serialize {
///             StorageOutput::Binary(self.content.join(",").as_bytes().to_vec())
///         } else {
///             StorageOutput::Text(self.content.join(","))
///         };
///         Ok(Some(result))
///     }
/// }
/// ```
pub trait RadStorage {
    /// Update storage with given arguments
    fn update(&mut self, args: &[String]) -> StorageResult<()>;
    /// Extract data from storage.
    ///
    /// # Args
    ///
    /// - serialize : whether to serialize storage output or not
    fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>>;
}

#[derive(Debug)]
/// Output that storage creates
pub enum StorageOutput {
    /// Binary form of output
    Binary(Vec<u8>),
    /// Text form of output
    Text(String),
}

impl StorageOutput {
    pub(crate) fn into_printable(self) -> String {
        match self {
            Self::Binary(bytes) => format!("{:?}", bytes),
            Self::Text(text) => text,
        }
    }
}
