use crate::{AnyComponent, Ecs, Index, Storage};
use itertools::multizip;

pub unsafe trait Query<T> {
    type Item<'w>;

    fn query_mut<'w>(ecs: &'w mut Ecs) -> Box<dyn Iterator<Item = Self::Item<'w>> + 'w>;

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>>;
}

unsafe impl<T: 'static> Query<&T> for &T {
    type Item<'w> = &'w Option<T>;

    fn query_mut<'w>(ecs: &'w mut Ecs) -> Box<dyn Iterator<Item = Self::Item<'w>> + 'w> {
        let component = ecs.get_component::<T>().unwrap();
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        Box::new(component.iter())
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>().unwrap();
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        component.get(idx)
    }
}

unsafe impl<T: 'static> Query<&mut T> for &mut T {
    type Item<'w> = &'w mut Option<T>;

    fn query_mut<'w>(ecs: &'w mut Ecs) -> Box<dyn Iterator<Item = Self::Item<'w>> + 'w> {
        let component = ecs.get_component_mut::<T>().unwrap();
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        Box::new(component.iter_mut())
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>().unwrap();
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        component.get_mut(idx)
    }
}

macro_rules! impl_queries {
    ($t:tt) => {};
    ($t:tt, $($i:tt),+) => {
        impl_query!($t, $($i),+);
        impl_queries!($($i),+);
    };
}

macro_rules! impl_query {
    ($($t:tt),+) => {
        unsafe impl<$($t: Query<$t> + 'static),+> Query<($($t),+)> for ($($t),+) {
            type Item<'w> = ($($t::Item<'w>),+);

            fn query_mut<'w>(ecs: &'w mut Ecs) -> Box<dyn Iterator<Item = Self::Item<'w>> + 'w> {
                Box::new(multizip((
                    ($($t::query_mut(unsafe { &mut *(ecs as *mut Ecs) })),+)
                )))
            }

            fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
                Some((
                    ($($t::get_mut(unsafe { &mut *(ecs as *mut Ecs) }, idx)?),+)
                ))
            }
        }
    };
}

impl_queries! { A, B, C, D, E, F, G, H }

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

        for p in <&mut i64>::query_mut(&mut ecs) {
            *p.as_mut().unwrap() += 1;
        }

        let mut iter = <&i64>::query_mut(&mut ecs);
        assert_eq!((iter.next().unwrap().unwrap()), 5_i64);
        assert_eq!((iter.next().unwrap().unwrap()), 6_i64);
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

        for (p, b) in <(&mut i64, &bool)>::query_mut(&mut ecs) {
            if *b.as_ref().unwrap() {
                *p.as_mut().unwrap() += 1
            }
        }

        let mut iter = <&i64>::query_mut(&mut ecs);
        assert_eq!((iter.next().unwrap().unwrap()), 6_i64);
        assert_eq!((iter.next().unwrap().unwrap()), 5_i64);
    }
}
