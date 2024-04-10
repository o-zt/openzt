#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use field_accessor_as_string::FieldAccessorAsString;

    #[derive(Default, FieldAccessorAsString)]
    struct MyStruct {
        field1: String,
        field2: i32,
        #[allow(dead_code)]
        #[skip_field]
        field3: i32,
    }


    #[derive(Default, FieldAccessorAsString)]
    struct MyChildStruct {
        #[deref_field]
        my_struct: MyStruct,
        child_field1: String,
        child_field2: i32,
    }

    impl Deref for MyChildStruct {
        type Target = MyStruct;

        fn deref(&self) -> &Self::Target {
            &self.my_struct
        }
    }


    #[test]
    fn test_set_fields() {
        let mut my_struct = MyStruct::default();
        
        let result = my_struct.set_field("field1", "hello");
        assert!(result.is_ok());
        assert_eq!(my_struct.field1, "hello");

        let result = my_struct.set_field("field2", "42");
        assert!(result.is_ok());
        assert_eq!(my_struct.field2, 42);

        let result = my_struct.set_field("field2", "not_a_number");
        assert!(result.is_err());
        assert_eq!(my_struct.field2, 42);

        let result = my_struct.set_field("nonexistent_field", "hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_deref_set_fields() {
        let mut my_child_struct = MyChildStruct::default();
        
        let result = my_child_struct.set_field("child_field1", "hello");
        assert!(result.is_ok());
        assert_eq!(my_child_struct.child_field1, "hello");

        let result = my_child_struct.set_field("child_field2", "42");
        assert!(result.is_ok());
        assert_eq!(my_child_struct.child_field2, 42);

        let result = my_child_struct.set_field("field1", "hello");
        assert!(result.is_ok());
        assert_eq!(my_child_struct.my_struct.field1, "hello");

        let result = my_child_struct.set_field("field2", "42");
        assert!(result.is_ok());
        assert_eq!(my_child_struct.my_struct.field2, 42);

        let result = my_child_struct.set_field("field2", "not_a_number");
        assert!(result.is_err());
        assert_eq!(my_child_struct.my_struct.field2, 42);

        let result = my_child_struct.set_field("nonexistent_field", "hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_fields() {
        let my_struct = MyStruct{field1: "hello".to_string(), field2: 42, field3: 43};
        
        let result = my_struct.get_field("field1");
        assert!(result.is_ok());
        assert!(result.unwrap() == "hello");

        let result = my_struct.get_field("field2");
        assert!(result.is_ok());
        assert!(result.unwrap() == "42");

        let result = my_struct.get_field("nonexistent_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_deref_get_fields() {
        let my_child_struct = MyChildStruct{my_struct: MyStruct{field1: "hello".to_string(), field2: 42, field3: 43}, child_field1: "hello".to_string(), child_field2: 42};
        
        let result = my_child_struct.get_field("child_field1");
        assert!(result.is_ok());
        assert!(result.unwrap() == "hello");

        let result = my_child_struct.get_field("child_field2");
        assert!(result.is_ok());
        assert!(result.unwrap() == "42");

        let result = my_child_struct.get_field("field1");
        assert!(result.is_ok());
        assert!(result.unwrap() == "hello");

        let result = my_child_struct.get_field("field2");
        assert!(result.is_ok());
        assert!(result.unwrap() == "42");

        let result = my_child_struct.get_field("nonexistent_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_field() {
        let my_struct = MyStruct::default();

        assert!(my_struct.is_field("field1"));
        assert!(my_struct.is_field("field2"));
        assert!(!my_struct.is_field("field3"));
        assert!(!my_struct.is_field("nonexistent_field"));
    }

    #[test]
    fn test_deref_is_field() {
        let my_child_struct = MyChildStruct::default();

        assert!(my_child_struct.is_field("child_field1"));
        assert!(my_child_struct.is_field("child_field2"));
        assert!(my_child_struct.is_field("field1"));
        assert!(my_child_struct.is_field("field2"));
        assert!(!my_child_struct.is_field("field3"));
        assert!(!my_child_struct.is_field("nonexistent_field"));
    }
}