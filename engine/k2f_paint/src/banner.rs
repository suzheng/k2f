use k2f_package::IntegrityStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Banner {
    Signed,
    Unsigned,
    SignedButBroken,
    BrokenIntegrity,
    Unlocked,
}

impl Banner {
    pub fn from_integrity(status: IntegrityStatus) -> Self {
        match status {
            IntegrityStatus::Signed => Banner::Signed,
            IntegrityStatus::Unsigned => Banner::Unsigned,
            IntegrityStatus::SignedButBroken => Banner::SignedButBroken,
            IntegrityStatus::Unlocked => Banner::Unlocked,
            _ => Banner::BrokenIntegrity,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Banner::Signed => "SIGNED",
            Banner::Unsigned => "UNSIGNED",
            Banner::SignedButBroken => "SIGNED_BUT_BROKEN",
            Banner::BrokenIntegrity => "BROKEN_INTEGRITY",
            Banner::Unlocked => "UNLOCKED",
        }
    }
}
