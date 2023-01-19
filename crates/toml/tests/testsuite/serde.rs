use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::BTreeMap;

use toml::map::Map;
use toml::Table;
use toml::Value;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with {}", stringify!($e), e),
        }
    };
}

macro_rules! equivalent {
    ($literal:expr, $toml:expr,) => {{
        let toml = $toml;
        let literal = $literal;

        // In/out of Value is equivalent
        println!("try_from");
        assert_eq!(t!(Table::try_from(literal.clone())), toml);
        println!("try_into");
        assert_eq!(literal, t!(toml.clone().try_into()));

        // Through a string equivalent
        println!("to_string(literal)");
        snapbox::assert_eq(toml.to_string(), t!(toml::to_string(&literal)));
        println!("to_string(toml)");
        snapbox::assert_eq(toml.to_string(), t!(toml::to_string(&toml)));
        println!("literal, from_str(toml)");
        assert_eq!(literal, t!(toml::from_str(&toml.to_string())));
        println!("toml, from_str(toml)");
        assert_eq!(toml, t!(toml::from_str(&toml.to_string())));
    }};
}

macro_rules! error {
    ($ty:ty, $toml:expr, $msg_parse:expr, $msg_decode:expr) => {{
        println!("attempting parsing");
        match toml::from_str::<$ty>(&$toml.to_string()) {
            Ok(_) => panic!("successful"),
            Err(e) => snapbox::assert_eq($msg_parse, e.to_string()),
        }

        println!("attempting toml decoding");
        match $toml.try_into::<$ty>() {
            Ok(_) => panic!("successful"),
            Err(e) => snapbox::assert_eq($msg_decode, e.to_string()),
        }
    }};
}

macro_rules! map( ($($k:ident: $v:expr),*) => ({
    let mut _m = Map::new();
    $(_m.insert(stringify!($k).to_string(), t!(Value::try_from($v)));)*
    _m
}) );

#[test]
fn smoke() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: isize,
    }

    equivalent!(Foo { a: 2 }, map! { a: Value::Integer(2) },);
}

#[test]
fn smoke_hyphen() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a_b: isize,
    }

    equivalent! {
        Foo { a_b: 2 },
        map! { a_b: Value::Integer(2)},
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo2 {
        #[serde(rename = "a-b")]
        a_b: isize,
    }

    let mut m = Map::new();
    m.insert("a-b".to_string(), Value::Integer(2));
    equivalent! {
        Foo2 { a_b: 2 },
        m,
    }
}

#[test]
fn nested() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: isize,
        b: Bar,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bar {
        a: String,
    }

    equivalent! {
        Foo { a: 2, b: Bar { a: "test".to_string() } },
        map! {
            a: Value::Integer(2),
            b: map! {
                a: Value::String("test".to_string())
            }
        },
    }
}

#[test]
fn application_decode_error() {
    #[derive(PartialEq, Debug)]
    struct Range10(usize);
    impl<'de> serde::Deserialize<'de> for Range10 {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Range10, D::Error> {
            let x: usize = serde::Deserialize::deserialize(d)?;
            if x > 10 {
                Err(serde::de::Error::custom("more than 10"))
            } else {
                Ok(Range10(x))
            }
        }
    }
    let d_good = Value::Integer(5);
    let d_bad1 = Value::String("not an isize".to_string());
    let d_bad2 = Value::Integer(11);

    assert_eq!(Range10(5), d_good.try_into().unwrap());

    let err1: Result<Range10, _> = d_bad1.try_into();
    assert!(err1.is_err());
    let err2: Result<Range10, _> = d_bad2.try_into();
    assert!(err2.is_err());
}

#[test]
fn array() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Vec<isize>,
    }

    equivalent! {
        Foo { a: vec![1, 2, 3, 4] },
        map! {
            a: Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4)
            ])
        },
    };
}

#[test]
fn inner_structs_with_options() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Option<Box<Foo>>,
        b: Bar,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bar {
        a: String,
        b: f64,
    }

    equivalent! {
        Foo {
            a: Some(Box::new(Foo {
                a: None,
                b: Bar { a: "foo".to_string(), b: 4.5 },
            })),
            b: Bar { a: "bar".to_string(), b: 1.0 },
        },
        map! {
            a: map! {
                b: map! {
                    a: Value::String("foo".to_string()),
                    b: Value::Float(4.5)
                }
            },
            b: map! {
                a: Value::String("bar".to_string()),
                b: Value::Float(1.0)
            }
        },
    }
}

#[test]
#[cfg(feature = "preserve_order")]
fn hashmap() {
    use std::collections::HashSet;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        set: HashSet<char>,
        map: BTreeMap<String, isize>,
    }

    equivalent! {
        Foo {
            set: {
                let mut s = HashSet::new();
                s.insert('a');
                s
            },
            map: {
                let mut m = BTreeMap::new();
                m.insert("bar".to_string(), 4);
                m.insert("foo".to_string(), 10);
                m
            }
        },
        map! {
            set: Value::Array(vec![Value::String("a".to_string())]),
            map: map! {
                bar: Value::Integer(4),
                foo: Value::Integer(10)
            }
        },
    }
}

#[test]
fn table_array() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Vec<Bar>,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bar {
        a: isize,
    }

    equivalent! {
        Foo { a: vec![Bar { a: 1 }, Bar { a: 2 }] },
        map! {
            a: Value::Array(vec![
                Value::Table(map!{ a: Value::Integer(1) }),
                Value::Table(map!{ a: Value::Integer(2) }),
            ])
        },
    }
}

#[test]
fn type_errors() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Foo {
        bar: isize,
    }

    error! {
        Foo,
        map! {
            bar: Value::String("a".to_string())
        },
        r#"TOML parse error at line 1, column 7
  |
1 | bar = "a"
  |       ^^^
invalid type: string "a", expected isize
"#,
        "invalid type: string \"a\", expected isize\n"
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Bar {
        foo: Foo,
    }

    error! {
        Bar,
        map! {
            foo: map! {
                bar: Value::String("a".to_string())
            }
        },
        r#"TOML parse error at line 2, column 7
  |
2 | bar = "a"
  |       ^^^
invalid type: string "a", expected isize
"#,
        "invalid type: string \"a\", expected isize\n"
    }
}

#[test]
fn missing_errors() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        bar: isize,
    }

    error! {
        Foo,
        map! { },
        r#"TOML parse error at line 1, column 1
  |
1 | 
  | ^
missing field `bar`
"#,
        "missing field `bar`\n"
    }
}

#[test]
fn parse_enum() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: E,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    #[serde(untagged)]
    enum E {
        Bar(isize),
        Baz(String),
        Last(Foo2),
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo2 {
        test: String,
    }

    equivalent! {
        Foo { a: E::Bar(10) },
        map! { a: Value::Integer(10) },
    }

    equivalent! {
        Foo { a: E::Baz("foo".to_string()) },
        map! { a: Value::String("foo".to_string()) },
    }

    equivalent! {
        Foo { a: E::Last(Foo2 { test: "test".to_string() }) },
        map! { a: map! { test: Value::String("test".to_string()) } },
    }
}

#[test]
fn parse_enum_string() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Sort,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    #[serde(rename_all = "lowercase")]
    enum Sort {
        Asc,
        Desc,
    }

    equivalent! {
        Foo { a: Sort::Desc },
        map! { a: Value::String("desc".to_string()) },
    }
}

// #[test]
// fn unused_fields() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: isize }
//
//     let v = Foo { a: 2 };
//     let mut d = Decoder::new(Table(map! {
//         a, Integer(2),
//         b, Integer(5)
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, Some(Table(map! {
//         b, Integer(5)
//     })));
// }
//
// #[test]
// fn unused_fields2() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: Bar }
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Bar { a: isize }
//
//     let v = Foo { a: Bar { a: 2 } };
//     let mut d = Decoder::new(Table(map! {
//         a, Table(map! {
//             a, Integer(2),
//             b, Integer(5)
//         })
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, Some(Table(map! {
//         a, Table(map! {
//             b, Integer(5)
//         })
//     })));
// }
//
// #[test]
// fn unused_fields3() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: Bar }
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Bar { a: isize }
//
//     let v = Foo { a: Bar { a: 2 } };
//     let mut d = Decoder::new(Table(map! {
//         a, Table(map! {
//             a, Integer(2)
//         })
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, None);
// }
//
// #[test]
// fn unused_fields4() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: BTreeMap<String, String> }
//
//     let v = Foo { a: map! { a, "foo".to_string() } };
//     let mut d = Decoder::new(Table(map! {
//         a, Table(map! {
//             a, Value::String("foo".to_string())
//         })
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, None);
// }
//
// #[test]
// fn unused_fields5() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: Vec<String> }
//
//     let v = Foo { a: vec!["a".to_string()] };
//     let mut d = Decoder::new(Table(map! {
//         a, Array(vec![Value::String("a".to_string())])
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, None);
// }
//
// #[test]
// fn unused_fields6() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: Option<Vec<String>> }
//
//     let v = Foo { a: Some(vec![]) };
//     let mut d = Decoder::new(Table(map! {
//         a, Array(vec![])
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, None);
// }
//
// #[test]
// fn unused_fields7() {
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Foo { a: Vec<Bar> }
//     #[derive(Serialize, Deserialize, PartialEq, Debug)]
//     struct Bar { a: isize }
//
//     let v = Foo { a: vec![Bar { a: 1 }] };
//     let mut d = Decoder::new(Table(map! {
//         a, Array(vec![Table(map! {
//             a, Integer(1),
//             b, Integer(2)
//         })])
//     }));
//     assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
//
//     assert_eq!(d.toml, Some(Table(map! {
//         a, Array(vec![Table(map! {
//             b, Integer(2)
//         })])
//     })));
// }

#[test]
fn empty_arrays() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Vec<Bar>,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bar;

    equivalent! {
        Foo { a: vec![] },
        map! {a: Value::Array(Vec::new())},
    }
}

#[test]
fn empty_arrays2() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Foo {
        a: Option<Vec<Bar>>,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bar;

    equivalent! {
        Foo { a: None },
        map! {},
    }

    equivalent! {
        Foo { a: Some(vec![]) },
        map! { a: Value::Array(vec![]) },
    }
}

#[test]
fn extra_keys() {
    #[derive(Serialize, Deserialize)]
    struct Foo {
        a: isize,
    }

    let toml = map! { a: Value::Integer(2), b: Value::Integer(2) };
    assert!(toml.clone().try_into::<Foo>().is_ok());
    assert!(toml::from_str::<Foo>(&toml.to_string()).is_ok());
}

#[test]
fn newtypes() {
    #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
    struct A {
        b: B,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
    struct B(u32);

    equivalent! {
        A { b: B(2) },
        map! { b: Value::Integer(2) },
    }
}

#[test]
fn newtypes2() {
    #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
    struct A {
        b: B,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
    struct B(Option<C>);

    #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
    struct C {
        x: u32,
        y: u32,
        z: u32,
    }

    equivalent! {
        A { b: B(Some(C { x: 0, y: 1, z: 2 })) },
        map! {
            b: map! {
                x: Value::Integer(0),
                y: Value::Integer(1),
                z: Value::Integer(2)
            }
        },
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct CanBeEmpty {
    a: Option<String>,
    b: Option<String>,
}

#[test]
fn table_structs_empty() {
    let text = "[bar]\n\n[baz]\n\n[bazv]\na = \"foo\"\n\n[foo]\n";
    let value: BTreeMap<String, CanBeEmpty> = toml::from_str(text).unwrap();
    let mut expected: BTreeMap<String, CanBeEmpty> = BTreeMap::new();
    expected.insert("bar".to_string(), CanBeEmpty::default());
    expected.insert("baz".to_string(), CanBeEmpty::default());
    expected.insert(
        "bazv".to_string(),
        CanBeEmpty {
            a: Some("foo".to_string()),
            b: None,
        },
    );
    expected.insert("foo".to_string(), CanBeEmpty::default());
    assert_eq!(value, expected);
    snapbox::assert_eq(text, toml::to_string(&value).unwrap());
}

#[test]
fn fixed_size_array() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Entity {
        pos: [i32; 2],
    }

    equivalent! {
        Entity { pos: [1, 2] },
        map! {
            pos: Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
            ])
        },
    }
}

#[test]
fn homogeneous_tuple() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Collection {
        elems: (i64, i64, i64),
    }

    equivalent! {
        Collection { elems: (0, 1, 2) },
        map! {
            elems: Value::Array(vec![
                Value::Integer(0),
                Value::Integer(1),
                Value::Integer(2),
            ])
        },
    }
}

#[test]
fn homogeneous_tuple_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Object(Vec<String>, Vec<String>, Vec<String>);

    equivalent! {
        map! {
            obj: Object(vec!["foo".to_string()], vec![], vec!["bar".to_string(), "baz".to_string()])
        },
        map! {
            obj: Value::Array(vec![
                Value::Array(vec![
                    Value::String("foo".to_string()),
                ]),
                Value::Array(vec![]),
                Value::Array(vec![
                    Value::String("bar".to_string()),
                    Value::String("baz".to_string()),
                ]),
            ])
        },
    }
}

#[test]
fn json_interoperability() {
    #[derive(Serialize, Deserialize)]
    struct Foo {
        any: toml::Value,
    }

    let _foo: Foo = serde_json::from_str(
        r#"
        {"any":1}
    "#,
    )
    .unwrap();
}

#[test]
fn table_type_enum_regression_issue_388() {
    #[derive(Deserialize)]
    struct DataFile {
        #[allow(dead_code)]
        data: Compare,
    }

    #[derive(Deserialize)]
    enum Compare {
        Gt(u32),
    }

    let dotted_table = r#"
        data.Gt = 5
        "#;
    assert!(toml::from_str::<DataFile>(dotted_table).is_ok());

    let inline_table = r#"
        data = { Gt = 5 }
        "#;
    assert!(toml::from_str::<DataFile>(inline_table).is_ok());
}