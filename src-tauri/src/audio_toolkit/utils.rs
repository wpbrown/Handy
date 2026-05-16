/// Returns the appropriate CPAL host for the current platform.
///
/// The default host on Linux now prefers PipeWire when the `pipewire`
/// cpal feature is enabled (falling back to PulseAudio, then ALSA),
/// which matches our build configuration in `Cargo.toml`.
pub fn get_cpal_host() -> cpal::Host {
    let host = cpal::default_host();
    log::debug!("Selected CPAL host: {}", host.id());
    host
}
