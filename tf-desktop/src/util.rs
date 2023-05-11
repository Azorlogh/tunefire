macro_rules! enum_lens (
	($variant:path) => {
		druid::lens::Map::new(
			|data| {
				let $variant(s) = data else { unreachable!() };
				s.clone()
			},
			|data, inner| {
				let $variant(s) = data else { unreachable!() };
				*s = inner
			}
		)
	};
);
