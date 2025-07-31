#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PofStatus {
    /// Whether POF warning is currently active
    pub warning_active: bool,
    /// Whether POF threshold has been configured
    pub threshold_configured: bool,
    /// Current VSYS voltage if available
    pub vsys_voltage: Option<f32>,
}
