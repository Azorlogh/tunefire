#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UserToken(pub u64);
