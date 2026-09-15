//! Operations: one intent = one command = one transaction (D-69, D-75). The
//! closed set and its behavior arrive with N-37; this is their boundary.
pub trait Intent {
    fn name(&self) -> &'static str;
}
