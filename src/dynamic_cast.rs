//! Utility traits for easily retrieving values from [Reflect] components.

use bevy::reflect::{DynamicStruct, GetTypeRegistration, Reflect, Struct};
use thiserror::Error;

/// A utility trait for easily casting [Reflect] components to an underlying type.
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

/// Errors returned from the [GetValue] trait.
#[derive(Error, Debug)]
pub enum GetValueError {
    #[error("The field {0} doesn't exist on the reflected type {1}")]
    FieldDoesntExist(String, String),
    #[error("The type {0} failed to downcast into the type {1}")]
    FailedCast(String, String),
}

/// A utility trait for easily retrieving the value of a field from a [DynamicStruct].
pub trait GetValue {
    /// Retrieves a reference to the given type from a field and unwraps immediately.
    /// Will panic if given the wrong type or the field doesn't exist.
    fn get<T: Reflect>(&self, field_name: &str) -> &T;

    /// Tries to retrieve a reference to the field value of the given type.
    fn try_get<T: Reflect + GetTypeRegistration>(
        &self,
        field_name: &str,
    ) -> Result<&T, GetValueError>;

    /// Retrieves a mutable reference to the given type from a field and unwraps immediately.
    /// Will panic if given the wrong type or the field doesn't exist.
    fn get_mut<T: Reflect>(&mut self, field_name: &str) -> &mut T;

    /// Tries to retrieve a mutable reference to the given type.
    fn try_get_mut<T: Reflect + GetTypeRegistration>(
        &mut self,
        field_name: &str,
    ) -> Result<&mut T, GetValueError>;
}

impl GetValue for DynamicStruct {
    fn get<T: Reflect>(&self, field_name: &str) -> &T {
        self.field(field_name).unwrap().downcast_ref::<T>().unwrap()
    }

    fn try_get<T: Reflect + GetTypeRegistration>(
        &self,
        field_name: &str,
    ) -> Result<&T, GetValueError> {
        match self.field(field_name) {
            Some(field) => match field.downcast_ref::<T>() {
                Some(value) => Ok(value),
                None => Err(GetValueError::FailedCast(
                    "DynamicStruct".to_string(),
                    T::get_type_registration().name().to_string(),
                )),
            },
            None => Err(GetValueError::FieldDoesntExist(
                field_name.to_string(),
                T::get_type_registration().name().to_string(),
            )),
        }
    }

    fn get_mut<T: Reflect>(&mut self, field_name: &str) -> &mut T {
        self.field_mut(field_name)
            .unwrap()
            .downcast_mut::<T>()
            .unwrap()
    }

    fn try_get_mut<T: Reflect + GetTypeRegistration>(
        &mut self,
        field_name: &str,
    ) -> Result<&mut T, GetValueError> {
        match self.field_mut(field_name) {
            Some(field) => match field.downcast_mut::<T>() {
                Some(value) => Ok(value),
                None => Err(GetValueError::FailedCast(
                    "DynamicStruct".to_string(),
                    T::get_type_registration().name().to_string(),
                )),
            },
            None => Err(GetValueError::FieldDoesntExist(
                field_name.to_string(),
                T::get_type_registration().name().to_string(),
            )),
        }
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
        let a = Test { i: 5, q: 10 };
        let a: Box<dyn Reflect> = Box::new(a);

        let a = a.cast_ref::<Test>();

        assert_eq!(a.i, 5);
        assert_eq!(a.q, 10);
    }

    #[test]
    fn mut_test() {
        let a = Test { i: 0, q: 0 };

        let mut a = a.clone_dynamic();

        let i = a.get_mut::<i32>("i");

        *i = 10;

        assert_eq!(*a.get::<i32>("i"), 10i32);
    }

    #[test]
    fn auto_cast() {
        let a = Test { i: 15, q: 25 };
        let b = a.clone_dynamic();
        let bi = b.get::<i32>("i");

        assert_eq!(*bi, 15);
    }
}
