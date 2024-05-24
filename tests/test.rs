#[cfg(test)]
mod tests_simple_struct {

    use sqlx_binder::MySqlBinder;

    #[derive(MySqlBinder)]
    pub struct Dog {
        name: String,
        age: u32,
        life_expectancy: u32,
    }

    #[test]
    fn test_get_struct_name() {
        let dog = Dog {
            name: "Taro".to_string(),
            age: 3,
            life_expectancy: 9,
        };
        let struct_name = dog.get_struct_name();

        assert_eq!(struct_name, "Dog");
    }

    #[test]
    fn test_get_struct_name_snake() {

        #[derive(MySqlBinder)]
        struct ThisIsStructName {
            name: String,
        }

        let dog = ThisIsStructName {
            name: "Taro".to_string(),
        };

        let struct_name = dog.get_struct_name_snake();
        assert_eq!(dog.name, String::from("Taro"));
        assert_eq!(struct_name, "this_is_struct_name");
    }

    #[test]
    fn test_get_field_names() {
        let dog = Dog {
            name: "Taro".to_string(),
            age: 3,
            life_expectancy: 9,
        };
        let field_names = dog.get_field_names();

        assert_eq!(field_names[0], "name");
        assert_eq!(field_names[1], "age");
        assert_eq!(field_names[2], "life_expectancy");
    }

    #[test]
    fn test_get_field_enums() {
        let dog = Dog {
            name: "Taro".to_string(),
            age: 3,
            life_expectancy: 9,
        };
        let field_enums = dog.get_field_enums();

        assert_eq!(field_enums[0], DogFieldEnum::name("Taro".to_string()));
        assert_eq!(field_enums[1], DogFieldEnum::age(3));
        assert_eq!(field_enums[2], DogFieldEnum::life_expectancy(9));
    }

    #[test]
    fn test_iterate_with_enum() {
        let dog = Dog {
            name: "Taro".to_string(),
            age: 3,
            life_expectancy: 9,
        };
        let fields = vec![
            "name".to_string(),
            "age".to_string(),
            "life_expectancy".to_string(),
        ];
        let mut fieldvalues: Vec<DogFieldEnum> = vec![];
        for field_name in fields.into_iter() {
            fieldvalues.push(dog.get_enum(&field_name).unwrap());
        }
        assert_eq!(fieldvalues[0], DogFieldEnum::name("Taro".to_string()));
        assert_eq!(fieldvalues[1], DogFieldEnum::age(3));
        assert_eq!(fieldvalues[2], DogFieldEnum::life_expectancy(9));
    }
}

#[cfg(test)]
mod tests_multiple_derive {

    use sqlx_binder::MySqlBinder;

    #[test]
    fn test_multiple_derive() {
        
        #[derive(MySqlBinder)]
        struct Test1 {
            pub name: String,
        }

        #[derive(MySqlBinder)]
        struct Test2 {
            pub name: String,
        }

        let test1 = Test1 { name: String::from("Taro")};
        let test2 = Test2 { name: String::from("Jiro")};

        let field1_names = test1.get_field_names();
        let mut field1_values = vec![];
        for i in field1_names.iter() {
            field1_values.push(test1.get_enum(i).unwrap());
        }
        assert_eq!(field1_values[0], Test1FieldEnum::name("Taro".to_string()));

        let field2_names = test2.get_field_names();
        let mut field2_values = vec![];
        for i in field2_names.iter() {
            field2_values.push(test2.get_enum(i).unwrap());
        }
        assert_eq!(field2_values[0], Test2FieldEnum::name("Jiro".to_string()));

    }
}

#[cfg(test)]
mod tests_binding {

    use sqlx::Execute;
    use sqlx_binder::MySqlBinder;

    #[derive(MySqlBinder)]
    pub struct Dog {
        name: String,
        age: u32,
        life_expectancy: u32,
    }

    #[test]
    fn test_binding() {
        let dog = Dog {
            name: "Taro".to_string(),
            age: 3,
            life_expectancy: 9,
        };
        let params = dog.get_field_enums();
        let mut query: sqlx::query::Query<'_, sqlx::MySql, sqlx::mysql::MySqlArguments> = sqlx::query("INSERT INTO dog VALUES (?,?,?)");
        for param in params {
            query = param.bind(query);
        } 
        assert_eq!(query.take_arguments().unwrap().len(), 3);
    }
}


#[cfg(test)]
mod tests_skip {

    use sqlx_binder::MySqlBinder;

    #[derive(MySqlBinder)]
    struct Skipper {
        name: String,
        #[sqlx_binder(skip)]
        age: u32,
        #[sqlx_binder(skip)]
        sex: String,
        life_expectancy: u32,
    }

    #[test]
    fn test_skip() {
        let dog = Skipper {
            name: "Taro".to_string(),
            age: 3,
            sex: "male".to_string(),
            life_expectancy: 9,
        };
        let field_enums = dog.get_field_enums();
        assert_eq!(dog.age, 3);
        assert_eq!(dog.sex, String::from("male"));
        assert_eq!(field_enums[0], SkipperFieldEnum::name("Taro".to_string()));
        assert_eq!(field_enums[1], SkipperFieldEnum::life_expectancy(9));
    }
}