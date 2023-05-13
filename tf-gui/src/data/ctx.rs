// MIT License
// Copyright (c) 2020 Jan Pochyla
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// Modified by Alix Bott

use std::fmt;

use druid::{lens::Field, widget::ListIter, Data, Lens, LensExt};

#[derive(Clone, Data)]
pub struct Ctx<C, T> {
	pub ctx: C,
	pub data: T,
}

impl<C, T> Ctx<C, T>
where
	C: Data,
	T: Data,
{
	pub fn new(c: C, t: T) -> Self {
		Self { ctx: c, data: t }
	}

	pub fn make<S: Data>(cl: impl Lens<S, C>, tl: impl Lens<S, T>) -> impl Lens<S, Self> {
		CtxMake { cl, tl }
	}

	pub fn data() -> impl Lens<Self, T> {
		Field::new(|c: &Self| &c.data, |c: &mut Self| &mut c.data)
	}

	pub fn ctx() -> impl Lens<Self, C> {
		Field::new(|c: &Self| &c.ctx, |c: &mut Self| &mut c.ctx)
	}

	pub fn map<U>(map: impl Lens<T, U>) -> impl Lens<Self, Ctx<C, U>>
	where
		U: Data,
	{
		CtxMap { map }
	}
}

impl<C, L> Ctx<C, L>
where
	C: Data,
	L: Data,
{
	pub fn enumerate<T>() -> impl Lens<Self, CtxEnumerate<C, L>>
	where
		L: ListIter<T>,
	{
		druid::lens::Map::new(
			|s: &Ctx<C, L>| CtxEnumerate { inner: s.clone() },
			|s: &mut Ctx<C, L>, i: CtxEnumerate<C, L>| *s = i.inner,
		)
	}
}

#[derive(Clone, Data)]
pub struct CtxEnumerate<C, T> {
	inner: Ctx<C, T>,
}

impl<C, T, L> ListIter<Ctx<C, (usize, T)>> for CtxEnumerate<C, L>
where
	C: Data,
	T: Data,
	L: ListIter<T>,
{
	fn for_each(&self, mut cb: impl FnMut(&Ctx<C, (usize, T)>, usize)) {
		self.inner.data.for_each(|item, index| {
			let d = Ctx::new(self.inner.ctx.to_owned(), (index, item.to_owned()));
			cb(&d, index);
		});
	}

	fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Ctx<C, (usize, T)>, usize)) {
		let ctx = &mut self.inner.ctx;
		let data = &mut self.inner.data;
		data.for_each_mut(|item, index| {
			let mut d = Ctx::new(ctx.to_owned(), (index, item.to_owned()));
			cb(&mut d, index);
			if !ctx.same(&d.ctx) {
				*ctx = d.ctx;
			}
			if !item.same(&d.data.1) {
				*item = d.data.1;
			}
		});
	}

	fn data_len(&self) -> usize {
		self.inner.data.data_len()
	}
}

struct CtxMake<CL, TL> {
	cl: CL,
	tl: TL,
}

impl<C, T, S, CL, TL> Lens<S, Ctx<C, T>> for CtxMake<CL, TL>
where
	C: Data,
	T: Data,
	S: Data,
	CL: Lens<S, C>,
	TL: Lens<S, T>,
{
	fn with<V, F>(&self, data: &S, f: F) -> V
	where
		F: FnOnce(&Ctx<C, T>) -> V,
	{
		let c = self.cl.get(data);
		let t = self.tl.get(data);
		let ct = Ctx::new(c, t);
		f(&ct)
	}

	fn with_mut<V, F>(&self, data: &mut S, f: F) -> V
	where
		F: FnOnce(&mut Ctx<C, T>) -> V,
	{
		let c = self.cl.get(data);
		let t = self.tl.get(data);
		let mut ct = Ctx::new(c, t);
		let v = f(&mut ct);
		self.cl.put(data, ct.ctx);
		self.tl.put(data, ct.data);
		v
	}
}

struct CtxMap<Map> {
	map: Map,
}

impl<C, T, U, Map> Lens<Ctx<C, T>, Ctx<C, U>> for CtxMap<Map>
where
	C: Data,
	T: Data,
	U: Data,
	Map: Lens<T, U>,
{
	fn with<V, F>(&self, c: &Ctx<C, T>, f: F) -> V
	where
		F: FnOnce(&Ctx<C, U>) -> V,
	{
		self.map.with(&c.data, |u| {
			let cu = Ctx::new(c.ctx.to_owned(), u.to_owned());
			f(&cu)
		})
	}

	fn with_mut<V, F>(&self, c: &mut Ctx<C, T>, f: F) -> V
	where
		F: FnOnce(&mut Ctx<C, U>) -> V,
	{
		let t = &mut c.data;
		let c = &mut c.ctx;
		self.map.with_mut(t, |u| {
			let mut cu = Ctx::new(c.to_owned(), u.to_owned());
			let v = f(&mut cu);
			*c = cu.ctx;
			*u = cu.data;
			v
		})
	}
}

impl<C, T, L> ListIter<Ctx<C, T>> for Ctx<C, L>
where
	C: Data,
	T: Data,
	L: ListIter<T>,
{
	fn for_each(&self, mut cb: impl FnMut(&Ctx<C, T>, usize)) {
		self.data.for_each(|item, index| {
			let d = Ctx::new(self.ctx.to_owned(), item.to_owned());
			cb(&d, index);
		});
	}

	fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Ctx<C, T>, usize)) {
		let ctx = &mut self.ctx;
		let data = &mut self.data;
		data.for_each_mut(|item, index| {
			let mut d = Ctx::new(ctx.to_owned(), item.to_owned());
			cb(&mut d, index);
			if !ctx.same(&d.ctx) {
				*ctx = d.ctx;
			}
			if !item.same(&d.data) {
				*item = d.data;
			}
		});
	}

	fn data_len(&self) -> usize {
		self.data.data_len()
	}
}

impl<C, L> fmt::Debug for Ctx<C, L>
where
	L: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.data.fmt(f)
	}
}

impl<C, L> PartialEq for Ctx<C, L>
where
	L: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.data.eq(&other.data)
	}
}
