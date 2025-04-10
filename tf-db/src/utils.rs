macro_rules! define_id {
	($name:ident) => {
		#[derive(
			Debug,
			Clone,
			Copy,
			PartialEq,
			Eq,
			Hash,
			PartialOrd,
			Ord,
			derive_more::FromStr,
			derive_more::Display,
			Deserialize,
			Serialize,
		)]
		pub struct $name(pub uuid::Uuid);
		impl $name {
			pub fn new() -> Self {
				Self(uuid::Uuid::new_v4())
			}
		}
		impl AsRef<[u8]> for $name {
			fn as_ref(&self) -> &[u8] {
				self.0.as_bytes()
			}
		}
		impl TryFrom<&[u8]> for $name {
			type Error = uuid::Error;
			fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
				uuid::Uuid::from_slice(value).map(Self)
			}
		}
	};
}
