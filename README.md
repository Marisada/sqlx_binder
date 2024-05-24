# sqlx binder
proc macro for binding struct field to sqlx's Query by generating StructFieldEnums to bind sqlx::Query in loop,
useful when insert/update large struct

from Struct

```rust
#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    life_expectancy: u32,
}
```

generate Enum

```rust
enum DogFieldEnum {
    name(String),
    age(u32),
    life_expectancy(u32),
}
```

with `bind` method to bind `sqlx::query::Query` type as below
```rust
let query: sqlx::query::Query<'_, sqlx::MySql, sqlx::mysql::MySqlArguments> = sqlx::query(&sql);
```

### `bind`
```rust
fn bind(&self, query: sqlx::Query) -> sqlx::Query;
```
bind FieldEnum's value to sqlx's Query like

```rust
    let result = sqlx::Query(&sql)
        .bind(&dog.name)
        .bind(&dog.age)
        .bind(&dog.life_expectancy)
        .execute(&pool).await?;
```
with
```rust
    let params = dog.get_field_enums(); 
    let mut query = sqlx::query(&sql);
    for param in params {
        query = param.bind(query);
    }
    let result = query.execute(&pool).await?;
```

## Field Attribute
### skip
```rust
#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    #[sqlx_binding(skip)]
    life_expectancy: u32,
}
```
will skip struct field 'life_expectancy' so we get DogFieldEnum like this
```rust
enum DogFieldEnum {
    name(String),
    age(u32),
}
```

## Struct Method

### `get_enum`
```rust
fn get_enum(&self, field_string: &String) -> Result<StructNameFieldEnum, String>;
```
get a single enum of field's value. Enum name is `Struct name` + `FieldEnum`.  
with varients such as `MyStructFieldEnum::Name(String)` from MyStruct { name: String }

### `get_struct_name`
```rust
fn get_struct_name(&self) -> '&'static str';
```
get a struct's names, in UpperCamelCase string
> most sql database converts all table names to lowercase on storage and lookup,  
> so 'SomeStructName' will be 'somestructname'.. not 'some_struct_name'
> but we need snake_case so look below

### `get_struct_name_snake`
```rust
fn get_struct_name_snake(&self) -> String;
```
get a struct's names, in snake_case string  
ex: SomeStructName -> some_struct_name

### `get_field_names`
```rust
fn get_field_names(&self) -> Vec<'&'static str'>;
```
get all struct's field names, in snake_case string

### `get_field_enums`
```rust
fn get_field_enums(&self) -> Vec<StructNameFieldEnum>;
```
get all struct's field enums.

## Usage and Example

```rust
#[macro_use(concat_string)]
extern crate concat_string;

use sqlx_binder::MySqlBinder;
use sqlx::{
    mysql::{MySqlPoolOptions, MySqlConnectOptions},
    Row, Executor,
}; 

#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    life_expectancy: u32,
    #[sqlx_binder(skip)]
    sex: String,
}

#[tokio::main]
async fn main() {

    let conn = MySqlConnectOptions::new()
        .host("127.0.0.1")
        .port(3306)
        .username("root")
        .database("SQLx")
        .charset("utf8");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(conn).await
        .expect("Fail initiate MySQL connection");   

    let drop_dog = "DROP TABLE IF EXISTS dog;";
    let create_dog = r#"
        CREATE TABLE dog (
        name VARCHAR(26) NOT NULL,
        age TINYINT NOT NULL,
        life_expectancy TINYINT NOT NULL
        ) Engine = InnoDB;
    "#;
    let _ = sqlx::query(&drop_dog).execute(&pool).await.unwrap();
    let _ = sqlx::query(&create_dog).execute(&pool).await.unwrap();

    let mut dog = Dog {
        name: "Taro".to_string(),
        age: 3,
        life_expectancy: 9,
        sex: "male".to_string(),
    };

    // dog.get_struct_name() return 'Dog' but our table name is 'dog',   
    // database will change to lowercase 'dog' for us so we can use 'Dog'.
    // If your table name is 'some_table_name' then
    // you need .get_struct_name_snake() to change 'SomeTableName' to 'some_table_name' 
    let struct_name = dog.get_struct_name();
    let mut field_names = dog.get_field_names();
    let mut params = dof.get_field_enums();

    // INSERT QUERY
    let sql = concat_string!(
        "INSERT INTO ", struct_name, " (", 
            field_names.join(","), 
        ") VALUES (",
            vec!["?";field_names.len()].join(","),
        ");"
    );
    let mut query = sqlx::Query(&sql);
    for param in params {
        query = param.bind(query);
    }
    let result = query.execute(&pool).await.unwrap();
    println!("insert dog with payload : {:?}", &result);

    // UPDATE QUERY
    dog.name = "Jiro".to_string();
    dog.life_expectancy = 7;

    let position = field_names.iter().position(|name| *name == "age").unwrap();
    // remove 'where' name and param
    let name_removed = field_names.swap_remove(position);
    let param_removed = params.swap_remove(position);

    let sql = concat_string!(
        "UPDATE ", struct_name, 
        " SET ", field_names.iter().map(|name| concat_string!(name, "=?")).collect::<Vec<String>>().join(","),
        // use `where` name here
        " WHERE ", name_removed, "=?;"
    );
    let mut query = sqlx::Query(&sql);
    for param in params {
        query = param.bind(query);
    }
    // bind 'where' param here
    query = param_removed.bind(query);
    let result = query.execute(&pool).await.unwrap();
    println!("update dog with payload : {:?}", &result);
}

```

## Inspiration
- [Field Accessor](https://github.com/europeanplaice/field_accessor)  
by Tomohiro Endo (europeanplaice@gmail.com)

- [struct-field-names-as-array](https://github.com/jofas/struct_field_names_as_array)  
by Jonas Fassbender (jonas@fc-web.de)