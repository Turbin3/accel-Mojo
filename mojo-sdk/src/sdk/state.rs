//! Definitions for Mojo state types

use crate::errors::MojoSDKError;

/// Trait that all Mojo state types must implement
///
/// Defines how state types are serialized and deserialized when interacting with chain
/// @dev Whatever state structure, since you interact with chain
/// it has to be serializable and deserializable
pub trait MojoState: Sized {
    /// Serialize the state to bytes
    fn serialize(&self) -> Result<Vec<u8>, MojoSDKError>;

    /// Deserialize the state from bytes
    fn deserialize(data: &[u8]) -> Result<Self, MojoSDKError>;

    /// Get the size of the serialized state
    fn size(&self) -> usize {
        self.serialize().map(|data| data.len()).unwrap_or(0)
    }
}

/// Helper macro acts as a wrapper for bytemuck
#[macro_export]
macro_rules! impl_mojo_state_pod {
    ($type:ty) => {
        impl $crate::MojoState for $type {
            fn serialize(&self) -> $crate::Result<Vec<u8>> {
                Ok(bytemuck::bytes_of(self).to_vec())
            }

            fn deserialize(data: &[u8]) -> $crate::Result<Self> {
                bytemuck::try_from_bytes(data)
                    .map(|p: &Self| *p)
                    .map_err(|e| {
                        $crate::MojoError::Deserialization(format!("Failed to deserialize: {}", e))
                    })
            }
        }
    };
}
