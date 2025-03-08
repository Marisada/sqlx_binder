# sqlx binder
Procedural Macros for binding struct field to SQLx's Query by generating `StructFieldEnums` to `bind` sqlx::Query in loop,
useful when insert/update large struct.

## Overview

Add sqlx_binder and SQLx to your dependencies
```toml
sqlx_binder = { git = "https://github.com/marisada/sqlx_binder" }
sqlx = { version = "0.8", features = [ "mysql", "runtime-tokio" ] }
```

Add `MySqlBinder` to your Struct

```rust
use sqlx::{mysql::MySqlQueryResult, MySql, Pool};
use sqlx_binder::MySqlBinder;

#[derive(MySqlBinder)]
struct Dog {
    id: u32,
    name: String,
    age: u32,
    life_expectancy: u32,
}
```

Will implement `insert` and `update` methods and generate `StructNameFieldEnum` with `bind` method.

```rust
impl Dog {
    pub fn insert(
        &self,
        primary_key: Option<&str>,
        custom_table_name: Option<&str>,
        extra_column: &str,
        extra_statement: &str,
        extra_values: &[&str],
        pool: &Pool<MySql>,
        db_name: &str,
    ) -> sqlx::Result<MySqlQueryResult>;

    pub fn update(&self,
        primary_key: &str,
        custom_table_name: Option<&str>,
        extra_column: &str,
        extra_values: &[&str],
        pool: &Pool<MySql>,
        db_name: &str,
    ) -> sqlx::Result<MySqlQueryResult>;

    pub fn get_enum(&self, field_string: &String) -> Result<DogFieldEnum, String>;
    pub fn get_struct_name(&self) -> &'static str;
    pub fn get_struct_name_snake(&self) -> String;
    pub fn get_field_names(&self) -> Vec<&'static str>;
    pub fn get_field_enums(&self) -> Vec<DogFieldEnum>;
}

pub enum DogFieldEnum {
    id(u32),
    name(String),
    age(u32),
    life_expectancy(u32),
}

impl DogFieldEnum {
    pub fn bind(&self, query: sqlx::Query) -> sqlx::Query;
}
```

## Bind
Binding FieldEnum's value with sqlx's Query in loop
```rust
    let params = dog.get_field_enums(); 
    let mut query = sqlx::query(&sql);
    for param in params {
        query = param.bind(query);
    }
    let result = query.execute(&pool).await?;
```
is the same as writing `bind()` to all fields
```rust
    let result = sqlx::query(&sql)
        .bind(dog.id)
        .bind(&dog.name)
        .bind(dog.age)
        .bind(dog.life_expectancy)
        .execute(&pool).await?;
```

## Insert
Insert struct into database with
```rust
let dog = Dog::new();
let result = dog.insert(Some("id"), Some("tbl_dog"), ",color,breed", ",?,?", &["white","Poodle"], &pool, "animal").await?;
```
is the same as
```rust
let dog = Dog::new();
let color = "white";
let breed = "Poodle"
let sql = "INSERT INTO animal.tbl_dog (name,age,life_expectancy,color,breed) VALUES (?,?,?,?,?);";
let result = sqlx::query(&sql)
    .bind(&dog.name)
    .bind(dog.age)
    .bind(dog.life_expectancy)
    .bind(color)
    .bind(breed)
    .execute(&pool).await?;
```

## Update
Update struct into database with
```rust
let dog = Dog::new();
let result = dog.update("id", Some("tbl_dog"), ",color=?,breed=?", &["white","Poodle"], &pool, "animal").await?;
```
is the same as
```rust
let dog = Dog::load();
let color = "white";
let breed = "Poodle"
let sql = "UPDATE animal.tbl_dog SET name=?,age=?,life_expectancy=?,color=?,breed=? WHERE id=?;";
let result = sqlx::query(&sql)
    .bind(&dog.name)
    .bind(dog.age)
    .bind(dog.life_expectancy)
    .bind(color)
    .bind(breed)
    .bind(dog.id)
    .execute(&pool).await?;
```

## Field Attributes
### rename
```rust
#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    #[sqlx_binder(rename = "gender")]
    sex: u32,
}
```
get_field_names method will return `["name", "age", "gender"]`
> Note: DogFieldEnum of sex will NOT change, still `DogFieldEnum::sex(u32)`

### skip
```rust
#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    #[sqlx_binder(skip)]
    life_expectancy: u32,
}
```
will skip Struct field `life_expectancy` so we get DogFieldEnum like this
```rust
enum DogFieldEnum {
    name(String),
    age(u32),
}
```

## Struct Methods
### get_enum
```rust
fn get_enum(&self, field_string: &String) -> Result<StructNameFieldEnum, String>
```
get a single enum of field's value. Enum name is `Struct name` + `FieldEnum`.  
with varients such as `MyStructFieldEnum::Name(String)` from `MyStruct { name: String }`

### get_struct_name
```rust
fn get_struct_name(&self) -> &'static str
```
get a struct's names, in UpperCamelCase string
> most sql database converts all table names to lowercase on storage and lookup,  
> so `SomeStructName` will be `somestructname`, not `some_struct_name`
> but we need a snake_case one by using `get_struct_name_snake` method below

### get_struct_name_snake
```rust
fn get_struct_name_snake(&self) -> String
```
get a Struct name, in snake_case string  
ex: `SomeStructName` -> `some_struct_name`

### get_field_names
```rust
fn get_field_names(&self) -> Vec<&'static str>
```
get all Struct's field names, in snake_case string

### get_field_enums
```rust
fn get_field_enums(&self) -> Vec<StructNameFieldEnum>
```
get all Struct's field enums.

## Usage and Example

```rust
use sqlx_binder::MySqlBinder;
use sqlx::{
    mysql::{MySqlPoolOptions, MySqlConnectOptions},
    Row, Executor,
}; 

#[derive(MySqlBinder)]
struct Dog {
    name: String,
    age: u32,
    #[sqlx_binder(skip)]
    sex: String,
    life_expectancy: u32,
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
        sex: "male".to_string(),
        life_expectancy: 9,
    };

    // dog.get_struct_name() return 'Dog' but our table name is 'dog',   
    // database will change to lowercase 'dog' for us so we can use 'Dog'.
    // If your table name is 'some_table_name' then
    // you need .get_struct_name_snake() to change 'SomeTableName' to 'some_table_name' 
    let struct_name = dog.get_struct_name();
    let mut field_names = dog.get_field_names();
    let mut params = dof.get_field_enums();

    // INSERT QUERY
    let sql = [
        "INSERT INTO ", &struct_name, " (", 
            &field_names.join(","), 
        ") VALUES (",
            &vec!["?";field_names.len()].join(","),
        ");"
    ].join("");
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

    let sql = [
        "UPDATE ", &struct_name, 
        " SET ", &field_names.iter().map(|name| [name, "=?"].join("")).collect::<Vec<String>>().join(","),
        // use `where` name here
        " WHERE ", &name_removed, "=?;"
    ].join("");
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
- [Field Accessor](https://github.com/europeanplaice/field_accessor) by Tomohiro Endo (europeanplaice@gmail.com)
- [struct-field-names-as-array](https://github.com/jofas/struct_field_names_as_array) by Jonas Fassbender (jonas@fc-web.de)