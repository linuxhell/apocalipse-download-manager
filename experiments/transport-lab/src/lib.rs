pub mod dictionary;
#[cfg(feature = "moqt")]
pub mod moqt;
pub mod multipath;

pub(crate) fn init_crypto() {
    // MoQ and the direct QUIC experiment compile different crypto backends.
    // Select one process provider explicitly before either client is built.
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
