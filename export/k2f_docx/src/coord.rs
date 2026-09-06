pub const EMU_PER_MILLIPT_NUM: i64 = 127;
pub const EMU_PER_MILLIPT_DEN: i64 = 10;

pub fn millipt_to_emu(millipt: i64) -> i64 {
    millipt.saturating_mul(EMU_PER_MILLIPT_NUM) / EMU_PER_MILLIPT_DEN
}

pub fn pt_to_emu(pt: k2f_core::Pt) -> i64 {
    millipt_to_emu(i64::try_from(pt.0).unwrap_or(i64::MAX))
}

pub fn millipt_to_twips(millipt: i64) -> i64 {
    millipt / 50
}

pub fn emu_to_twips(emu: i64) -> i64 {
    emu / 635
}

pub fn pt_to_twips(pt: k2f_core::Pt) -> i64 {
    millipt_to_twips(i64::try_from(pt.0).unwrap_or(i64::MAX))
}
