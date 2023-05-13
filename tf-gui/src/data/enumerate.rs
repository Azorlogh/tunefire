use druid::{lens, widget::ListIter, Data, Lens};

pub fn lens_enumerate<T, L>() -> impl Lens<L, Enumerate<L>>
where
	L: ListIter<T>,
{
	druid::lens::Map::new(
		|s: &L| Enumerate { inner: s.clone() },
		|s: &mut L, i: Enumerate<L>| *s = i.inner,
	)
}

pub fn deenumerate<T>() -> impl Lens<(usize, T), T> {
	lens::Field::new(|(_, x): &_| x, |(_, x): &mut _| x)
}

#[derive(Clone, Data)]
pub struct Enumerate<T> {
	inner: T,
}

impl<T, L> ListIter<(usize, T)> for Enumerate<L>
where
	T: Data,
	L: ListIter<T>,
{
	fn for_each(&self, mut cb: impl FnMut(&(usize, T), usize)) {
		self.inner.for_each(|item, index| {
			let d = (index, item.to_owned());
			cb(&d, index);
		});
	}

	fn for_each_mut(&mut self, mut cb: impl FnMut(&mut (usize, T), usize)) {
		self.inner.for_each_mut(|item, index| {
			let mut d = (index, item.to_owned());
			cb(&mut d, index);
			if !item.same(&d.1) {
				*item = d.1;
			}
		});
	}

	fn data_len(&self) -> usize {
		self.inner.data_len()
	}
}
