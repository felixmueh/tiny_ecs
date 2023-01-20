use crate::{AnyComponent, Ecs, Index, Storage};
use itertools::multizip;

pub unsafe trait Query<T> {
    type Item<'w>;

    fn query_mut<'w>(
        ecs: &'w mut Ecs,
    ) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>>;

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>>;
}

unsafe impl<T: 'static> Query<&T> for &T {
    type Item<'w> = &'w T;

    fn query_mut<'w>(
        ecs: &'w mut Ecs,
    ) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>> {
        let component = ecs.get_component::<T>()?;
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        Some(Box::new(component.iter().map(|x| x.as_ref())))
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>()?;
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        component.get(idx)?.as_ref()
    }
}

unsafe impl<T: 'static> Query<&mut T> for &mut T {
    type Item<'w> = &'w mut T;

    fn query_mut<'w>(
        ecs: &'w mut Ecs,
    ) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>> {
        let component = ecs.get_component_mut::<T>()?;
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        Some(Box::new(component.iter_mut().map(|x| x.as_mut())))
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>().unwrap();
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        component.get_mut(idx)?.as_mut()
    }
}

macro_rules! impl_queries {
    ($lc:tt $t:tt) => {};
    ($lc:tt $t:tt, $($lci:tt $i:tt),+) => {
        impl_query!($lc $t, $($lci $i),+);
        impl_queries!($($lci $i),+);
    };
}

macro_rules! impl_query {
    ($($lc:tt $t:tt),+) => {
        unsafe impl<$($t: Query<$t> + 'static),+> Query<($($t),+)> for ($($t),+) {
            type Item<'w> = ($($t::Item<'w>),+);

            fn query_mut<'w>(ecs: &'w mut Ecs) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>> {
                Some(Box::new(multizip(
                    ($($t::query_mut(unsafe { &mut *(ecs as *mut Ecs) })?),+)
                ).map(|($($lc),+)| Some(($($lc?),+)))))
            }

            fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
                Some((
                    ($($t::get_mut(unsafe { &mut *(ecs as *mut Ecs) }, idx)?),+)
                ))
            }
        }
    };
}

impl_queries! { a A, b B, c C, d D, e E, f F, g G, h H }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_teration() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let i1 = ecs.add_entity();
        let i2 = ecs.add_entity();
        ecs.set_entity(i1, 4_i64).unwrap();
        ecs.set_entity(i2, 5_i64).unwrap();

        for p in <&mut i64>::query_mut(&mut ecs).unwrap().flatten() {
            *p += 1;
        }

        let mut iter = <&i64>::query_mut(&mut ecs).unwrap();
        assert_eq!(*(iter.next().unwrap().unwrap()), 5_i64);
        assert_eq!(*(iter.next().unwrap().unwrap()), 6_i64);
    }

    #[test]
    fn test_tuple_teration() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<bool>();
        let i1 = ecs.add_entity();
        let i2 = ecs.add_entity();
        ecs.set_entity(i1, true).unwrap();
        ecs.set_entity(i1, 5_i64).unwrap();
        ecs.set_entity(i2, false).unwrap();
        ecs.set_entity(i2, 5_i64).unwrap();

        for (p, b) in <(&mut i64, &bool)>::query_mut(&mut ecs).unwrap().flatten() {
            if *b {
                *p += 1
            }
        }

        let mut iter = <&i64>::query_mut(&mut ecs).unwrap();
        assert_eq!(*(iter.next().unwrap().unwrap()), 6_i64);
        assert_eq!(*(iter.next().unwrap().unwrap()), 5_i64);
    }
}
