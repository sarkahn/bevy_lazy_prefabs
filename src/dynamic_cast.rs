use bevy::reflect::{Reflect, DynamicStruct, Struct};



pub trait DynamicCast: Reflect {
    /// Downcast to `&T` and unwrap immediately. Will panic if
    /// given the wrong type.
    fn cast_ref<T: Reflect>(&self) -> &T;
    /// Downcast to `&mut T` and unwrap immediately. Will panic if given
    /// the wrong type.
    fn cast_mut<T: Reflect>(&mut self) -> &mut T;
}

impl DynamicCast for dyn Reflect {
    fn cast_ref<T: Reflect>(&self) -> &T {
        self.downcast_ref::<T>().unwrap()
    }

    fn cast_mut<T: Reflect>(&mut self) -> &mut T {
        self.downcast_mut::<T>().unwrap()
    }
}

pub trait GetValue {
    /// Retrieves the given value from a field and unwraps immediately.
    /// Will panic if given the wrong type or the field doesn't exist.
    fn get<T: Reflect>(&self, field_name: &str) -> &T;
}

impl GetValue for DynamicStruct {
    fn get<T: Reflect>(&self, field_name: &str) -> &T {
        self.field(field_name).unwrap().downcast_ref::<T>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Reflect)]
    struct Test {
        i: i32,
        q: i32,
    }

    impl Default for Test {
        fn default() -> Self {
            Self { i: 0, q: 99 }
        }
    }
    
    #[test]
    fn cast() {
        let a = Test {
            i: 5,
            q: 10,
        };
        let a: Box<dyn Reflect> = Box::new(a);

        let a = a.cast_ref::<Test>();

        assert_eq!(a.i, 5);
        assert_eq!(a.q, 10);
    }
    
    #[test]
    fn auto_cast() {
        let a = Test { i: 15, q: 25 };
        let b = a.clone_dynamic();
        let bi = b.get::<i32>("i");

        assert_eq!(bi, &15);
    }
    

    
}