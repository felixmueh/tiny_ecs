use crate::{prelude::*, Component};
use crate::{AnyComponent, Ecs, Index, Storage};
use itertools::multizip;

pub unsafe trait Set<T> {
    fn set(ecs: &mut Ecs, index: Index, value: T) -> Result<()>;
    fn set_or_create(ecs: &mut Ecs, index: Index, value: T) -> Result<()>;
}

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
        component.get(idx).ok()?.as_ref()
    }
}

unsafe impl<T: 'static> Set<T> for T
where
    T: Component,
{
    fn set(ecs: &mut Ecs, index: Index, value: T) -> Result<()> {
        let component = ecs.get_component_mut::<T>().ok_or("Can't find component")?;
        component.set(index, Some(value))
    }

    fn set_or_create(ecs: &mut Ecs, index: Index, value: T) -> Result<()> {
        let component = match ecs.get_component_mut::<T>() {
            Some(component) => component,
            None => {
                ecs.add_component::<T>();
                ecs.get_component_mut::<T>().unwrap()
            }
        };
        component.set(index, Some(value))
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
        component.get_mut(idx).ok()?.as_mut()
    }
}

unsafe impl<T: 'static> Query<Option<&T>> for Option<&T> {
    type Item<'w> = &'w Option<T>;

    fn query_mut<'w>(
        ecs: &'w mut Ecs,
    ) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>> {
        let component = ecs.get_component::<T>().unwrap();
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        Some(Box::new(component.iter().map(Some)))
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>()?;
        let component = component.as_any().downcast_ref::<Storage<T>>().unwrap();
        component.get(idx).ok()
    }
}

unsafe impl<T: 'static> Query<Option<&mut T>> for Option<&mut T> {
    type Item<'w> = &'w mut Option<T>;

    fn query_mut<'w>(
        ecs: &'w mut Ecs,
    ) -> Option<Box<dyn Iterator<Item = Option<Self::Item<'w>>> + 'w>> {
        let component = ecs.get_component_mut::<T>().unwrap();
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        Some(Box::new(component.iter_mut().map(Some)))
    }

    fn get_mut(ecs: &mut Ecs, idx: Index) -> Option<Self::Item<'_>> {
        let component = ecs.get_component_mut::<T>()?;
        let component = component.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        component.get_mut(idx).ok()
    }
}

macro_rules! impl_traits {
    ($lc:tt $t:tt) => {};
    ($lc:tt $t:tt, $($lci:tt $i:tt),+) => {
        impl_query!($lc $t, $($lci $i),+);
        impl_set!($lc $t, $($lci $i),+);
        impl_traits!($($lci $i),+);
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

macro_rules! impl_set {
    ($($lc:tt $t:tt),+) => {
        unsafe impl<$($t: Set<$t> + 'static),+> Set<($($t),+)> for ($($t),+)
        {
            fn set(ecs: &mut Ecs, index: Index, value: ($($t),+)) -> Result<()> {
                let ($($lc),+) = value;
                $(<$t>::set(ecs, index, $lc)?;)+
                Ok(())
            }
            fn set_or_create(ecs: &mut Ecs, index: Index, value: ($($t),+)) -> Result<()> {
                let ($($lc),+) = value;
                $(<$t>::set_or_create(ecs, index, $lc)?;)+
                Ok(())
            }
        }
    };
}

impl_traits! { a A, b B, c C, d D, e E, f F, g G, h H }

#[cfg(test)]
mod tests {
    use component_derive::Component;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Component)]
    struct HP(u32);

    #[derive(Debug, PartialEq, Eq, Component)]
    struct IS(bool);

    #[test]
    fn test_set_value() {
        let mut ecs = Ecs::new();
        ecs.add_component::<u64>();
        let index = ecs.new_empty_entity();
        <(HP, IS)>::set_or_create(&mut ecs, index, (HP(42), IS(true))).unwrap();
    }

    #[test]
    fn test_get_optional_component() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let i1 = ecs.new_empty_entity();

        let opt = <Option<&mut i64>>::get_mut(&mut ecs, i1).unwrap();
        opt.replace(42);
        assert_eq!(42, *<&i64>::get_mut(&mut ecs, i1).unwrap());
    }

    #[test]
    fn test_basic_teration() {
        let mut ecs = Ecs::new();
        ecs.add_component::<HP>();
        let i1 = ecs.new_empty_entity();
        let i2 = ecs.new_empty_entity();
        ecs.set_entity_create_components(i1, HP(4)).unwrap();
        ecs.set_entity_create_components(i2, HP(5)).unwrap();

        for p in <&mut HP>::query_mut(&mut ecs).unwrap().flatten() {
            p.0 += 1;
        }

        let mut iter = <&HP>::query_mut(&mut ecs).unwrap();
        assert_eq!(*(iter.next().unwrap().unwrap()), HP(5));
        assert_eq!(*(iter.next().unwrap().unwrap()), HP(6));
    }

    #[test]
    fn test_tuple_teration() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<bool>();
        let i1 = ecs.new_empty_entity();
        let i2 = ecs.new_empty_entity();
        ecs.set_entity_create_components(i1, IS(true)).unwrap();
        ecs.set_entity_create_components(i1, HP(5)).unwrap();
        ecs.set_entity_create_components(i2, IS(false)).unwrap();
        ecs.set_entity_create_components(i2, HP(5)).unwrap();

        for (p, b) in <(&mut HP, &IS)>::query_mut(&mut ecs).unwrap().flatten() {
            if b.0 {
                p.0 += 1
            }
        }

        let mut iter = <&HP>::query_mut(&mut ecs).unwrap();
        assert_eq!(*(iter.next().unwrap().unwrap()), HP(6));
        assert_eq!(*(iter.next().unwrap().unwrap()), HP(5));
    }
}
