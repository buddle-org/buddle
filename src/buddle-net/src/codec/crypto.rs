/// How encryption of data frames should be handled by the
/// codec implementation.
pub enum EncryptionMode {
    /// Always encrypt all frames and exchange corresponding
    /// key material.
    Always,
    /// Never encrypt any frames.
    Never,
}
