#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserToken(pub u64);
